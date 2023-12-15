use crate::args::*;
use std::net::ToSocketAddrs;
use tokio::net::TcpListener;
use tokio::{runtime, signal};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod args;
mod handler;
mod headers;
mod http;
mod num_cpus;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "httpbox=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Cli::parse();

    if let Some(generator) = args.completions {
        args.print_completions(generator);
        return Ok(());
    }

    let threads = args.threads.unwrap_or_else(num_cpus::num_cpus);
    let addr = (args.host.clone(), args.port)
        .to_socket_addrs()
        .ok()
        .and_then(|iter| iter.last())
        .unwrap_or_else(|| {
            panic!(
                "Invalid listening address: {}:{}",
                args.host.clone(),
                args.port
            )
        });

    let runtime = runtime::Builder::new_multi_thread()
        .worker_threads(threads.get())
        .enable_io()
        .build()?;

    tracing::info!("Listening on {} with {} threads", addr, threads);
    runtime.block_on(async {
        let service = ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .service(service::router());

        let listener = TcpListener::bind(addr).await.unwrap();

        let server = server::Server::new(listener, service)
            .with_graceful_shutdown(shutdown_signal());

        let _ = server.serve().await;
    });
    Ok(())
}
