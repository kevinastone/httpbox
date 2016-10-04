extern crate docopt;
#[macro_use]
extern crate horrorshow;
#[macro_use(itry, iexpect)]
extern crate iron;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
extern crate rustc_serialize;

use docopt::Docopt;
use iron::{Iron, Protocol};

mod app;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NAME: &'static str = env!("CARGO_PKG_NAME");

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_host: String,
    flag_port: u16,
    flag_version: bool,
    flag_threads: usize,
}

macro_rules! usage {( $name:expr ) => (format!("
Usage:
  {name} [--host=<host>] [--port=<port>] [--threads=<threads>]
  {name} (-h | --help)
  {name} --version

Options:
  -h --help             Show this screen.
  --version             Show version.
  --host=<host>         Host address to listen on [default: localhost]
  --port=<port>         Port to listen on [default: 3000]
  --threads=<threads>   Number of threads to process requests
", name=$name))}

fn main() {
    let args: Args = Docopt::new(usage!(NAME))
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{name} v{version}", name = NAME, version = VERSION);
        return;
    }

    let threads: usize = if args.flag_threads > 0 {
        args.flag_threads
    } else {
        8 * ::num_cpus::get()
    };
    println!("Listening on {}:{} with {} threads",
             args.flag_host,
             args.flag_port,
             threads);
    Iron::new(app::app())
        .listen_with((&args.flag_host[..], args.flag_port),
                     threads,
                     Protocol::Http,
                     None)
        .unwrap();
}
