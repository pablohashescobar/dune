use crate::api_error::ApiError;
use crate::db;
use crate::schema::user;
use argon2::Config;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Insertable, Clone)]
#[table_name = "user"]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub username: String,
    pub data_version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, AsChangeset)]
#[table_name = "user"]
pub struct UserMessage {
    pub email: String,
    pub password: String,
    pub name: String,
    pub username: String,
}
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    email: String,
    name: String,
    username: String,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub username: String,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        AuthUser {
            id: claims.sub,
            email: claims.email,
            name: claims.name,
            username: claims.username,
        }
    }
}

impl From<User> for AuthUser {
    fn from(user: User) -> Self {
        AuthUser {
            id: user.id,
            email: user.email,
            name: user.name,
            username: user.username,
        }
    }
}

impl User {
    pub fn find_all() -> Result<Vec<Self>, ApiError> {
        let conn = db::connection()?;

        let users = user::table.load::<User>(&conn)?;

        Ok(users)
    }

    pub fn find(id: Uuid) -> Result<Self, ApiError> {
        let conn = db::connection()?;

        let user = user::table.filter(user::id.eq(id)).first(&conn)?;

        Ok(user)
    }

    pub fn find_by_email(email: String) -> Result<Self, ApiError> {
        let conn = db::connection()?;

        let user = user::table.filter(user::email.eq(email)).first(&conn)?;

        Ok(user)
    }

    pub fn find_by_email_or_username(email: String, username: String) -> Result<Self, ApiError> {
        let conn = db::connection()?;

        let user = user::table
            .filter(user::email.eq(email))
            .or_filter(user::username.eq(username))
            .first(&conn)?;

        Ok(user)
    }

    pub fn create(user: UserMessage) -> Result<Self, ApiError> {
        let conn = db::connection()?;

        let mut user = User::from(user);
        user.hash_password()?;

        let user = diesel::insert_into(user::table)
            .values(user)
            .get_result(&conn)?;

        Ok(user)
    }

    pub fn update(id: Uuid, user: UserMessage) -> Result<Self, ApiError> {
        let conn = db::connection()?;

        let user = diesel::update(user::table)
            .filter(user::id.eq(id))
            .set(user)
            .get_result(&conn)?;

        Ok(user)
    }

    pub fn delete(id: Uuid) -> Result<usize, ApiError> {
        let conn = db::connection()?;

        let res = diesel::delete(user::table.filter(user::id.eq(id))).execute(&conn)?;

        Ok(res)
    }

    pub fn hash_password(&mut self) -> Result<(), ApiError> {
        let salt: [u8; 32] = rand::thread_rng().gen();
        let config = Config::default();

        self.password = argon2::hash_encoded(self.password.as_bytes(), &salt, &config)
            .map_err(|e| ApiError::new(500, format!("Failed to hash password: {}", e)))?;

        Ok(())
    }
    pub fn verify_password(&self, password: &[u8]) -> Result<bool, ApiError> {
        argon2::verify_encoded(&self.password, password)
            .map_err(|e| ApiError::new(500, format!("Failed to verify password: {}", e)))
    }
}

impl From<UserMessage> for User {
    fn from(user: UserMessage) -> Self {
        User {
            id: Uuid::new_v4(),
            email: user.email,
            password: user.password,
            username: user.username,
            name: user.name,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
            data_version: 1,
        }
    }
}
