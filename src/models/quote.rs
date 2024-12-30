use serde::{Deserialize, Serialize};
use sqlx::types::{Uuid, chrono::{DateTime, Local}};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Quote {
    pub id: Uuid,
    pub author: String,
    pub quote: String,
    pub created_at: DateTime<Local>,
    pub version: i32,
}
