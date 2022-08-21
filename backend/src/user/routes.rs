use crate::api_error::ApiError;
use crate::user::{AuthUser, LoginRequest, User, UserMessage};
use actix_identity::Identity;
use actix_web::{delete, get, post, put, web, HttpResponse};

use serde_json::json;

use uuid::Uuid;

#[get("/users/")]
async fn find_all() -> Result<HttpResponse, ApiError> {
    let users = User::find_all()?;
    Ok(HttpResponse::Ok().json(users))
}

#[get("/users/{id}/")]
async fn find(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let user = User::find(id.into_inner())?;
    Ok(HttpResponse::Ok().json(user))
}

#[post("/register/")]
async fn register(user: web::Json<UserMessage>, id: Identity) -> Result<HttpResponse, ApiError> {
    let user = User::create(user.into_inner())?;

    let response = AuthUser {
        email: user.email,
        id: user.id,
        name: user.name,
        username: user.username,
    };

    let user_string = serde_json::to_string(&response).unwrap();

    id.remember(user_string);

    Ok(HttpResponse::Ok().json(response))
}

#[post("/login/")]
async fn sign_in(
    credentials: web::Json<LoginRequest>,
    id: Identity,
) -> Result<HttpResponse, ApiError> {
    let credentials = credentials.into_inner();

    let user = User::find_by_email_or_username(credentials.email, credentials.username)?;

    let is_valid = User::verify_password(&user, credentials.password.as_bytes())?;

    if is_valid {
        let response = AuthUser {
            email: user.email,
            id: user.id,
            name: user.name,
            username: user.username,
        };

        let user_string = serde_json::to_string(&response).unwrap();

        id.remember(user_string);

        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(ApiError::new(401, "Invalid Credentials".to_string()))
    }
}

#[put("/users/")]
async fn update(
    user: web::Json<UserMessage>,
    identity: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let user = User::update(identity.id, user.into_inner())?;
    Ok(HttpResponse::Ok().json(user))
}

#[delete("/users/{id}/")]
async fn delete(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let num_deleted = User::delete(id.into_inner())?;
    Ok(HttpResponse::Ok().json(json!({ "deleted": num_deleted })))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
    cfg.service(find);
    cfg.service(register);
    cfg.service(sign_in);
    cfg.service(update);
    cfg.service(delete);
}
