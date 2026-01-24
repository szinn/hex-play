use hex_play_core::Error;
use hex_play_core::Repository;
use hex_play_core::Transaction;
use sea_orm::AccessMode;
use sea_orm::DatabaseConnection;
use sea_orm::TransactionTrait;

use crate::TransactionImpl;
use crate::error::handle_dberr;

#[derive(Clone)]
pub(crate) struct RepositoryImpl {
    database: DatabaseConnection,
}

impl RepositoryImpl {
    pub(crate) fn new(database_connection: DatabaseConnection) -> Self {
        Self { database: database_connection }
    }
}

#[async_trait::async_trait]
impl Repository for RepositoryImpl {
    async fn begin(&self) -> Result<Box<dyn Transaction>, Error> {
        let transaction = self.database.begin().await.map_err(handle_dberr)?;
        Ok(Box::new(TransactionImpl::new(transaction)))
    }

    async fn begin_read_only(&self) -> Result<Box<dyn Transaction>, Error> {
        let transaction = self.database.begin_with_config(None, Some(AccessMode::ReadOnly)).await.map_err(handle_dberr)?;
        Ok(Box::new(TransactionImpl::new(transaction)))
    }

    async fn close(&self) -> Result<(), Error> {
        self.database.clone().close().await.map_err(handle_dberr)?;

        Ok(())
    }
}
