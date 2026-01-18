use anyhow::Context;
use anyhow::Result;
use hex_play::commands::CommandLine;
use hex_play::commands::Commands;
use hex_play::commands::run_server_command;
use hex_play::config::Config;
use hex_play::logging::init_logging;

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
