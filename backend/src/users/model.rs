use chrono::{NaiveDateTime, Utc};
use serde::{Deserializable, Serializable};
use uuid::Uuid;

#[derive(Deserializable, Serializable)]
pub struct user {
    pub id: Uuid,
    pub name: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub data_version: i32,
    pub created_at: NativeDateTime,
    pub updated_at: Option<NaiveDateTime>,
}
