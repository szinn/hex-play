use crate::{
    Error,
    models::{NewUser, User},
    services::Transaction,
};

#[async_trait::async_trait]
pub trait UserService: Send + Sync {
    async fn add_user(&self, transaction: &dyn Transaction, user: NewUser) -> Result<User, Error>;
    async fn update_user(&self, transaction: &dyn Transaction, user: User) -> Result<User, Error>;
    async fn delete_user(&self, transaction: &dyn Transaction, user: User) -> Result<User, Error>;
    async fn list_users(&self, transaction: &dyn Transaction, start_id: Option<i64>, page_size: Option<u64>) -> Result<Vec<User>, Error>;
    async fn find_by_id(&self, transaction: &dyn Transaction, id: i64) -> Result<Option<User>, Error>;
    async fn find_by_email(&self, transaction: &dyn Transaction, email: &str) -> Result<Option<User>, Error>;
}
