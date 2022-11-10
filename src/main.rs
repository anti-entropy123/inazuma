use axum::{routing::get, Router};
use std::env;
use tower_http::cors::CorsLayer;

mod db;
mod err;
mod handler;
mod resp;

use err::AppError;
use handler::{
    get_error, query_degree_top_k, query_interact_of_protein_set, query_interact_path_by_score,
    query_neo4j, query_protein_by_name, query_shortest_path, query_similarity_proteins,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "DEBUG");
    }
    env_logger::init();

    let app = Router::new()
        .route("/", get(query_neo4j))
        .route("/protein", get(query_protein_by_name))
        .route("/protein_set", get(query_interact_of_protein_set))
        .route("/interact_path", get(query_interact_path_by_score))
        .route("/shortest_path", get(query_shortest_path))
        .route("/similar_proteins", get(query_similarity_proteins))
        .route("/degree_top_k", get(query_degree_top_k))
        .route("/error", get(get_error))
        .layer(CorsLayer::permissive());

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
