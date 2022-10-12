use axum::{routing::get, Json, Router};
use lazy_static::lazy_static;
use neo4rs::{config, query, Graph};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::OnceCell;

lazy_static! {
    static ref GRAPH: OnceCell<Arc<Graph>> = OnceCell::const_new();
}

async fn get_graph_connect() -> &'static Arc<Graph> {
    let config = config()
        .uri("127.0.0.1:7687")
        .user("neo4j")
        .password("123456")
        .build()
        .unwrap();

    GRAPH
        .get_or_init(|| async { Arc::new(Graph::connect(config).await.unwrap()) })
        .await
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new().route("/", get(query_neo4j));
    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

pub enum AppErrorType {}

#[derive(Serialize)]
struct Neo4jResp<T: Serialize> {
    pub errcode: i32,
    pub msg: String,
    pub data: Option<T>,
}

async fn query_neo4j() -> Json<Neo4jResp<Vec<String>>> {
    let graph = get_graph_connect().await;
    let mut result = graph
        .execute(query("MATCH (n:Resource) RETURN n LIMIT 25"))
        .await
        .unwrap();

    let mut resp: Vec<String> = Vec::new();
    while let Ok(Some(row)) = result.next().await {
        resp.push(format!("{:?}", row))
    }

    println!("{:?}", resp);
    Json(Neo4jResp {
        errcode: 0,
        msg: String::new(),
        data: Some(resp),
    })
}
