#[derive(Debug, clap::Parser)]
#[clap(name = "hex-play", about = "A CLI for rust experimentation")]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Subcommands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    /// Start server.
    Server,
}
