use crate::api_error::ApiError;
use crate::models::{AuthUser, Benchmark, BenchmarkInput};
use actix_web::{delete, get, post, put, web, HttpResponse};
use uuid::Uuid;

use serde_json::json;

#[get("/benchmarks/")]
async fn find_all() -> Result<HttpResponse, ApiError> {
    let benchmarks = Benchmark::find_all()?;

    Ok(HttpResponse::Ok().json(benchmarks))
}

#[get("/benchmarks/{id}/")]
async fn find(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let benchmark = Benchmark::find(id.into_inner())?;

    Ok(HttpResponse::Ok().json(benchmark))
}

#[post("/benchmarks/")]
async fn create(
    benchmark: web::Json<BenchmarkInput>,
    identity: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let benchmark = Benchmark::create(benchmark.into_inner(), identity.id)?;

    Ok(HttpResponse::Ok().json(benchmark))
}

#[put("/benchmarks/{id}/")]
async fn update(
    benchmark: web::Json<BenchmarkInput>,
    id: web::Path<Uuid>,
    identity: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let benchmark = Benchmark::update(id.into_inner(), benchmark.into_inner(), identity.id)?;

    Ok(HttpResponse::Ok().json(benchmark))
}

#[delete("/benchmarks/{id}/")]
async fn delete(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let num_deleted = Benchmark::delete(id.into_inner())?;

    Ok(HttpResponse::Ok().json(json!({ "deleted": num_deleted })))
}

pub fn benchmark_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
    cfg.service(find);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}
