use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
use hyperdot_common_config::Chain;
use hyperdot_common_config::PostgresDataEngine;
use hyperdot_common_config::PostgresDataEngineConnection;
use hyperdot_common_config::PostgresDataEngineForChain;
use hyperdot_common_config::PublicChain;
use rust_decimal::prelude::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::Serialize;
use subxt::ext::codec::Decode;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_postgres::types::FromSql;
use tokio_postgres::types::Type;
use tokio_postgres::Client;
use tokio_postgres::Column;
use tokio_postgres::NoTls;
use tokio_postgres::Row;

use super::super::engine::DataEngine;
use super::writer::SubstrateWriter;
use crate::runtime_api::polkadot;
use crate::runtime_api::GetName;
use crate::types::block::polkadot_chain;
use crate::types::rpc::WriteBlockRequest;

pub type JSONValue = serde_json::Value;

fn convert_primitive_type<'a, T: FromSql<'a>>(
    row: &'a Row,
    column: &Column,
    column_i: usize,
    cfn: impl Fn(T) -> Result<JSONValue, anyhow::Error>,
) -> Result<JSONValue, anyhow::Error> {
    let raw_val = row
        .try_get::<_, Option<T>>(column_i)
        .with_context(|| format!("column_name:{}", column.name()))?;
    raw_val.map_or(Ok(JSONValue::Null), cfn)
}

fn convert_array_type<'a, T: FromSql<'a>>(
    row: &'a Row,
    column: &Column,
    column_i: usize,
    cfn: impl Fn(T) -> Result<JSONValue, anyhow::Error>,
) -> Result<JSONValue, anyhow::Error> {
    let raw_val_array = row
        .try_get::<_, Option<Vec<T>>>(column_i)
        .with_context(|| format!("column_name:{}", column.name()))?;
    Ok(match raw_val_array {
        Some(val_array) => {
            let mut result = vec![];
            for val in val_array {
                result.push(cfn(val)?);
            }
            JSONValue::Array(result)
        }
        None => JSONValue::Null,
    })
}

// For TS_VECTOR convert
struct StringCollector(String);
impl FromSql<'_> for StringCollector {
    fn from_sql(
        _: &Type,
        raw: &[u8],
    ) -> Result<StringCollector, Box<dyn std::error::Error + Sync + Send>> {
        let result = std::str::from_utf8(raw)?;
        Ok(StringCollector(result.to_owned()))
    }
    fn accepts(_ty: &Type) -> bool {
        true
    }
}

