use chrono::{DateTime, Utc};
use derive_builder::Builder;

#[derive(Debug, Default, Builder)]
pub struct User {
    #[builder(default = "0")]
    pub id: i64,
    #[builder(default = "0")]
    pub version: i64,
    pub name: String,
    pub email: String,
    #[builder(default = "Utc::now()")]
    pub created_at: DateTime<Utc>,
    #[builder(default = "Utc::now()")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default, Builder)]
pub struct NewUser {
    pub name: String,
    pub email: String,
}
