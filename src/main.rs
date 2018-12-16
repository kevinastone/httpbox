use clap::{value_t, value_t_or_exit, App, Arg, Error, ErrorKind, Shell};
use std::io;
use std::net::ToSocketAddrs;

mod app;
mod headers;
mod router;

#[cfg(test)]
mod test;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

fn cli() -> App<'static, 'static> {
    App::new(NAME)
        .version(VERSION)
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .takes_value(true)
                .default_value("localhost")
                .help("Host address to listen on"),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .takes_value(true)
                .default_value("3000")
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

fn main() {
    let matches = cli().get_matches();

    if let Some(shell) = matches.value_of("completions") {
        cli().gen_completions_to(
            NAME,
            shell.parse::<Shell>().unwrap(),
            &mut io::stdout(),
        );
        return;
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
        .expect(&format!("Invalid listening address: {}:{}", host, port)[..]);
    println!("Listening on {}:{} with {} threads", host, port, threads,);
    gotham::start_with_num_threads(addr, app::app(), threads)
}
