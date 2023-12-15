use clap::CommandFactory;
pub use clap::Parser;
use clap_complete::{generate, Generator, Shell};
use std::io;
use std::num::NonZeroUsize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    #[arg(long, exclusive = true)]
    pub completions: Option<Shell>,

    #[arg(
        short,
        long,
        env,
        default_value = "0.0.0.0",
        help = "Host address to listen on"
    )]
    pub host: String,

    #[arg(
        short,
        long,
        env,
        default_value_t = 3000,
        help = "Port to listen on"
    )]
    pub port: u16,

    #[arg(long, env, help = "Number of threads to process requests")]
    pub threads: Option<NonZeroUsize>,

    #[arg(long, action = clap::ArgAction::Help, help = "Print help information")]
    pub help: (),
}

impl Cli {
    pub fn print_completions<G: Generator>(&self, gen: G) {
        let mut cmd = Self::command();
        let bin_name = cmd.get_name().to_string();
        generate(gen, &mut cmd, bin_name, &mut io::stdout());
    }
}
