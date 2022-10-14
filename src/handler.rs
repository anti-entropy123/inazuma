use log::info;
use neo4rs::{query, Node};

use crate::db::get_graph_connect;
use crate::err::AppErrorType;
use crate::resp::{resp_err, resp_ok, JsonResult};

pub async fn query_neo4j() -> JsonResult<Vec<String>> {
    let graph = get_graph_connect().await?;
    let mut result = graph
        .execute(query("MATCH (n:Resource) RETURN n LIMIT 25"))
        .await
        .unwrap();

    let mut resp: Vec<String> = Vec::new();
    while let Ok(Some(row)) = result.next().await {
        let node = row.get::<Node>("n").unwrap();
        info!("node_id={:#?}", node.id());
        resp.push(format!("{:?}", node));
    }

    resp_ok(resp)
}

pub async fn get_error() -> JsonResult<()> {
    resp_err(AppErrorType::DefaultErr, "this is custom error".to_string())
}
