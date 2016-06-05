extern crate docopt;
#[macro_use(itry, iexpect)]
extern crate iron;
extern crate num_cpus;
extern crate rustc_serialize;

use docopt::Docopt;
use iron::Protocol;

mod app;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
Httpbox.

Usage:
  httpbox [--host=<host>] [--port=<port>] [--threads=<threads>]
  httpbox (-h | --help)
  httpbox --version

Options:
  -h --help             Show this screen.
  --version             Show version.
  --host=<host>         Host address to listen on [default: localhost]
  --port=<port>         Port to listen on [default: 3000]
  --threads=<threads>   Number of threads to process requests
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_host: String,
    flag_port: u16,
    flag_version: bool,
    flag_threads: usize,
}


fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("httpbox v{}", VERSION);
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
    app::app()
        .listen_with((&args.flag_host[..], args.flag_port),
                     threads,
                     Protocol::Http,
                     None)
        .unwrap();
}
