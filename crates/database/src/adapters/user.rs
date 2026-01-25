use chrono::Utc;
use hex_play_core::{Error, RepositoryError, Transaction, User, UserService};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};

use crate::{
    entities::{prelude, users},
    error::handle_dberr,
    transaction::TransactionImpl,
};

impl From<users::Model> for User {
    fn from(model: users::Model) -> Self {
        Self {
            id: model.id,
            version: model.version,
            name: model.name,
            email: model.email,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }
}

pub struct UserServiceAdapter;

impl UserServiceAdapter {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl UserService for UserServiceAdapter {
    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn add_user(&self, transaction: &dyn Transaction, user: User) -> Result<User, Error> {
        let transaction = TransactionImpl::get_db_transaction(transaction)?;
        let model = users::ActiveModel {
            name: Set(user.name),
            email: Set(user.email),
            version: Set(0),
            ..Default::default()
        };

        let model = model.insert(transaction).await.map_err(handle_dberr)?;

        Ok(model.into())
    }

    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn update_user(&self, transaction: &dyn Transaction, user: User) -> Result<User, Error> {
        let transaction = TransactionImpl::get_db_transaction(transaction)?;
        let existing = prelude::Users::find_by_id(user.id).one(transaction).await.map_err(handle_dberr)?;
        if existing.is_none() {
            return Err(Error::RepositoryError(RepositoryError::NotFound));
        }
        let existing = existing.unwrap();
        if existing.version != user.version {
            return Err(Error::RepositoryError(RepositoryError::Conflict));
        }

        let mut updater: users::ActiveModel = existing.clone().into();
        if existing.name != user.name {
            updater.name = Set(user.name);
        }
        if existing.email != user.email {
            updater.email = Set(user.email);
        }

        let updated = updater.update(transaction).await.map_err(handle_dberr)?;

        Ok(updated.into())
    }

    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn find_by_email(&self, transaction: &dyn Transaction, email: &str) -> Result<Option<User>, Error> {
        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        match prelude::Users::find_by_email(email).one(transaction).await.map_err(handle_dberr)? {
            Some(model) => Ok(Some(model.into())),
            None => Ok(None),
        }
    }
}
