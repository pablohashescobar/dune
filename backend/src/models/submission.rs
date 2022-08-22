use crate::api_error::ApiError;
use crate::db;
use crate::schema::submission;
use base64;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "submission"]
pub struct Submission {
    pub id: Uuid,
    pub language: String,
    pub code: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub user_id: Uuid,
    pub status: String,
    pub benchmark_id: Option<Uuid>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exec_duration: i32,
    pub message: Option<String>,
    pub error: Option<String>,
    pub lint_score: Option<i32>,
    pub quality_score: Option<i32>,
    pub mem_usage: i32,
    pub code_hash: Option<String>,
    pub cyclomatic_complexity: i32,
}

#[derive(Serialize, Deserialize, AsChangeset)]
#[table_name = "submission"]
pub struct SubmissionMessage {
    pub language: String,
    pub code: String,
    pub user_id: Uuid,
    pub status: String,
    pub benchmark_id: Option<Uuid>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exec_duration: i32,
    pub message: Option<String>,
    pub error: Option<String>,
    pub lint_score: Option<i32>,
    pub quality_score: Option<i32>,
    pub mem_usage: i32,
    pub code_hash: Option<String>,
    pub cyclomatic_complexity: i32,
}

#[derive(Serialize, Deserialize, AsChangeset)]
#[table_name = "submission"]
pub struct SubmissionInput {
    pub language: String,
    pub code: String,
    pub status: String,
    pub benchmark_id: Option<Uuid>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exec_duration: i32,
    pub message: Option<String>,
    pub error: Option<String>,
    pub lint_score: Option<i32>,
    pub quality_score: Option<i32>,
    pub mem_usage: i32,
    pub cyclomatic_complexity: i32,
}

impl Submission {
    pub fn find_all() -> Result<Vec<Self>, ApiError> {
        let conn = db::connection()?;

        let submissions = submission::table.load::<Submission>(&conn)?;

        Ok(submissions)
    }

    pub fn find(id: Uuid) -> Result<Self, ApiError> {
        let conn = db::connection()?;

        let submission = submission::table
            .filter(submission::id.eq(id))
            .first(&conn)?;

        Ok(submission)
    }

    pub fn create(submission: SubmissionInput, user_id: Uuid) -> Result<Self, ApiError> {
        let mut hasher = Sha256::new();
        hasher.update(submission.code.as_bytes());
        let result = hasher.finalize();
        let encoded = base64::encode(&result);

        let submission = SubmissionMessage {
            language: submission.language,
            code: submission.code,
            user_id: user_id,
            status: submission.status,
            benchmark_id: submission.benchmark_id,
            stdout: submission.stdout,
            stderr: submission.stderr,
            exec_duration: submission.exec_duration,
            message: submission.message,
            error: submission.error,
            lint_score: submission.lint_score,
            quality_score: submission.quality_score,
            mem_usage: submission.mem_usage,
            code_hash: Some(encoded),
            cyclomatic_complexity: submission.cyclomatic_complexity,
        };

        let conn = db::connection()?;

        let submission = Submission::from(submission);

        let submission = diesel::insert_into(submission::table)
            .values(submission)
            .get_result(&conn)?;

        Ok(submission)
    }

    pub fn update(id: Uuid, submission: SubmissionInput, user_id: Uuid) -> Result<Self, ApiError> {
        let conn = db::connection()?;
        let mut hasher = Sha256::new();
        hasher.update(submission.code.as_bytes());
        let result = hasher.finalize();
        let encoded = base64::encode(&result);

        let submission = SubmissionMessage {
            language: submission.language,
            code: submission.code,
            user_id: user_id,
            status: submission.status,
            benchmark_id: submission.benchmark_id,
            stdout: submission.stdout,
            stderr: submission.stderr,
            exec_duration: submission.exec_duration,
            message: submission.message,
            error: submission.error,
            lint_score: submission.lint_score,
            quality_score: submission.quality_score,
            mem_usage: submission.mem_usage,
            code_hash: Some(encoded),
            cyclomatic_complexity: submission.cyclomatic_complexity,
        };

        let benchmark = diesel::update(submission::table)
            .filter(submission::id.eq(id))
            .set(submission)
            .get_result(&conn)?;

        Ok(benchmark)
    }

    pub fn delete(id: Uuid) -> Result<usize, ApiError> {
        let conn = db::connection()?;

        let res = diesel::delete(submission::table.filter(submission::id.eq(id))).execute(&conn)?;

        Ok(res)
    }

    pub fn find_user_submissions(user_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let conn = db::connection()?;

        let submissions = submission::table
            .filter(submission::user_id.eq(user_id))
            .get_results(&conn)?;

        Ok(submissions)
    }
}

impl From<SubmissionMessage> for Submission {
    fn from(submission: SubmissionMessage) -> Self {
        Submission {
            id: Uuid::new_v4(),
            language: submission.language,
            code: submission.code,
            user_id: submission.user_id,
            status: submission.status,
            benchmark_id: submission.benchmark_id,
            stdout: submission.stdout,
            stderr: submission.stderr,
            exec_duration: submission.exec_duration,
            message: submission.message,
            error: submission.error,
            lint_score: submission.lint_score,
            quality_score: submission.quality_score,
            mem_usage: submission.mem_usage,
            code_hash: submission.code_hash,
            cyclomatic_complexity: submission.cyclomatic_complexity,
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc()),
        }
    }
}
