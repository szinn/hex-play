use anyhow::Context;
use anyhow::Result;
use hex_play::args::Args;
use hex_play::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = clap::Parser::parse();
    let config = Config::load().context("Cannot load configuration")?;
    println!("{:?}", config);
    println!("{:?}", args);

    Ok(())
}
