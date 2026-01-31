use uuid::Uuid;

use crate::{Error, models::user_info::UserInfo, services::Transaction};

#[async_trait::async_trait]
pub trait UserInfoService: Send + Sync {
    async fn add_info(&self, transaction: &dyn Transaction, user_token: Uuid, age: i16) -> Result<UserInfo, Error>;
    async fn update_info(&self, transaction: &dyn Transaction, user_token: Uuid, age: i16) -> Result<UserInfo, Error>;
    async fn find_by_token(&self, transaction: &dyn Transaction, user_token: Uuid) -> Result<Option<UserInfo>, Error>;
    async fn find_by_tokens(&self, transaction: &dyn Transaction, user_tokens: &[Uuid]) -> Result<Vec<UserInfo>, Error>;
}
