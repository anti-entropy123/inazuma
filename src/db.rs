use lazy_static::lazy_static;
use neo4rs::{config, query, Graph, RowStream};
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

pub async fn get_related_protein_by_name(name: String, limit: i32) -> Result<RowStream, AppError> {
    let cyper = format!("MATCH (n1:owl__Class) -[r*2]- (n2:owl__Class{{rdfs__label:'{}'}}) Return n1, n2, r Limit {} ;", name, limit);
    log::debug!("get_related_protein_by_name qeury sentence={}", cyper);

    let graph = get_graph_connect().await?;
    let result = graph.execute(query(&cyper)).await?;

    Ok(result)
}

pub async fn is_related_proteins(protein1: &String, protein2: &String) -> Result<bool, AppError> {
    let cyper = format!("MATCH (n1:owl__Class{{rdfs__label:'{}'}}) <-[r1]- () -[r2]-> (n2:owl__Class{{rdfs__label:'{}'}}) Return *; ", protein1, protein2);
    log::debug!("is_related_proteins qeury sentence={}", cyper);

    let graph = get_graph_connect().await?;
    let mut result = graph.execute(query(&cyper)).await?;

    Ok(result.next().await?.is_some())
}

pub async fn neo4j_query_test() -> Result<RowStream, AppError> {
    let cyper = format!("MATCH (n:Resource) RETURN n LIMIT 25");
    log::debug!("neo4j_query_test qeury sentence={}", cyper);

    let graph = get_graph_connect().await?;
    let result = graph.execute(query(&cyper)).await?;

    Ok(result)
}
