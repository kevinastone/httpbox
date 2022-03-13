use clap::{App, IntoApp, Parser};
use clap_complete::{generate, Generator, Shell};
use hyper::Server;
use std::io;
use std::net::ToSocketAddrs;
use std::num::NonZeroUsize;
use tokio::runtime;
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
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(
        short,
        long,
        env,
        default_value = "0.0.0.0",
        help = "Host address to listen on"
    )]
    host: String,

    #[clap(
        short,
        long,
        env,
        default_value_t = 3000,
        help = "Port to listen on"
    )]
    port: u16,

    #[clap(long, env, help = "Number of threads to process requests")]
    threads: Option<NonZeroUsize>,

    #[clap(long)]
    completions: Option<Shell>,
}

fn print_completions<G: Generator>(gen: G, app: &mut App) {
    generate(gen, app, app.get_name().to_string(), &mut io::stdout());
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c().await.unwrap()
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
        let mut app = Cli::into_app();
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
        let router = service::router();

        let server = Server::bind(&addr)
            .serve(router)
            .with_graceful_shutdown(shutdown_signal());

        server.await?;
        Ok(())
    })
}
