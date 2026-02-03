use chrono::{DateTime, Utc};
use derive_builder::Builder;
use uuid::Uuid;

use super::newtypes::Age;

#[derive(Debug, Clone, Builder)]
pub struct UserInfo {
    #[builder(default = "0")]
    pub id: i64,
    #[builder(default = "Uuid::nil()")]
    pub user_token: Uuid,
    #[builder(default)]
    pub age: Age,

    #[builder(default = "Utc::now()")]
    pub created_at: DateTime<Utc>,
    #[builder(default = "Utc::now()")]
    pub updated_at: DateTime<Utc>,
}

impl Default for UserInfo {
    fn default() -> Self {
        Self {
            id: 0,
            user_token: Uuid::nil(),
            age: Age::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
