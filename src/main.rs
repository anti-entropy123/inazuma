use axum::{routing::get, Router};
use handler::get_error;

mod db;
mod err;
mod handler;
mod resp;

use crate::handler::query_neo4j;
use err::AppError;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new()
        .route("/", get(query_neo4j))
        .route("/error", get(get_error));

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
