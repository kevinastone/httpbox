#[macro_use]
mod macros;

use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version,
};
use clap::{value_t, value_t_or_exit, App, Arg, Error, ErrorKind, Shell};
use futures::prelude::*;
use hyper::server::conn::AddrStream;
use hyper::service::make_service_fn;
use hyper::Server;
use pretty_env_logger;
use std::convert::Infallible;
use std::io;
use std::net::ToSocketAddrs;
use tokio::runtime;

mod handler;
mod headers;
mod http;
mod option;
mod path;
mod random;
mod router;
mod service;

#[cfg(test)]
mod test;

fn cli<'a, 'b>() -> App<'a, 'b> {
    app_from_crate!()
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .takes_value(true)
                .default_value("0.0.0.0")
                .env("HOSTNAME")
                .help("Host address to listen on"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .takes_value(true)
                .default_value("3000")
                .env("PORT")
                .help("Port to listen on"),
        )
        .arg(
            Arg::with_name("threads")
                .long("threads")
                .value_name("THREADS")
                .takes_value(true)
                .help("Number of threads to process requests"),
        )
        .arg(
            Arg::with_name("completions")
                .long("completions")
                .takes_value(true)
                .value_name("SHELL")
                .hidden(true)
                .possible_values(&Shell::variants()),
        )
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c().await.unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let matches = cli().get_matches();

    if let Some(shell) = matches.value_of("completions") {
        cli().gen_completions_to(
            crate_name!(),
            shell.parse::<Shell>()?,
            &mut io::stdout(),
        );
        return Ok(());
    }

    let host = matches.value_of("host").expect("Invalid host");
    let port = value_t_or_exit!(matches.value_of("port"), u16);
    let threads = match value_t!(matches.value_of("threads"), usize) {
        Ok(val) => val,
        Err(Error {
            kind: ErrorKind::ArgumentNotFound,
            ..
        }) => ::num_cpus::get(),
        Err(e) => e.exit(),
    };

    let addr = (host, port)
        .to_socket_addrs()
        .ok()
        .and_then(|iter| iter.last())
        .unwrap_or_else(|| {
            panic!("Invalid listening address: {}:{}", host, port)
        });

    let mut runtime = runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .core_threads(threads)
        .build()?;

    println!("Listening on {} with {} threads", addr, threads);
    runtime.block_on(async {
        let router = service::router();

        let server = Server::bind(&addr).serve(make_service_fn(
            move |conn: &AddrStream| {
                future::ok::<_, Infallible>(
                    (&router).service(Some(conn.remote_addr())),
                )
            },
        ));

        let graceful = server.with_graceful_shutdown(shutdown_signal());

        graceful.await?;
        Ok(())
    })
}
