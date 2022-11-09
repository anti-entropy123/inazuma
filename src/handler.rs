use axum::extract::Query;
use log::{error, info};
use neo4rs::{Node, Path, RowStream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    db::{
        get_protein_path_by_score, get_related_protein_by_name, get_shortest_path,
        get_similar_proteins, is_related_proteins, neo4j_query_test,
    },
    err::{AppError, AppErrorType},
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

async fn get_protein_path_from_neo4j_row_stream(
    mut result: RowStream,
) -> Result<(HashMap<i32, String>, Vec<Interact>), AppError> {
    let mut proteins = HashMap::<i32, String>::new();
    let mut rels = Vec::<Interact>::new();
    while let Some(row) = result.next().await? {
        // log::debug!("row={:?}", row);
        let p: Path = row.get("p").expect("unexpect doesn't have a path");
        let mut rel = Vec::new();
        let mut scores: Vec<String> = Vec::new();
        for node in p.nodes() {
            if node.labels().contains(&"owl__Axiom".to_owned()) {
                scores.push(node.get("ns4__SWO_0000425").unwrap_or("".to_owned()));
                continue;
            }
            log::debug!("get_protein_path_from_neo4j_row_stream a node={:?}", node);
            proteins.insert(
                node.id() as i32,
                node.get("rdfs__label").unwrap_or("".to_owned()),
            );
            rel.push(node.id() as i32);
        }
        for idx in 0..scores.len() {
            match (rel.get(idx), rel.get(idx + 1), scores.get(idx)) {
                (Some(node1), Some(node2), Some(score)) => {
                    rels.push(Interact {
                        node1: *node1,
                        node2: *node2,
                        score: score.parse::<i32>().unwrap(),
                    });
                }
                _ => break,
            }
        }
    }
    return Ok((proteins, rels));
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
    let result = get_protein_path_by_score(args.score, limit).await?;
    let (proteins, rels) = get_protein_path_from_neo4j_row_stream(result).await?;
    resp_ok(ProteinPathResp {
        protein_names: proteins,
        rels,
    })
}

#[derive(Deserialize)]
pub struct ShortestPathArgs {
    pub source_protein: String,
    pub target_protein: String,
}

pub async fn query_shortest_path(
    Query(args): Query<ShortestPathArgs>,
) -> JsonResult<ProteinPathResp> {
    let result = get_shortest_path(&args.source_protein, &args.target_protein).await?;
    let (proteins, rels) = get_protein_path_from_neo4j_row_stream(result).await?;
    resp_ok(ProteinPathResp {
        protein_names: proteins,
        rels,
    })
}

#[derive(Deserialize)]
pub struct QuerySimilarityArgs {
    pub protein_num: i32,
    pub top_k: i32,
}

#[derive(Serialize)]
pub struct ProteinSimilarity {
    pub name1: String,
    pub name2: String,
    pub similarity: f64,
}

pub async fn query_similarity_proteins(
    Query(args): Query<QuerySimilarityArgs>,
) -> JsonResult<Vec<ProteinSimilarity>> {
    let mut result = get_similar_proteins(args.top_k, args.protein_num).await?;
    let mut resp = Vec::new();
    while let Some(row) = result.next().await? {
        let name1 = row.get::<String>("name1").expect("should have name1");
        let name2 = row.get::<String>("name2").expect("should have name2");
        let similarity = row
            .get::<f64>("similarity")
            .expect("should have similarity");
        resp.push(ProteinSimilarity {
            name1,
            name2,
            similarity,
        });
    }
    resp_ok(resp)
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
