use crate::api_error::ApiError;
use crate::models::{AuthUser, Submission, SubmissionInput};
use actix_web::{delete, get, post, put, web, HttpResponse};
use uuid::Uuid;

use serde_json::json;

#[get("/submissions/")]
async fn find_all() -> Result<HttpResponse, ApiError> {
    let submissions = Submission::find_all()?;

    Ok(HttpResponse::Ok().json(submissions))
}

#[get("/submissions/{id}/")]
async fn find(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let submission = Submission::find(id.into_inner())?;

    Ok(HttpResponse::Ok().json(submission))
}

#[post("/submissions/")]
async fn create(
    submission: web::Json<SubmissionInput>,
    identity: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let submission = Submission::create(submission.into_inner(), identity.id)?;

    Ok(HttpResponse::Ok().json(submission))
}

#[put("/submissions/{id}/")]
async fn update(
    submission: web::Json<SubmissionInput>,
    id: web::Path<Uuid>,
    identity: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let submission = Submission::update(id.into_inner(), submission.into_inner(), identity.id)?;

    Ok(HttpResponse::Ok().json(submission))
}

#[delete("/submissions/{id}/")]
async fn delete(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let num_deleted = Submission::delete(id.into_inner())?;

    Ok(HttpResponse::Ok().json(json!({ "deleted": num_deleted })))
}

#[get("/user/submissions/")]
async fn user_submissions(identity: AuthUser) -> Result<HttpResponse, ApiError> {
    let submissions = Submission::find_user_submissions(identity.id)?;

    Ok(HttpResponse::Ok().json(submissions))
}

pub fn submission_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
    cfg.service(find);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
    cfg.service(user_submissions);
}
