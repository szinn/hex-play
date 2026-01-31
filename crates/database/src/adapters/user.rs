use chrono::Utc;
use hex_play_core::{
    Error, RepositoryError,
    models::{NewUser, User},
    services::{Transaction, UserService},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect};

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
    async fn add_user(&self, transaction: &dyn Transaction, user: NewUser) -> Result<User, Error> {
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
        if user.id < 0 {
            return Err(Error::InvalidId(user.id));
        }

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
    async fn delete_user(&self, transaction: &dyn Transaction, id: i64) -> Result<User, Error> {
        if id < 0 {
            return Err(Error::InvalidId(id));
        }

        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        let existing = prelude::Users::find_by_id(id).one(transaction).await.map_err(handle_dberr)?;
        let Some(existing) = existing else {
            return Err(Error::RepositoryError(RepositoryError::NotFound));
        };

        let user: User = existing.clone().into();
        existing.delete(transaction).await.map_err(handle_dberr)?;

        Ok(user)
    }

    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn list_users(&self, transaction: &dyn Transaction, start_id: Option<i64>, page_size: Option<u64>) -> Result<Vec<User>, Error> {
        const DEFAULT_PAGE_SIZE: u64 = 50;
        const MAX_PAGE_SIZE: u64 = 50;

        if let Some(start_id) = start_id {
            if start_id < 0 {
                return Err(Error::InvalidId(start_id));
            }
        }

        if let Some(page_size) = page_size {
            if page_size < 1 {
                return Err(Error::InvalidPageSize(page_size));
            }
        }

        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        let mut query = prelude::Users::find().order_by_asc(users::Column::Id);

        if let Some(start_id) = start_id {
            query = query.filter(users::Column::Id.gte(start_id));
        }

        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE);
        query = query.limit(page_size);

        let users = query.all(transaction).await.map_err(handle_dberr)?;

        Ok(users.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn find_by_id(&self, transaction: &dyn Transaction, id: i64) -> Result<Option<User>, Error> {
        if id < 0 {
            return Err(Error::InvalidId(id));
        }

        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        match prelude::Users::find_by_id(id).one(transaction).await.map_err(handle_dberr)? {
            Some(model) => Ok(Some(model.into())),
            None => Ok(None),
        }
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
