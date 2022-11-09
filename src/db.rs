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

async fn common_query_neo4j(cyper: String, log_name: &str) -> Result<RowStream, AppError> {
    log::debug!("{} qeury sentence={}", log_name, cyper);
    let graph = get_graph_connect().await?;
    let result = graph.execute(query(&cyper)).await?;
    Ok(result)
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

pub async fn get_protein_path_by_score(score: i32, limit: i32) -> Result<RowStream, AppError> {
    let cyper = format!("MATCH p=(n1:owl__Class)<-- (score) -->(n2:owl__Class) where toInteger(score.ns4__SWO_0000425) > {} and n1.rdfs__comment <> n2.rdfs__comment RETURN p liMIT {}", score, limit);
    log::debug!("get_protein_path_by_score qeury sentence={}", cyper);
    let graph = get_graph_connect().await?;
    let result = graph.execute(query(&cyper)).await?;
    Ok(result)
}

pub async fn get_shortest_path(protein1: &str, protein2: &str) -> Result<RowStream, AppError> {
    let cyper = format!("match (a:owl__Class) where a.rdfs__label in ['{}', '{}'] with collect(id(a)) as nodeIds call gds.shortestPath.dijkstra.stream('dijk1', {{sourceNode:nodeIds[0], TargetNode:nodeIds[1]}}) yield sourceNode, targetNode, path return path as p;", protein1, protein2);
    log::debug!("get_shortest_path qeury sentence={}", cyper);
    let graph = get_graph_connect().await?;
    let result = graph.execute(query(&cyper)).await?;
    Ok(result)
}

pub async fn get_similar_proteins(topk: i32, proteins_num: i32) -> Result<RowStream, AppError> {
    let cyper = format!("CALL gds.knn.stream('proj', {{nodeLabels:['owl__Class'], nodeProperties:['embedding'], topK:{}}}) YIELD  node1, node2, similarity RETURN gds.util.asNode(node1).rdfs__label AS name1, gds.util.asNode(node2).rdfs__label AS name2, similarity LIMIT {}", topk ,topk * proteins_num);
    common_query_neo4j(cyper, "get_similar_proteins").await
}

pub async fn neo4j_query_test() -> Result<RowStream, AppError> {
    let cyper = format!("MATCH (n:Resource) RETURN n LIMIT 25");
    log::debug!("neo4j_query_test qeury sentence={}", cyper);
    let graph = get_graph_connect().await?;
    let result = graph.execute(query(&cyper)).await?;
    Ok(result)
}
