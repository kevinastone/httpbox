#[macro_use]
extern crate clap;
#[macro_use]
extern crate horrorshow;
#[macro_use(itry, iexpect)]
extern crate iron;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
extern crate rustc_serialize;

use clap::{App, Arg, Shell, Error, ErrorKind};
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

    let host = matches.value_of("host").expect("Invalid host");
    let port = value_t_or_exit!(matches.value_of("port"), u16);
    let threads = match value_t!(matches.value_of("threads"), usize) {
        Ok(val) => val,
        Err(Error{kind: ErrorKind::ArgumentNotFound, ..}) => 8 * ::num_cpus::get(),
        Err(e) => e.exit(),
    };
    println!("Listening on {}:{} with {} threads",
             host,
             port,
             threads,
    );
    Iron::new(app::app())
        .listen_with((host, port), threads, Protocol::Http, None)
        .unwrap();
}
