use hex_play_database::migration::run_migration_cli;

#[tokio::main]
async fn main() {
    run_migration_cli().await;
}
