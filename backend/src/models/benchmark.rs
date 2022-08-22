use crate::api_error::ApiError;
use crate::db;
use crate::schema::benchmark;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[table_name = "benchmark"]
pub struct Benchmark {
    pub id: Uuid,
    pub title: String,
    pub subject: String,
    pub difficulty: String,
    pub creator_id: Option<Uuid>,
    pub git_url: Option<String>,
    pub max_cyclomatic_complex: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[table_name = "benchmark"]
pub struct BenchmarkMessage {
    pub title: String,
    pub subject: String,
    pub difficulty: String,
    pub git_url: Option<String>,
    pub creator_id: Uuid,
    pub max_cyclomatic_complex: i32,
}

#[derive(Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[table_name = "benchmark"]
pub struct BenchmarkInput {
    pub title: String,
    pub subject: String,
    pub difficulty: String,
    pub git_url: Option<String>,
    pub max_cyclomatic_complex: i32,
}

impl Benchmark {
    pub fn find_all() -> Result<Vec<Self>, ApiError> {
        let conn = db::connection()?;

        let benchmarks = benchmark::table.load::<Benchmark>(&conn)?;

        Ok(benchmarks)
    }

    pub fn find(id: Uuid) -> Result<Self, ApiError> {
        let conn = db::connection()?;

        let benchmark = benchmark::table.filter(benchmark::id.eq(id)).first(&conn)?;

        Ok(benchmark)
    }

    pub fn create(benchmark: BenchmarkInput, user_id: Uuid) -> Result<Self, ApiError> {
        let conn = db::connection()?;

        let benchmark = BenchmarkMessage {
            title: benchmark.title,
            subject: benchmark.subject,
            difficulty: benchmark.difficulty,
            max_cyclomatic_complex: benchmark.max_cyclomatic_complex,
            git_url: benchmark.git_url,
            creator_id: user_id,
        };

        let benchmark = Benchmark::from(benchmark);

        let benchmark = diesel::insert_into(benchmark::table)
            .values(benchmark)
            .get_result(&conn)?;

        Ok(benchmark)
    }

    pub fn update(id: Uuid, benchmark: BenchmarkInput, user_id: Uuid) -> Result<Self, ApiError> {
        let conn = db::connection()?;

        let benchmark = BenchmarkMessage {
            title: benchmark.title,
            subject: benchmark.subject,
            difficulty: benchmark.difficulty,
            max_cyclomatic_complex: benchmark.max_cyclomatic_complex,
            git_url: benchmark.git_url,
            creator_id: user_id,
        };

        let benchmark = Benchmark::from(benchmark);

        let benchmark = diesel::update(benchmark::table)
            .filter(benchmark::id.eq(id))
            .set(benchmark)
            .get_result(&conn)?;

        Ok(benchmark)
    }

    pub fn delete(id: Uuid) -> Result<usize, ApiError> {
        let conn = db::connection()?;

        let res = diesel::delete(benchmark::table.filter(benchmark::id.eq(id))).execute(&conn)?;

        Ok(res)
    }
}

impl From<BenchmarkMessage> for Benchmark {
    fn from(benchmark: BenchmarkMessage) -> Self {
        Benchmark {
            id: Uuid::new_v4(),
            title: benchmark.title,
            subject: benchmark.subject,
            difficulty: benchmark.difficulty,
            git_url: benchmark.git_url,
            max_cyclomatic_complex: benchmark.max_cyclomatic_complex,
            created_at: Utc::now().naive_utc(),
            updated_at: Some(Utc::now().naive_utc()),
            creator_id: Some(benchmark.creator_id),
        }
    }
}
