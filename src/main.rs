use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use std::io;
use std::net::ToSocketAddrs;
use std::num::NonZeroUsize;
use tokio::net::TcpListener;
use tokio::{runtime, signal};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

        let server = server::Server::new(listener, service)
            .with_graceful_shutdown(shutdown_signal());

        let _ = server.serve().await;
    });
    Ok(())
}
