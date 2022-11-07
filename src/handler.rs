use axum::extract::Query;
use log::{error, info};
use neo4rs::{Node, Path};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    db::{
        get_protein_path_by_score, get_related_protein_by_name, is_related_proteins,
        neo4j_query_test,
    },
    err::AppErrorType,
    resp::{resp_err, resp_ok, JsonResult},
};

#[derive(Serialize, Deserialize)]
pub struct ProteinResp {
    protein_names: HashMap<i32, String>,
    relate: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize)]
pub struct Interact {
    node1: i32,
    node2: i32,
    score: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ProteinPathResp {
    protein_names: HashMap<i32, String>,
    rels: Vec<Interact>,
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
    let limit: i32 = args.limit.unwrap_or(10);

    let mut result = get_related_protein_by_name(args.name, limit).await?;

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
            if is_related_proteins(n1.1, n2.1).await? {
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

#[derive(Deserialize, Debug)]
pub struct ProteinSetArgs {
    pub proteins: String,
}

pub async fn query_interact_of_protein_set(
    Query(args): Query<ProteinSetArgs>,
) -> JsonResult<ProteinResp> {
    let set: Vec<String> = args.proteins.split(",").map(|v| v.to_owned()).collect();

    let mut proteins = HashMap::<i32, String>::new();
    let mut rels = Vec::<(i32, i32)>::new();
    for (id1, n1) in set.iter().enumerate() {
        proteins.insert(id1 as i32, n1.to_owned());
        for (id2, n2) in set.iter().enumerate() {
            if n1 <= n2 {
                continue;
            }
            if is_related_proteins(n1, n2).await? {
                rels.push((id1 as i32, id2 as i32))
            }
        }
    }
    resp_ok(ProteinResp {
        protein_names: proteins,
        relate: rels,
    })
}

#[derive(Deserialize, Debug)]
pub struct ProteinScoreArgs {
    pub score: i32,
    pub limit: Option<i32>,
}

pub async fn query_interact_path_by_score(
    Query(args): Query<ProteinScoreArgs>,
) -> JsonResult<ProteinPathResp> {
    let limit = args.limit.unwrap_or(15);
    let mut result = get_protein_path_by_score(args.score, limit).await?;

    let mut proteins = HashMap::<i32, String>::new();
    let mut rels = Vec::<Interact>::new();
    while let Some(row) = result.next().await? {
        let p: Path = row.get("p").expect("unexpect doesn't have a path");
        let mut rel = Vec::new();
        let mut score = String::new();
        for node in p.nodes() {
            if node.labels().contains(&"owl__Axiom".to_owned()) {
                log::debug!(
                    "node ns4__SWO_0000425 = {:?}",
                    node.get::<String>("ns4__SWO_0000425")
                );
                score = node.get("ns4__SWO_0000425").unwrap_or("".to_owned());
                continue;
            }
            log::debug!("query_interact_path_by_score a node={:?}", node);
            proteins.insert(
                node.id() as i32,
                node.get("rdfs__label").unwrap_or("".to_owned()),
            );
            rel.push(node.id() as i32);
        }
        rels.push(Interact {
            node1: rel[0],
            node2: rel[1],
            score: score.parse::<i32>().unwrap(),
        })
    }

    resp_ok(ProteinPathResp {
        protein_names: proteins,
        rels: rels,
    })
}

pub async fn query_neo4j() -> JsonResult<Vec<String>> {
    let mut result = neo4j_query_test().await?;

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
