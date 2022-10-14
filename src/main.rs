use axum::{routing::get, Router};
use handler::get_error;
use std::env;

mod db;
mod err;
mod handler;
mod resp;

use crate::handler::query_neo4j;
use err::AppError;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let app = Router::new()
        .route("/", get(query_neo4j))
        .route("/error", get(get_error));

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
