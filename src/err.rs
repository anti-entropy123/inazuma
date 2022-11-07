use axum::{response::IntoResponse, Json};
use derive_more::Display;
use log::info;
use std::error::Error;

use crate::resp::Neo4jResp;

#[derive(Debug, Display)]
pub enum AppErrorType {
    Ok,
    #[display(fmt = "DefaultErr")]
    DefaultErr,
    #[display(fmt = "Neo4jErr")]
    Neo4jErr,
    #[display(fmt = "ArgmentErr")]
    ArgmentErr,
}

#[derive(Debug)]
pub struct AppError {
    errortype: AppErrorType,
    msg: String,
}

impl AppError {
    pub fn code(&self) -> i32 {
        self.errortype as i32
    }

    pub fn msg(&self) -> String {
        format!("{}:{}", self.errortype, self.msg)
    }

    pub fn new(errortype: AppErrorType, msg: String) -> Self {
        Self { errortype, msg }
    }
}

impl From<neo4rs::Error> for AppError {
    fn from(e: neo4rs::Error) -> Self {
        info!("AppError from e=neo4rs::Error({:?})", e);
        Self::new(AppErrorType::Neo4jErr, format!("neo4rs::Error({:?})", e))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        log::info!("AppError from e=serde_json::Error({:?})", e);
        Self::new(
            AppErrorType::ArgmentErr,
            format!("serde_json::Error({:?})", e),
        )
    }
}

// 实现Display的trait
impl std::fmt::Display for AppError {
    // 一般情况下是固定写法
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "errortype={:?}", self.errortype)
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let resp: Neo4jResp<()> = Neo4jResp::<()>::from_error(self);
        Json(resp).into_response()
    }
}
