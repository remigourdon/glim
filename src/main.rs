mod cli;
mod config;
mod repository;

use cli::CLI;

use anyhow::Result;
use structopt::StructOpt;

fn main() -> Result<()> {
    let mut cli = CLI::from_args();
    cli.run()
}
