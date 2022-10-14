use crate::{err::AppErrorType, AppError};
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Neo4jResp<T: Serialize> {
    pub errcode: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> Neo4jResp<T>
where
    T: Serialize,
{
    pub fn new(errcode: i32, msg: String, data: Option<T>) -> Self {
        Neo4jResp { errcode, msg, data }
    }

    pub fn from_error(error: AppError) -> Neo4jResp<()> {
        Neo4jResp::new(error.code(), error.msg(), None)
    }
}

pub type JsonResult<T> = Result<Json<Neo4jResp<T>>, AppError>;

pub fn resp_ok<T>(data: T) -> JsonResult<T>
where
    T: Serialize,
{
    Ok(Json(Neo4jResp::new(
        AppErrorType::Ok as i32,
        String::new(),
        Some(data),
    )))
}

pub fn resp_err(errortype: AppErrorType, msg: String) -> JsonResult<()> {
    Err(AppError::new(errortype, msg))
}
