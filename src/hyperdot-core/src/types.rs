use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use tokio_postgres::Row;

use super::utils::to_json_value;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub support_chains: HashMap<String, ChainInfo>,
}

#[derive(Clone, Default, Serialize)]
pub struct PostgresTableInfo {
    pub column_name: String,
    pub data_type: String,
}

#[derive(Default, Serialize)]
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