pub fn to_json_value(
    row: &Row,
    column: &Column,
    column_idx: usize,
) -> Result<JSONValue, anyhow::Error> {
    let f64_to_json_number = |raw_val: f64| -> Result<JSONValue, anyhow::Error> {
        let temp =
            serde_json::Number::from_f64(raw_val.into()).ok_or(anyhow!("invalid json-float"))?;
        Ok(JSONValue::Number(temp))
    };

    Ok(match *column.type_() {
        // for rust-postgres <> postgres type-mappings: https://docs.rs/postgres/latest/postgres/types/trait.FromSql.html#types
        // for postgres types: https://www.postgresql.org/docs/7.4/datatype.html#DATATYPE-TABLE
        // single types
        Type::BOOL => {
            convert_primitive_type(row, column, column_idx, |a: bool| Ok(JSONValue::Bool(a)))?
        }

        Type::INT2 => convert_primitive_type(row, column, column_idx, |a: i16| {
            Ok(JSONValue::Number(serde_json::Number::from(a)))
        })?,

        Type::INT4 => convert_primitive_type(row, column, column_idx, |a: i32| {
            Ok(JSONValue::Number(serde_json::Number::from(a)))
        })?,

        Type::INT8 => convert_primitive_type(row, column, column_idx, |a: i64| {
            Ok(JSONValue::Number(serde_json::Number::from(a)))
        })?,

        Type::NUMERIC => {
            let decimal = row
                .try_get::<_, Option<Decimal>>(column_idx)
                .with_context(|| format!("column_name: {}", column.name()))?;

            decimal.map_or(JSONValue::Null, |decimal| {
                JSONValue::String(decimal.to_string())
            })
        }

        Type::TEXT | Type::VARCHAR => {
            convert_primitive_type(row, column, column_idx, |a: String| {
                Ok(JSONValue::String(a))
            })?
        }
        // Type::JSON | Type::JSONB => get_basic(row, column, column_i, |a: JSONValue| Ok(a))?,
        Type::FLOAT4 => convert_primitive_type(row, column, column_idx, |a: f32| {
            Ok(f64_to_json_number(a.into())?)
        })?,
        Type::FLOAT8 => {
            convert_primitive_type(row, column, column_idx, |a: f64| Ok(f64_to_json_number(a)?))?
        }
        // these types require a custom StringCollector struct as an intermediary (see struct at bottom)
        Type::TS_VECTOR => {
            convert_primitive_type(row, column, column_idx, |a: StringCollector| {
                Ok(JSONValue::String(a.0))
            })?
        }

        // array types
        Type::BOOL_ARRAY => {
            convert_array_type(row, column, column_idx, |a: bool| Ok(JSONValue::Bool(a)))?
        }
        Type::INT2_ARRAY => convert_array_type(row, column, column_idx, |a: i16| {
            Ok(JSONValue::Number(serde_json::Number::from(a)))
        })?,
        Type::INT4_ARRAY => convert_array_type(row, column, column_idx, |a: i32| {
            Ok(JSONValue::Number(serde_json::Number::from(a)))
        })?,
        Type::INT8_ARRAY => convert_array_type(row, column, column_idx, |a: i64| {
            Ok(JSONValue::Number(serde_json::Number::from(a)))
        })?,
        Type::TEXT_ARRAY | Type::VARCHAR_ARRAY => {
            convert_array_type(row, column, column_idx, |a: String| {
                Ok(JSONValue::String(a))
            })?
        }
        Type::JSON_ARRAY | Type::JSONB_ARRAY | Type::JSONB | Type::JSON => {
            unimplemented!("JSON TYPE FAMLIY")
            //    get_array(row, column, column_i, |a: JSONValue| Ok(a))?
        }
        Type::FLOAT4_ARRAY => convert_array_type(row, column, column_idx, |a: f32| {
            Ok(f64_to_json_number(a.into())?)
        })?,
        Type::FLOAT8_ARRAY => {
            convert_array_type(row, column, column_idx, |a: f64| Ok(f64_to_json_number(a)?))?
        }
        // these types require a custom StringCollector struct as an intermediary (see struct at bottom)
        Type::TS_VECTOR_ARRAY => {
            convert_array_type(row, column, column_idx, |a: StringCollector| {
                Ok(JSONValue::String(a.0))
            })?
        }

        _ => anyhow::bail!(
            "Cannot convert pg-cell \"{}\" of type \"{}\" to a JSONValue.",
            column.name(),
            column.type_().name()
        ),
    })
}
#[derive(Default, Serialize)]
pub struct PostgresRows {
    pub columns: Vec<String>,
    pub len: usize,
    pub rows: Vec<serde_json::Map<String, JSONValue>>,
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
            let mut result: serde_json::Map<String, JSONValue> = serde_json::Map::new();

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

pub(crate) struct ConnectionState {
    pub(crate) client: Client,
    pub(crate) used_connection: PostgresDataEngineConnection,
    pub(crate) support_chain: PostgresDataEngineForChain,
    pub(crate) connection_config: tokio_postgres::Config,
    pub(crate) connection_handle: JoinHandle<anyhow::Result<()>>,
}

pub struct PgEngine {
    // support chain name of connection map to state
    connections: RwLock<HashMap<String, Arc<ConnectionState>>>,
}

impl PgEngine {
    pub async fn new(engine: PostgresDataEngine) -> anyhow::Result<Self> {
        let mut connections = HashMap::new();
        for support_chain in engine.support_chains.iter() {
            if !support_chain.enabled {
                tracing::info!(
                    "💁 {}: skipped not enabled for postgres data engine",
                    support_chain.name
                );
                continue;
            }

            let used_connection = match engine
                .connections
                .iter()
                .find(|c| c.name == support_chain.use_connection)
            {
                None => {
                    tracing::error!(
                        "💁 {}: using connection name = {} for postgres, but not found in postgres.connections, please check config",
                        support_chain.name,
                        support_chain.use_connection,
                    );
                    continue;
                }
                Some(c) => c,
            };
            let used_connection_name = used_connection.name.clone();

            let mut connection_config = tokio_postgres::Config::default();
            connection_config.user(&used_connection.username);
            connection_config.password(&used_connection.password);
            connection_config.host(&used_connection.host);
            connection_config.port(used_connection.port);
            connection_config.dbname(&support_chain.dbname);
            let (client, connection) = match connection_config.connect(NoTls).await {
                Err(err) => {
                    tracing::error!(
                        "💔 {}: connection name = {} connect postgres error: {}",
                        support_chain.name,
                        used_connection.name,
                        err
                    );
                    continue;
                }
                Ok(res) => res,
            };

            // TODO: check database tables, if not exists maybe consider init it.
            let connection_handle = tokio::spawn(async move {
                // TODO: consider re-connection
                if let Err(err) = connection.await {
                    tracing::error!(
                        "🐛 {}: postgres connection has broken: {}",
                        used_connection_name,
                        err
                    );
                    return Err(anyhow!("{}", err));
                }
                return Ok(());
            });

            tracing::info!(
                "🙅 {}: postgres data engine connected at dbname({})",
                support_chain.name,
                support_chain.dbname,
            );

            connections.insert(
                support_chain.name.to_string(),
                Arc::new(ConnectionState {
                    support_chain: support_chain.clone(),
                    used_connection: used_connection.clone(),
                    connection_config,
                    client,
                    connection_handle,
                }),
            );
        }

        Ok(Self {
            connections: RwLock::new(connections),
        })
    }

