use std::collections::HashMap;

use hex_play_core::Error;
use rust_embed::EmbeddedFile;
use sea_orm::{DatabaseConnection, TransactionTrait};
use sea_orm_migration::prelude::*;
use serde::Deserialize;

use crate::error::handle_dberr;

struct MigrationInfo {
    name: String,
    up: String,
    down: String,
}

impl MigrationName for MigrationInfo {
    fn name(&self) -> &str {
        &self.name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for MigrationInfo {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(&self.up).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(&self.down).await?;

        Ok(())
    }
}

#[derive(rust_embed::RustEmbed)]
#[folder = "migrations/"]
struct Asset;

struct Migrator;

#[tracing::instrument(level = "trace", skip(database))]
pub async fn apply_migrations(database: &DatabaseConnection) -> Result<(), Error> {
    Migrator::up(database, None).await.map_err(handle_dberr)?;

    Ok(())
}

#[tracing::instrument(level = "trace", skip(database))]
pub async fn reset_migrations(database: &DatabaseConnection) -> Result<(), Error> {
    Migrator::reset(database).await.map_err(handle_dberr)?;

    Ok(())
}

#[derive(Deserialize)]
struct DatabaseConfiguration {
    tables: Vec<String>,
}

#[tracing::instrument(level = "trace", skip(database))]
pub async fn reset_tables(database: &DatabaseConnection) -> Result<(), Error> {
    crate::migration::apply_migrations(database).await?;

    let configuration = Asset::get("configuration.json").ok_or_else(|| Error::Message("Can't read configuration.json".into()))?;
    let configuration: DatabaseConfiguration =
        serde_json::from_str(&contents(configuration)).map_err(|_| Error::Message("Can't load configuration.json".into()))?;

    let tx = TransactionTrait::begin(database).await.map_err(handle_dberr)?;
    for table in configuration.tables {
        tracing::debug!("Deleting data from table {}", table);
        sea_orm::ConnectionTrait::execute_unprepared(&tx, &format!("delete from {};", table))
            .await
            .map_err(handle_dberr)?;
    }
    tx.commit().await.map_err(handle_dberr)?;

    Ok(())
}

pub async fn run_migration_cli() {
    cli::run_cli(Migrator).await
}

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut up_migrations: HashMap<String, String> = HashMap::new();
        let mut down_migrations: HashMap<String, String> = HashMap::new();

        for file in Asset::iter() {
            let file_data = Asset::get(file.as_ref()).unwrap();
            let contents = contents(file_data);

            if file.starts_with("up/") {
                up_migrations.insert(file.strip_prefix("up/").unwrap().into(), contents);
            } else if file.starts_with("down/") {
                down_migrations.insert(file.strip_prefix("down/").unwrap().into(), contents);
            }
        }

        let mut migration_names: Vec<&String> = up_migrations.keys().collect();
        migration_names.sort();

        let migrations: Vec<Box<dyn MigrationTrait + 'static>> = migration_names
            .iter()
            .map(|name| {
                let name = name.to_string();

                let info = MigrationInfo {
                    up: up_migrations.get(&name).unwrap().to_string(),
                    down: down_migrations.get(&name).unwrap().to_string(),
                    name,
                };

                Box::new(info) as Box<dyn MigrationTrait>
            })
            .collect();

        migrations
    }
}

fn contents(file_data: EmbeddedFile) -> String {
    let contents = file_data.data.as_ref();
    let contents = Vec::from(contents);

    String::from_utf8(contents).unwrap()
}
