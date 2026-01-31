use chrono::{DateTime, Utc};
use derive_builder::Builder;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Builder)]
pub struct UserInfo {
    #[builder(default = "0")]
    pub id: i64,
    #[builder(default = "Uuid::nil()")]
    pub user_token: Uuid,
    #[builder(default = "0")]
    pub age: i16,

    #[builder(default = "Utc::now()")]
    pub created_at: DateTime<Utc>,
    #[builder(default = "Utc::now()")]
    pub updated_at: DateTime<Utc>,
}
