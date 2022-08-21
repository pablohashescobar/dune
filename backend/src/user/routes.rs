use crate::api_error::ApiError;
use crate::user::{AuthResponse, LoginRequest, User, UserMessage};
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
async fn register(user: web::Json<UserMessage>) -> Result<HttpResponse, ApiError> {
    let user = User::create(user.into_inner())?;

    let token = user.generate_token().unwrap();

    let response = AuthResponse { token: token };

    Ok(HttpResponse::Ok().json(response))
}

#[post("/sign-in/")]
async fn sign_in(credentials: web::Json<LoginRequest>) -> Result<HttpResponse, ApiError> {
    let credentials = credentials.into_inner();

    let user = User::find_by_email_or_username(credentials.email, credentials.username)?;

    let is_valid = user.verify_password(credentials.password.as_bytes())?;

    if is_valid {
        let token = user.generate_token().unwrap();
        let response = AuthResponse { token: token };

        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(ApiError::new(401, "Invalid Credentials".to_string()))
    }
}

#[put("/users/{id}/")]
async fn update(
    id: web::Path<Uuid>,
    user: web::Json<UserMessage>,
) -> Result<HttpResponse, ApiError> {
    let user = User::update(id.into_inner(), user.into_inner())?;
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
