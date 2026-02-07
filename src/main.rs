use clap::Parser;
use std::path::PathBuf;

mod api;
mod config;
mod domain;
mod match_runner;
mod uci;

#[derive(Debug, Parser)]
#[command(name = "chessbench", version, about = "UCI engine vs engine server")]
struct Cli {
    #[arg(long, default_value = "0.0.0.0:8080")]
    bind: String,
    #[arg(long, value_name = "PATH")]
    config: PathBuf,
}

fn main() {
    let _cli = Cli::parse();
}
