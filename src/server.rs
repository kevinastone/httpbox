use futures::prelude::*;
use hyper::Request as HTTPRequest;
use hyper::body::{Body, Incoming};
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::task::TaskTracker;
use tower::Service;
use tower::ServiceExt;

pub trait Connected {
    fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }
}

impl Connected for TcpStream {
    fn remote_addr(&self) -> Option<SocketAddr> {
        self.peer_addr().ok()
    }
}

#[cfg(test)]
impl Connected for tokio::io::DuplexStream {}

pub struct Server<C, S, F> {
    conn_stream: C,
    service: S,
    shutdown_signal: F,
}

impl<S> Server<TcpListenerStream, S, future::Pending<()>> {
    pub fn new(listener: TcpListener, service: S) -> Self {
        Self::from_stream(TcpListenerStream::new(listener), service)
    }
}

impl<C, S> Server<C, S, future::Pending<()>> {
    pub fn from_stream(conn_stream: C, service: S) -> Self {
        Self {
            conn_stream,
            service,
            shutdown_signal: future::pending(),
        }
    }
}

impl<C, S, F> Server<C, S, F> {
    pub fn with_graceful_shutdown<Fut: Future>(
        self,
        fut: Fut,
    ) -> Server<C, S, Fut> {
        Server {
            conn_stream: self.conn_stream,
            service: self.service,
            shutdown_signal: fut,
        }
    }
}

impl<C, S, F, I, RespBody, E> Server<C, S, F>
where
    C: Stream<Item = std::io::Result<I>> + Unpin + Send + 'static,
    I: AsyncRead + AsyncWrite + Connected + Send + Unpin + 'static,
    F: Future + Send + 'static,
    S: Service<
            HTTPRequest<Incoming>,
            Response = hyper::Response<RespBody>,
            Error = E,
        > + Clone
        + Send
        + 'static,
    S::Future: Send,
    RespBody: Body + Send + 'static,
    RespBody::Data: Send,
    RespBody::Error: Sync + Send + std::error::Error,
    E: Send + Sync + std::error::Error + 'static,
{
    pub async fn serve(self) -> std::io::Result<()> {
        let (close_tx, close_rx) = watch::channel(());
        let tracker = TaskTracker::new();

        let service = self.service;
        let conn_stream = self.conn_stream;

        let mut auto_builder = auto::Builder::new(TokioExecutor::new());
        auto_builder
            .http1()
            .timer(TokioTimer::new())
            .header_read_timeout(Duration::from_secs(30));
        auto_builder
            .http2()
            .timer(TokioTimer::new())
            .keep_alive_interval(Duration::from_secs(60))
            .keep_alive_timeout(Duration::from_secs(10));

        let tracker_spawn = tracker.clone();
        let conn_stream = conn_stream
            .take_until(self.shutdown_signal)
            .and_then(move |stream| {
                let auto_builder = auto_builder.clone();
                let mut close_rx = close_rx.clone();
                let service = service.clone();
                let tracker = tracker_spawn.clone();

                async move {
                    let addr = stream.remote_addr();
                    let stream = TokioIo::new(stream);

                    // Inject the client addr into the request if available
                    let tower_service = service.map_request(
                        move |mut req: HTTPRequest<_>| {
                            if let Some(addr) = addr {
                                req.extensions_mut().insert(addr);
                            }
                            req
                        },
                    );

                    tracker.spawn(async move {
                        let hyper_service = hyper::service::service_fn(
                            move |request: HTTPRequest<_>| {
                                tower_service.clone().call(request)
                            },
                        );

                        let conn = auto_builder
                            .serve_connection_with_upgrades(stream, hyper_service);

                        let mut conn = std::pin::pin!(conn);

                        loop {
                            tokio::select! {
                                // Poll the connection. This completes when the client has closed the
                                // connection, graceful shutdown has completed, or we encounter a TCP error.
                                result = conn.as_mut() => {
                                    if let Err(err) = result {
                                        tracing::error!("Error serving connection: {err:#}");
                                    }
                                    break;
                                }
                                // Start graceful shutdown when we receive a shutdown signal.
                                //
                                // We use a loop to continue polling the connection to allow requests to finish
                                _ = close_rx.changed() => {
                                    tracing::debug!("signal received, starting graceful shutdown");
                                    conn.as_mut().graceful_shutdown();
                                }
                            }
                        }
                    });

                    Ok(())
                }
            });

        // Run the listener stream to completion
        let _ = conn_stream.map(Ok).forward(futures::sink::drain()).await;

        // Stop accepting new tracked tasks and notify existing connections to start graceful shutdown
        tracker.close();
        let _ = close_tx.send(());

        // Wait for all connection tasks to complete or timeout after 10 seconds.
        tracing::debug!("waiting for {} tasks to finish", tracker.len());
        if tokio::time::timeout(Duration::from_secs(10), tracker.wait())
            .await
            .is_err()
        {
            tracing::warn!(
                "shutdown timeout reached with {} active connections remaining; forcing exit",
                tracker.len()
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::{future, stream};
    use http_body_util::Empty;
    use hyper::client::conn::http2;

    #[tokio::test]
    async fn test_http2_cleartext_request() {
        let (client_conn, server_conn) = tokio::io::duplex(64 * 1024);
        let incoming =
            stream::once(future::ok(server_conn)).chain(stream::pending());

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let server = Server::from_stream(incoming, crate::service::router())
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            });

        let server_task = tokio::spawn(async move {
            server.serve().await.unwrap();
        });

        let stream = TokioIo::new(client_conn);
        let (mut sender, conn) = http2::handshake(TokioExecutor::new(), stream)
            .await
            .expect("HTTP/2 handshake failed");

        tokio::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("HTTP/2 connection error: {err}");
            }
        });

        let req = hyper::Request::builder()
            .uri("http://localhost/healthz")
            .version(hyper::Version::HTTP_2)
            .body(Empty::<hyper::body::Bytes>::new())
            .unwrap();

        let res = sender.send_request(req).await.unwrap();

        assert_eq!(res.version(), hyper::Version::HTTP_2);
        assert_eq!(res.status(), hyper::http::StatusCode::OK);

        let _ = shutdown_tx.send(());
        let _ = server_task.await;
    }
}
