use futures::prelude::*;
use hyper::Request as HTTPRequest;
use hyper::body::{Body, Incoming};
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio_stream::wrappers::TcpListenerStream;
use tower::Service;
use tower::ServiceExt;

pub struct Server<S, F> {
    conn_stream: TcpListenerStream,
    service: S,
    shutdown_signal: F,
}

impl<S, RespBody, E> Server<S, future::Pending<()>>
where
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
    pub fn new(listener: TcpListener, service: S) -> Self {
        Self {
            conn_stream: TcpListenerStream::new(listener),
            service,
            shutdown_signal: future::pending(),
        }
    }

    pub fn with_graceful_shutdown<Fut: Future>(
        self,
        fut: Fut,
    ) -> Server<S, Fut> {
        Server {
            conn_stream: self.conn_stream,
            service: self.service,
            shutdown_signal: fut,
        }
    }
}

impl<S, F, RespBody, E> Server<S, F>
where
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

        let conn_close_rx = close_rx.clone();
        let conn_stream = conn_stream
            .take_until(self.shutdown_signal)
            .and_then(move |stream| {
                let auto_builder = auto_builder.clone();
                let mut close_rx = conn_close_rx.clone();
                let service = service.clone();

                async move {
                    let addr = stream.peer_addr()?;
                    let stream = TokioIo::new(stream);

                    // Inject the client addr into the request
                    let tower_service = service.map_request(
                        move |mut req: HTTPRequest<_>| {
                            req.extensions_mut().insert(addr);
                            req
                        },
                    );

                    tokio::task::spawn(async move {
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

                    // Drop the watch receiver to signal to `main` that this task is done.
                    drop(close_rx);
                });

                Ok(())
            }});

        // Run the listener stream to completion
        let _ = conn_stream.map(Ok).forward(futures::sink::drain()).await;

        drop(close_rx);
        let _ = close_tx.send(());

        // Wait for all tasks to complete.
        tracing::debug!(
            "waiting for {} tasks to finish",
            close_tx.receiver_count()
        );
        close_tx.closed().await;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use http_body_util::Empty;
    use hyper::client::conn::http2;
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_http2_cleartext_request() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let server = Server::new(listener, crate::service::router())
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            });

        let server_task = tokio::spawn(async move {
            server.serve().await.unwrap();
        });

        let stream = TokioIo::new(TcpStream::connect(addr).await.unwrap());
        let (mut sender, conn) = http2::handshake(TokioExecutor::new(), stream)
            .await
            .expect("HTTP/2 handshake failed");

        tokio::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("HTTP/2 connection error: {err}");
            }
        });

        let req = hyper::Request::builder()
            .uri(format!("http://{addr}/healthz"))
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
