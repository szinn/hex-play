use uuid::Uuid;

use crate::{
    Error,
    models::{Age, user_info::UserInfo},
    repositories::Transaction,
};

#[async_trait::async_trait]
pub trait UserInfoRepository: Send + Sync {
    async fn add_info(&self, transaction: &dyn Transaction, user_token: Uuid, age: Age) -> Result<UserInfo, Error>;
    async fn update_info(&self, transaction: &dyn Transaction, user_token: Uuid, age: Age) -> Result<UserInfo, Error>;
    async fn find_by_token(&self, transaction: &dyn Transaction, user_token: Uuid) -> Result<Option<UserInfo>, Error>;
}
