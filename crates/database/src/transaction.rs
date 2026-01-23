use sea_orm::DatabaseTransaction;

use crate::error::Error;

#[async_trait::async_trait]
pub trait Transaction {
    async fn commit(self: Box<Self>) -> Result<(), Error>;
    async fn rollback(self: Box<Self>) -> Result<(), Error>;
}

pub(crate) struct TransactionImpl {
    transaction: DatabaseTransaction,
}

impl TransactionImpl {
    pub(crate) fn new(transaction: DatabaseTransaction) -> Self {
        Self { transaction }
    }
}

#[async_trait::async_trait]
impl Transaction for TransactionImpl {
    async fn commit(self: Box<Self>) -> Result<(), Error> {
        self.transaction.commit().await?;
        Ok(())
    }
    async fn rollback(self: Box<Self>) -> Result<(), Error> {
        self.transaction.rollback().await?;
        Ok(())
    }
}
