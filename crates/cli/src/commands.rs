mod server;

pub use server::*;

#[derive(Debug, clap::Parser)]
#[command(
    name = "HexPlay",
    help_template = r#"
{before-help}{name} {version} - {about}

{usage-heading} {usage}

{all-args}{after-help}

AUTHORS:
    {author}
"#,
    version,
    author
)]
#[command(about, long_about = None)]
#[command(propagate_version = true, arg_required_else_help = true)]
pub struct CommandLine {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    #[command(about = "Start server", display_order = 10)]
    Server,

    #[command(about = "Server status check", display_order = 20)]
    Status { question: String },

    #[command(about = "Add user", display_order = 30)]
    AddUser { name: String, email: String, age: i16 },

    #[command(about = "Delete user", display_order = 31)]
    DeleteUser { id: i64 },

    #[command(about = "Update user", display_order = 32)]
    UpdateUser {
        id: i64,
        #[arg(value_name = "name")]
        name: Option<String>,
        #[arg(value_name = "email")]
        email: Option<String>,
        #[arg(value_name = "age")]
        age: Option<i16>,
    },

    #[command(about = "Get users", display_order = 33)]
    GetUsers {},
}
