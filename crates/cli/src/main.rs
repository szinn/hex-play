use anyhow::{Context, Result};
use hex_play::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load().context("Cannot load configuration")?;
    println!("{:?}", config);

    Ok(())
}
