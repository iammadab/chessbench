use clap::Parser;
use std::path::PathBuf;
use std::{fs, process};

mod api;
mod config;
mod domain;
mod engine;
mod match_runner;
mod server;
mod uci;

#[derive(Debug, Parser)]
#[command(name = "chessbench", version, about = "UCI engine vs engine server")]
struct Cli {
    #[arg(long, default_value = "0.0.0.0:8080")]
    bind: String,
    #[arg(long, value_name = "PATH")]
    config: PathBuf,
}

#[tokio::main]
async fn main() {
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

    let engines = match uci::discover_engines(&config.engine).await {
        Ok(engines) => engines,
        Err(err) => {
            eprintln!("engine discovery failed: {err}");
            process::exit(1);
        }
    };

    if engines.is_empty() {
        eprintln!("no engines available after discovery");
        process::exit(1);
    }

    let app = server::build_router(engines);

    let listener = match tokio::net::TcpListener::bind(&cli.bind).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("failed to bind {}: {err}", cli.bind);
            process::exit(1);
        }
    };

    if let Err(err) = axum::serve(listener, app).await {
        eprintln!("server error: {err}");
        process::exit(1);
    }
}
