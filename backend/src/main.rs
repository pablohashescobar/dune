#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use listenfd::ListenFd;
use std::env;
mod api_error;
mod db;
mod handlers;
mod models;
mod schema;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let secret_key = env::var("SECRET_TOKEN").expect("SECRET_TOKEN   NOT SET");
    let domain: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

    db::init();

    let mut listenfd = ListenFd::from_env();

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(secret_key.as_bytes())
                    .name("auth")
                    .path("/")
                    .domain(domain.as_str())
                    .secure(false), // this can only be true if you have https
            ))
            .service(
                web::scope("/api")
                    .configure(handlers::user_routes)
                    .configure(handlers::submission_routes)
                    .configure(handlers::benchmark_routes),
            )
    });
    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host: String = env::var("HOST").expect("Host not set");
            let port: String = env::var("PORT").expect("Port not set");
            server.bind(format!("{}:{}", host, port))?
        }
    };

    info!("Starting server 🚀");

    server.run().await
}
