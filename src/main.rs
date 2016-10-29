extern crate clap;
#[macro_use]
extern crate horrorshow;
#[macro_use(itry, iexpect)]
extern crate iron;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
extern crate rustc_serialize;

use clap::{App, Arg, Shell};
use iron::{Iron, Protocol};
use std::io;

mod app;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NAME: &'static str = env!("CARGO_PKG_NAME");

fn cli() -> App<'static, 'static> {
    App::new(NAME)
        .version(VERSION)
        .arg(Arg::with_name("host")
            .short("h")
            .long("host")
            .value_name("HOST")
            .takes_value(true)
            .default_value("localhost")
            .help("Host address to listen on"))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .takes_value(true)
            .default_value("3000")
            .help("Port to listen on"))
        .arg(Arg::with_name("threads")
            .long("threads")
            .value_name("THREADS")
            .takes_value(true)
            .help("Number of threads to process requests"))
        .arg(Arg::with_name("completions")
            .long("completions")
            .takes_value(true)
            .value_name("SHELL")
            .hidden(true)
            .possible_values(&Shell::variants()))
}

fn main() {
    let matches = cli().get_matches();

    if let Some(shell) = matches.value_of("completions") {
        cli().gen_completions_to(NAME, shell.parse::<Shell>().unwrap(), &mut io::stdout());
        return;
    }

    let host = matches.value_of("host").unwrap();
    let port = matches.value_of("port").and_then(|p| p.parse::<u16>().ok()).unwrap();
    let threads = matches.value_of("threads")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or_else(|| 8 * ::num_cpus::get());
    println!("Listening on {}:{} with {} threads",
             host,
             port,
             threads,
    );
    Iron::new(app::app())
        .listen_with((host, port), threads, Protocol::Http, None)
        .unwrap();
}
