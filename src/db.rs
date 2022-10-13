use lazy_static::lazy_static;
use neo4rs::{config, Graph};
use std::{result::Result, sync::Arc};
use tokio::sync::OnceCell;

use crate::err::AppError;

lazy_static! {
    static ref GRAPH: OnceCell<Arc<Graph>> = OnceCell::const_new();
}

pub async fn get_graph_connect() -> Result<&'static Arc<Graph>, AppError> {
    let config = config()
        .uri("127.0.0.1:7687")
        .user("neo4j")
        .password("123456")
        .build()?;

    Ok(GRAPH
        .get_or_init(|| async { Arc::new(Graph::connect(config).await.unwrap()) })
        .await)
}
