use axum::extract::Query;
use log::{error, info};
use neo4rs::{query, Node, Relation, UnboundedRelation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::option::IntoIter;

use crate::db::get_graph_connect;
use crate::err::AppErrorType;
use crate::resp::{resp_err, resp_ok, JsonResult};

#[derive(Serialize, Deserialize)]
pub struct ProteinResp {
    protein_names: HashMap<i32, String>,
    relate: Vec<(i32, i32)>,
}

#[derive(Deserialize, Debug)]
pub struct ProteinArgs {
    pub name: String,
    pub limit: Option<i32>,
}

fn get_protein_id_and_name(node: &Node) -> (i32, String) {
    match node.get("rdfs__label") {
        Some(name) => (node.id() as i32, name),
        None => (-1, String::new()),
    }
}

pub async fn query_protein_by_name(Query(args): Query<ProteinArgs>) -> JsonResult<ProteinResp> {
    log::debug!("/protein, args={:?}", args);
    let limit = match args.limit {
        Some(limit) => limit,
        None => 10,
    }
    .to_string();

    let cyper = format!("MATCH (n1:owl__Class) -[r*2]- (n2:owl__Class{{rdfs__label:'{}'}}) Return n1, n2, r Limit {} ;", args.name, limit);
    log::debug!("qeury semantic={}", cyper);
    let graph = get_graph_connect().await?;
    let mut result = graph.execute(query(&cyper)).await?;

    let mut proteins = HashMap::<i32, String>::new();
    let mut rels = Vec::<(i32, i32)>::new();
    while let Some(row) = result.next().await? {
        let n1 = get_protein_id_and_name(&row.get::<Node>("n1").unwrap());
        let n2 = get_protein_id_and_name(&row.get::<Node>("n2").unwrap());
        proteins.insert(n1.0, n1.1);
        proteins.insert(n2.0, n2.1);
    }

    for n1 in &proteins {
        for n2 in &proteins {
            if n1.0 <= n2.0 {
                continue;
            }
            let mut result = graph.execute(query(&format!("MATCH (n1:owl__Class{{rdfs__label:'{}'}}) <-[r1]- () -[r2]-> (n2:owl__Class{{rdfs__label:'{}'}}) Return *; ", n1.1, n2.1))).await?;
            if let Some(_) = result.next().await? {
                rels.push((*n1.0, *n2.0))
            }
        }
    }
    let resp = ProteinResp {
        protein_names: proteins,
        relate: rels,
    };
    resp_ok(resp)
}

pub async fn query_neo4j() -> JsonResult<Vec<String>> {
    let graph = get_graph_connect().await?;
    let mut result = graph
        .execute(query("MATCH (n:Resource) RETURN n LIMIT 25"))
        .await?;

    let mut resp: Vec<String> = Vec::new();
    while let Some(row) = result.next().await? {
        if let Some(node) = row.get::<Node>("n") {
            info!("node_id={:#?}", node.id());
            resp.push(format!("{:?}", node));
        } else {
            error!("neo4j row didn't has Node, row={:?}", row);
            return resp_err(
                AppErrorType::DefaultErr,
                "Invalid neo4j row format".to_string(),
            );
        }
    }

    resp_ok(resp)
}

pub async fn get_error() -> JsonResult<()> {
    resp_err(AppErrorType::DefaultErr, "this is custom error".to_string())
}
