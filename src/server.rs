use futures::prelude::*;
use hyper::Request as HTTPRequest;
use hyper::body::{Body, Incoming};
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
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

        let conn_stream = conn_stream
            .take_until(self.shutdown_signal)
            .and_then(|stream| async {
                let addr = stream.peer_addr()?;
                let stream = TokioIo::new(stream);

                // Inject the client addr into the request
                let tower_service = service.clone().map_request(
                    move |mut req: HTTPRequest<_>| {
                        req.extensions_mut().insert(addr);
                        req
                    },
                );

                let mut close_rx = close_rx.clone();

                tokio::task::spawn(async move {
                    let hyper_service = hyper::service::service_fn(
                        move |request: HTTPRequest<_>| {
                            tower_service.clone().call(request)
                        },
                    );

                    let conn = http1::Builder::new()
                        .serve_connection(stream, hyper_service)
                        .with_upgrades();

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
            });

        // Run the listener stream to completion
        let _ = conn_stream.map(Ok).forward(futures::sink::drain()).await;

        drop(close_rx);

        // Wait for all tasks to complete.
        tracing::debug!(
            "waiting for {} tasks to finish",
            close_tx.receiver_count()
        );
        close_tx.closed().await;

        Ok(())
    }
}
