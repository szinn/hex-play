#[cfg(not(feature = "server"))]
fn main() {
    hex_play_frontend::web::launch_web_frontend();
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use hex_play::{
        commands::{CommandLine, Commands, run_server_command},
        config::Config,
        logging::init_logging,
    };

    let cli: CommandLine = clap::Parser::parse();
    let config = Config::load().context("Cannot load configuration")?;

    match cli.command {
        Commands::Server => {
            init_logging()?;
            run_server_command(&config).await.context("Couldn't start server")?
        }
        Commands::Status { question } => {
            let answer = hex_play_api::grpc::system::api::status(question).await?;
            println!("Status: {}", answer);
        }
        Commands::AddUser { name, email, age } => {
            let user = hex_play_api::grpc::user::api::create(name, email, age).await?;
            println!("Added user: {:?}", user);
        }
        Commands::DeleteUser { id } => {
            let user = hex_play_api::grpc::user::api::delete(id).await?;
            println!("Deleted user: {:?}", user);
        }
        Commands::UpdateUser { id, name, email, age } => {
            let user = hex_play_api::grpc::user::api::update(id, name, email, age).await?;
            println!("Updated user: {:?}", user);
        }
        Commands::GetUsers {} => {
            let users = hex_play_api::grpc::user::api::list(None, None).await?;
            println!("Users: {:?}", users);
        }
    }
    Ok(())
}
