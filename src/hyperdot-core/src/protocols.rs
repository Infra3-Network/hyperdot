use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use super::types::ChainKind;
use super::types::EngineInfo;
use super::types::PostgresRows;
use super::types::PostgresTableInfo;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ResponseCode {
    Success,
    Error,
}

impl Default for ResponseCode {
    fn default() -> Self {
        Self::Success
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub code: Option<ResponseCode>,
    pub message: Option<String>,
    pub reason: Option<String>,
}

impl ResponseMetadata {
    pub fn success(message: &str) -> Self {
        Self {
            code: Some(ResponseCode::Success),
            message: Some(message.to_string()),
            reason: None,
        }
    }
    pub fn set_code(&mut self, code: ResponseCode) {
        self.code = Some(code)
    }

    pub fn set_reason(&mut self, reason: String) {
        self.reason = Some(reason)
    }

    pub fn set_error(&mut self, reason: String) {
        self.code = Some(ResponseCode::Error);
        self.reason = Some(reason)
    }

    pub fn set_success_msg(&mut self, msg: String) {
        self.code = Some(ResponseCode::Success);
        self.message = Some(msg)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ListDataEngineResquest {}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ListDataEngineResponse {
    pub header: ResponseMetadata,
    pub engines: HashMap<String, EngineInfo>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct GetPostgresSchemeRequest {
    pub chain: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct GetPostgresSchemeResponse {
    pub meta: ResponseMetadata,
    pub chain: String,
    pub tables: HashMap<String, Vec<PostgresTableInfo>>,
}

#[derive(Deserialize)]
pub struct QueryPostgresRequest {
    /// which engine is queried.
    pub engine: String,
    /// Which chain to query.
    pub chain: String,
    /// What query it is.
    pub query: String,
}

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct QueryPostgresResponse {
    pub meta: ResponseMetadata,
    pub rows: PostgresRows,
}
