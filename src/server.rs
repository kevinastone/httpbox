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
