use clap::Parser;
use std::path::PathBuf;
use std::{fs, process};

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
    let cli = Cli::parse();

    let config_text = match fs::read_to_string(&cli.config) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("failed to read config {}: {err}", cli.config.display());
            process::exit(1);
        }
    };

    let config = match config::EngineConfigFile::from_str(&config_text) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("invalid config format: {err}");
            process::exit(1);
        }
    };

    if let Err(err) = config.validate() {
        eprintln!("invalid config contents: {err:?}");
        process::exit(1);
    }
}
