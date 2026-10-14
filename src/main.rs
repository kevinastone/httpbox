use crate::args::*;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::{runtime, signal};
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod args;
mod handler;
mod headers;
mod http;
mod random;
mod router;
mod server;
mod service;

#[cfg(test)]
mod test;

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn run_server(addr: SocketAddr) -> std::io::Result<()> {
    let service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .service(service::router());

    let listener = TcpListener::bind(addr).await?;

    let server = server::Server::new(listener, service)
        .with_graceful_shutdown(shutdown_signal());

    server.serve().await
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "httpbox=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Cli::parse();

    if let Some(generator) = args.completions {
        args.print_completions(generator);
        return Ok(());
    }

    let threads = args.threads;
    let addr = (args.host.as_str(), args.port)
        .to_socket_addrs()?
        .next_back()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                format!("Invalid listening address: {}:{}", args.host, args.port),
            )
        })?;

    let runtime = runtime::Builder::new_multi_thread()
        .worker_threads(threads.get())
        .enable_io()
        .enable_time()
        .build()?;

    tracing::info!("Listening on {} with {} threads", addr, threads);
    runtime.block_on(run_server(addr))?;
    Ok(())
}
