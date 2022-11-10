use function_name::named;
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

type Neo4jResult = Result<RowStream, AppError>;

async fn common_query_neo4j(cyper: String, log_name: &str) -> Neo4jResult {
    log::debug!("{} qeury sentence={}", log_name, cyper);
    let graph = get_graph_connect().await?;
    let result = graph.execute(query(&cyper)).await?;
    Ok(result)
}

#[named]
pub async fn get_related_protein_by_name(name: String, limit: i32) -> Neo4jResult {
    let cyper = format!("MATCH (n1:owl__Class) -[r*2]- (n2:owl__Class{{rdfs__label:'{}'}}) Return n1, n2, r Limit {} ;", name, limit);
    common_query_neo4j(cyper, function_name!()).await
}

#[named]
pub async fn is_related_proteins(protein1: &String, protein2: &String) -> Result<bool, AppError> {
    let cyper = format!("MATCH (n1:owl__Class{{rdfs__label:'{}'}}) <-[r1]- () -[r2]-> (n2:owl__Class{{rdfs__label:'{}'}}) Return *; ", protein1, protein2);
    let mut result = common_query_neo4j(cyper, function_name!()).await?;

    Ok(result.next().await?.is_some())
}

#[named]
pub async fn get_protein_path_by_score(score: i32, limit: i32) -> Neo4jResult {
    let cyper = format!("MATCH p=(n1:owl__Class)<-- (score) -->(n2:owl__Class) where toInteger(score.ns4__SWO_0000425) > {} and n1.rdfs__comment <> n2.rdfs__comment RETURN p liMIT {}", score, limit);
    common_query_neo4j(cyper, function_name!()).await
}

#[named]
pub async fn get_shortest_path(protein1: &str, protein2: &str) -> Neo4jResult {
    let cyper = format!("match (a:owl__Class) where a.rdfs__label in ['{}', '{}'] with collect(id(a)) as nodeIds call gds.shortestPath.dijkstra.stream('dijk1', {{sourceNode:nodeIds[0], TargetNode:nodeIds[1]}}) yield sourceNode, targetNode, path return path as p;", protein1, protein2);
    common_query_neo4j(cyper, function_name!()).await
}

#[named]
pub async fn get_similar_proteins(topk: i32, proteins_num: i32) -> Neo4jResult {
    let cyper = format!("CALL gds.knn.stream('proj', {{nodeLabels:['owl__Class'], nodeProperties:['embedding'], topK:{}}}) YIELD  node1, node2, similarity RETURN gds.util.asNode(node1).rdfs__label AS name1, gds.util.asNode(node2).rdfs__label AS name2, similarity LIMIT {}", topk ,topk * proteins_num);
    common_query_neo4j(cyper, function_name!()).await
}

#[named]
pub async fn get_degree_top_k(top_k: i32) -> Neo4jResult {
    let cyper = format!(
        "CALL gds.degree.stream('degree') \
         YIELD nodeId, score \
         RETURN gds.util.asNode(nodeId).rdfs__label AS name, score AS number \
         ORDER BY number DESCENDING, name LIMIT {}",
        top_k
    );
    common_query_neo4j(cyper, function_name!()).await
}

pub async fn neo4j_query_test() -> Neo4jResult {
    let cyper = format!("MATCH (n:Resource) RETURN n LIMIT 25");
    log::debug!("neo4j_query_test qeury sentence={}", cyper);
    let graph = get_graph_connect().await?;
    let result = graph.execute(query(&cyper)).await?;
    Ok(result)
}
