use axum::{response::IntoResponse, Json};
use std::error::Error;

use crate::resp::Neo4jResp;

#[derive(Debug)]
pub enum AppErrorType {
    Ok,
    TestErr,
    Neo4jErr,
}

#[derive(Debug)]
pub struct AppError {
    errortype: AppErrorType,
}

impl AppError {
    pub fn code(&self) -> i32 {
        return self.errortype as i32;
    }

    pub fn new(errortype: AppErrorType) -> Self {
        return Self { errortype };
    }
}

impl From<neo4rs::Error> for AppError {
    fn from(_: neo4rs::Error) -> Self {
        return Self::new(AppErrorType::Neo4jErr);
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
