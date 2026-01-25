use anyhow::{Context, Result};
use hex_play::{
    commands::{CommandLine, Commands, run_server_command},
    config::Config,
    logging::init_logging,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli: CommandLine = clap::Parser::parse();
    let config = Config::load().context("Cannot load configuration")?;

    match cli.command {
        Commands::Server => {
            init_logging()?;

            run_server_command(&config).await.context("Couldn't start server")?
        }
    }
    Ok(())
}
