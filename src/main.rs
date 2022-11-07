use axum::{routing::get, Router};
use std::env;
use tower_http::cors::CorsLayer;

mod db;
mod err;
mod handler;
mod resp;

use err::AppError;
use handler::{
    get_error, query_interact_of_protein_set, query_interact_path_by_score, query_neo4j,
    query_protein_by_name,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();

    let app = Router::new()
        .route("/", get(query_neo4j))
        .route("/protein", get(query_protein_by_name))
        .route("/protein_set", get(query_interact_of_protein_set))
        .route("/interact_path", get(query_interact_path_by_score))
        .route("/error", get(get_error))
        .layer(CorsLayer::permissive());

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
