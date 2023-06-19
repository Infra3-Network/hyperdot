use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use tokio_postgres::Row;

use super::utils::to_json_value;

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum ChainKind {
    Ethereum,
    Polkadot,
}

impl Default for ChainKind {
    fn default() -> Self {
        Self::Polkadot
    }
}

impl ToString for ChainKind {
    fn to_string(&self) -> String {
        match *self {
            Self::Ethereum => "ethereum".to_string(),
            Self::Polkadot => "polkadot".to_string(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataEngineKind {
    Postgres,
}

impl Default for DataEngineKind {
    fn default() -> Self {
        Self::Postgres
    }
}

impl ToString for DataEngineKind {
    fn to_string(&self) -> String {
        match *self {
            Self::Postgres => "postgres".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresDataEngineForChain {
    /// The chain id.
    pub id: usize,
    /// The chain alias name.
    pub name: String,
    /// The chain storage used connection name.
    pub use_connection: String,
    /// The chain database name.
    pub dbname: String,
    /// If true the storage enabled
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresDataEngineConnection {
    /// The postgres connection identify name.
    pub name: String,
    /// The postgres connection username.
    pub username: String,
    /// The postgres connection password.
    pub password: String,
    /// The postgres connection host.
    pub host: String,
    /// The postgres connection port.
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresDataEngine {
    /// The multiple connections for postgres. It colud
    /// are same database or multiple database.
    pub connections: Vec<PostgresDataEngineConnection>,
    /// The postgres support chains
    /// Note: connections[i] + support_chains[i].dbname = real connection.
    pub support_chains: Vec<PostgresDataEngineForChain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataEngineInfo {
    pub kind: DataEngineKind,
    pub postgres: Option<PostgresDataEngine>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub support_chains: HashMap<String, ChainInfo>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PostgresTableInfo {
    pub column_name: String,
    pub data_type: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub enum PostgresColumnDataType {
    Boolean,
    Int2,
    Int4,
    Int8,
    Numberic,
    Bytea,
    Text,
    Varchar,
    Float4,
    Float8,
}

pub struct PostgresColumnData {
    pub column_type: PostgresColumnDataType,
    pub column_value: serde_json::Value,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PostgresRows {
    pub columns: Vec<String>,
    pub len: usize,
    pub rows: Vec<serde_json::Map<String, serde_json::Value>>,
}

impl TryFrom<Vec<Row>> for PostgresRows {
    type Error = anyhow::Error;
    fn try_from(rows: Vec<Row>) -> Result<Self, Self::Error> {
        let mut obj = Self {
            columns: vec![],
            len: rows.len(),
            rows: vec![],
        };

        if rows.is_empty() {
            return Ok(obj);
        }

        for col in rows[0].columns() {
            obj.columns.push(col.name().to_string());
        }

        for row in rows {
            let mut result: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

            for (idx, column) in row.columns().iter().enumerate() {
                let name = column.name();
                let json_value = to_json_value(&row, column, idx)?;
                result.insert(name.to_string(), json_value);
            }
            obj.rows.push(result);
        }
        Ok(obj)
    }
}
