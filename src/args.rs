use crate::num_cpus;
use clap::CommandFactory;
pub use clap::Parser;
use clap_complete::{Generator, Shell, generate};
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

    #[arg(long, env, default_value_t = num_cpus::num_cpus(), help = "Number of threads to process requests")]
    pub threads: NonZeroUsize,

    #[arg(long, action = clap::ArgAction::Help, help = "Print help information")]
    pub help: (),
}

impl Cli {
    pub fn print_completions<G: Generator>(&self, generator: G) {
        let mut cmd = Self::command();
        let bin_name = cmd.get_name().to_string();
        generate(generator, &mut cmd, bin_name, &mut io::stdout());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }

    #[test]
    fn test_args_defaults() {
        let args = Cli::parse_from(vec!["httpbox"]);
        assert_eq!(args.host, "0.0.0.0");
        assert_eq!(args.port, 3000u16);
        assert_eq!(args.threads, num_cpus::num_cpus());
    }
}