    async fn get_conn_state_for_chain(&self, chain: &str) -> anyhow::Result<Arc<ConnectionState>> {
        let rl = self.connections.read().await;
        match rl.get(chain) {
            None => {
                return Err(anyhow::anyhow!("Postgres not support chain({})", chain));
            }
            Some(conn) => Ok(conn.clone()),
        }
    }
    pub async fn write_block_internal(
        &self,
        chain: &Chain,
        block: Box<dyn Any + Send + Sync>,
    ) -> anyhow::Result<()> {
        let conn_state = self.get_conn_state_for_chain(&chain.name).await?;
        let block = block.downcast::<polkadot_chain::Block>().unwrap();
        SubstrateWriter::write_block(conn_state.clone(), *block).await
    }

    /// Run query sql for chain.
    pub async fn query(
        &self,
        chain: &str,
        sql: &str,
    ) -> anyhow::Result<hyperdot_core::types::PostgresRows> {
        let conn_state = self.get_conn_state_for_chain(chain).await?;
        let rows = conn_state.client.query(sql, &[]).await.map_err(|err| {
            anyhow::anyhow!(
                "Postgres data engine run sql({}) for chain({}) error:{}",
                sql,
                chain,
                err
            )
        })?;

        hyperdot_core::types::PostgresRows::try_from(rows)
    }

    // pub async fn write_block_for_polkadot_chain(
    //     &self,
    //     chain: &Chain,
    //     block: Box<dyn Any + Send + Sync>,
    // ) -> anyhow::Result<()> {
    //     let runtime_name = chain
    //         .polkadot_runtime
    //         .as_ref()
    //         .map_or("substrate".to_string(), |r| r.config.clone());
    //     match runtime_name.as_str() {
    //         "polkadot" => {
    //             let block = block.downcast::<polkadot_chain::Block>().unwrap();
    //             self.write_block_for_polkadot_runtime_polkadot(chain, *block)
    //                 .await
    //         }
    //         "substrate" => {
    //             let block = block.downcast::<polkadot_chain::Block>().unwrap();
    //             self.write_block_for_polkadot_runtime_substrate(chain, *block)
    //                 .await
    //         }

    //         _ => unimplemented!(),
    //     }
    // }

    // pub async fn write_block_for_polkadot_runtime_polkadot(
    //     &self,
    //     chain: &Chain,
    //     block: polkadot_chain::Block,
    // ) -> anyhow::Result<()> {
    //     let state = match self.connections.get(&chain.name) {
    //         None => {
    //             tracing::error!(
    //                 "👉 Postgres can't writ block({}) for chain({}), connection not found",
    //                 block.header.block_number,
    //                 chain.name
    //             );
    //             return Ok(());
    //         }
    //         Some(state) => state,
    //     };

    //     todo!()
    // }

    // pub async fn write_block_for_polkadot_runtime_substrate(
    //     &self,
    //     chain: &Chain,
    //     block: polkadot_chain::Block,
    // ) -> anyhow::Result<()> {
    //     todo!()
    // }
}

#[async_trait::async_trait]
impl DataEngine for PgEngine {
    fn name(&self) -> String {
        "Postgres".to_string()
    }

    async fn write_block(
        &self,
        chain: Chain,
        blocks: Vec<Box<dyn Any + Send + Sync>>,
    ) -> anyhow::Result<()> {
        for block in blocks {
            match self.write_block_internal(&chain, block).await {
                Err(err) => {
                    tracing::error!("write block error: {}", err);
                    continue;
                }
                Ok(_) => {}
            };
        }

        Ok(())
    }
}
