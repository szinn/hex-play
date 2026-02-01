use chrono::Utc;
use hex_play_core::{
    Error,
    models::user_info::UserInfo,
    repositories::{Transaction, UserInfoRepository},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, prelude::Uuid};

use crate::{
    entities::{prelude, user_info},
    error::handle_dberr,
    transaction::TransactionImpl,
};

impl From<user_info::Model> for UserInfo {
    fn from(model: user_info::Model) -> Self {
        Self {
            id: model.id,
            user_token: model.user_token,
            age: model.age,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }
}

pub struct UserInfoRepositoryAdapter;

impl UserInfoRepositoryAdapter {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl UserInfoRepository for UserInfoRepositoryAdapter {
    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn add_info(&self, transaction: &dyn Transaction, user_token: Uuid, age: i16) -> Result<UserInfo, Error> {
        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        let model = user_info::ActiveModel {
            user_token: Set(user_token),
            age: Set(age),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        let model = model.insert(transaction).await.map_err(handle_dberr)?;

        Ok(model.into())
    }

    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn update_info(&self, transaction: &dyn Transaction, user_token: Uuid, age: i16) -> Result<UserInfo, Error> {
        let tx = TransactionImpl::get_db_transaction(transaction)?;

        // Find existing record by user_token
        let existing = prelude::UserInfo::find()
            .filter(user_info::Column::UserToken.eq(user_token))
            .one(tx)
            .await
            .map_err(handle_dberr)?;

        match existing {
            Some(existing) => {
                // Update existing record
                let mut updater: user_info::ActiveModel = existing.into();
                updater.age = Set(age);
                if updater.is_changed() {
                    updater.updated_at = Set(Utc::now().into());
                }
                Ok(updater.update(tx).await.map_err(handle_dberr)?.into())
            }
            None => self.add_info(transaction, user_token, age).await,
        }
    }

    #[tracing::instrument(level = "trace", skip(self, transaction))]
    async fn find_by_token(&self, transaction: &dyn Transaction, user_token: Uuid) -> Result<Option<UserInfo>, Error> {
        let transaction = TransactionImpl::get_db_transaction(transaction)?;

        Ok(prelude::UserInfo::find()
            .filter(user_info::Column::UserToken.eq(user_token))
            .one(transaction)
            .await
            .map_err(handle_dberr)?
            .map(Into::into))
    }
}
