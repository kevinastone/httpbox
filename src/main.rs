use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use futures::prelude::*;
use hyper::server::conn::http1;
use hyper::Request as HTTPRequest;
use hyper_util::rt::TokioIo;
use std::io;
use std::net::ToSocketAddrs;
use std::num::NonZeroUsize;
use tokio::net::TcpListener;
use tokio::{runtime, signal, sync::watch};
use tokio_stream::wrappers::TcpListenerStream;
use tower::Service;
use tower::ServiceBuilder;
use tower::ServiceExt;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handler;
mod headers;
mod http;
mod num_cpus;
mod random;
mod router;
mod service;

#[cfg(test)]
mod test;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, disable_help_flag = true)]
struct Cli {
    #[arg(
        short,
        long,
        env,
        default_value = "0.0.0.0",
        help = "Host address to listen on"
    )]
    host: String,

    #[arg(
        short,
        long,
        env,
        default_value_t = 3000,
        help = "Port to listen on"
    )]
    port: u16,

    #[arg(long, env, help = "Number of threads to process requests")]
    threads: Option<NonZeroUsize>,

    #[arg(long)]
    completions: Option<Shell>,

    #[arg(long, action = clap::ArgAction::Help, help = "Print help information")]
    help: (),
}

fn print_completions<G: Generator>(gen: G, app: &mut Command) {
    generate(gen, app, app.get_name().to_string(), &mut io::stdout());
}

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
        let mut app = Cli::command();
        print_completions(generator, &mut app);
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
        let (close_tx, close_rx) = watch::channel(());

        let conn_stream =
            TcpListenerStream::new(listener).take_until(shutdown_signal());

        let conn_stream = conn_stream.and_then(|stream| async {
            let addr = stream.peer_addr()?;
            let stream = TokioIo::new(stream);

            // Inject the client addr into the request
            let tower_service = service.clone().map_request(
                move |mut req: HTTPRequest<_>| {
                    req.extensions_mut().insert(addr);
                    req
                },
            );

            let close_rx = close_rx.clone();

            runtime.spawn(async move {
                let hyper_service = hyper::service::service_fn(
                    move |request: hyper::Request<_>| {
                        tower_service.clone().call(request)
                    },
                );

                let conn = http1::Builder::new()
                    .serve_connection(stream, hyper_service).with_upgrades();

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
                        _ = shutdown_signal() => {
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
    });
    Ok(())
}
