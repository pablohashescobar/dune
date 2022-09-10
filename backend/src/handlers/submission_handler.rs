use crate::api_error::ApiError;
use crate::models::{AuthUser, Submission, SubmissionInput, SubmissionWorker};
use actix_web::{delete, get, post, put, web, HttpResponse};
use lapin::{
    options::*, publisher_confirm::Confirmation, BasicProperties, Connection, ConnectionProperties,
};
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

#[post("/submissions/run/{id}/")]
async fn run_user_submission(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let submission = Submission::find(id.into_inner())?;

    let addr = std::env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqp://admin:password@localhost:5672//dev".into());

    async_global_executor::block_on(async move {
        let conn = Connection::connect(&addr, ConnectionProperties::default())
            .await
            .unwrap();

        info!("CONNECTED");

        let channel_a = conn.create_channel().await.unwrap();

        let submission_request = SubmissionWorker {
            id: submission.id,
            language: submission.language,
            code: submission.code,
        };

        let serialized_submission = serde_json::to_string(&submission_request).unwrap();

        let confirm = channel_a
            .basic_publish(
                "jobs_ex",
                "jobs_rk",
                BasicPublishOptions::default(),
                serialized_submission.as_bytes(),
                BasicProperties::default(),
            )
            .await
            .unwrap()
            .await
            .unwrap();
        assert_eq!(confirm, Confirmation::NotRequested);
    });

    Ok(HttpResponse::Ok().json({}))
}

pub fn submission_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
    cfg.service(find);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
    cfg.service(user_submissions);
    cfg.service(run_user_submission);
}
