#[macro_use]
extern crate tracing;

use std::{error::Error, net::SocketAddr, str::FromStr};

use anyhow::{anyhow, bail, Result};
use config::Config;
use grpc::run_server;
use tokio::signal;
use tracing::Level;

mod cli {
    use clap::{arg, command, Command};

    pub fn cli_app<'a>() -> Command<'a> {
        command!()
            .arg(arg!(-c --config <FILE> "Sets a custom config file"))
            .allow_external_subcommands(true)
    }
}

struct ServerConfig {
    settings: Config,
}

impl ServerConfig {
    pub fn new(matches: clap::ArgMatches) -> Result<Self> {
        if let Some(config_file) = matches.value_of("config") {
            let settings = Config::builder()
                .add_source(config::File::with_name(config_file))
                .build()?;
            Ok(Self { settings })
        } else {
            bail!(anyhow!("Config file missing!"));
        }
    }

    pub fn get_string(&self, s: &str) -> Result<String> {
        Ok(self.settings.get_string(s)?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // enable backtraces
    std::env::set_var("RUST_BACKTRACE", "1");

    // CLI, config
    let matches = cli::cli_app().get_matches();
    let server_config = ServerConfig::new(matches)?;

    // tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::from_str(&server_config.get_string("log_level")?).unwrap())
        .init();

    // start server
    let server_addr = server_config.get_string("grpc_listen_address")?;
    info!("starting grpc server at {:?}", server_addr);
    run_server(SocketAddr::from_str(&server_addr).unwrap()).await?;

    tokio::select! {
        _ = signal::ctrl_c() => {},
    }

    Ok(())
}
