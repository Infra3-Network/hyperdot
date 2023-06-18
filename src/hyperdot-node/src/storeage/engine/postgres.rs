use std::any::Any;
use std::collections::HashMap;

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
use tokio::task::JoinHandle;
use tokio_postgres::types::FromSql;
use tokio_postgres::types::Type;
use tokio_postgres::Client;
use tokio_postgres::Column;
use tokio_postgres::NoTls;
use tokio_postgres::Row;

use super::utils::FiveTopics;
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

#[derive(Debug)]
pub struct PostgresStorageParams {
    // TODO: postgres://{raw} in uper
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

impl PostgresStorageParams {
    pub fn to_url(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname = {}",
            self.host, self.port, self.user, self.password, self.dbname,
        )
    }
}

struct ConnectionState {
    client: Client,
    used_connection: PostgresDataEngineConnection,
    support_chain: PostgresDataEngineForChain,
    connection_config: tokio_postgres::Config,
    connection_handle: JoinHandle<anyhow::Result<()>>,
}

pub struct PostgresStorage {
    params: PostgresStorageParams,
    pub pg_client: Client,
    pg_conn_handle: JoinHandle<anyhow::Result<()>>,
    connections: HashMap<String, ConnectionState>, // support chain name of connection map to state
}

impl PostgresStorage {
    pub async fn new(params: PostgresStorageParams) -> anyhow::Result<Self> {
        tracing::info!(
            "🐘 PostgresStorage: try connecting {}:{}, dbname = {}",
            params.host,
            params.port,
            params.dbname
        );
        let (pg_client, connection) = tokio_postgres::connect(&params.to_url(), NoTls).await?;

        // The connection object performs the actual communication with the database,
        // // so spawn it off to run on its own.
        let pg_conn_handle = tokio::spawn(async move {
            if let Err(err) = connection.await {
                tracing::error!("🐛 PostgresStorage: postgres connection error: {}", err);
                return Err(anyhow!("{}", err));
            }
            return Ok(());
        });

        tracing::info!(
            "🐘 PostgresStorage: connected {}:{}, dbname = {}",
            params.host,
            params.port,
            params.dbname
        );
        Ok(Self {
            params,
            pg_client,
            pg_conn_handle,
            connections: HashMap::new(),
        })
    }
}

pub struct PolkadotPostgresStorageImpl {
    pub base: PostgresStorage,
}

impl PolkadotPostgresStorageImpl {
    async fn write_events(
        &self,
        header: &crate::types::polkadot::BlockHeader,
        events: &mut [crate::types::polkadot::EventDescribe],
    ) -> anyhow::Result<()> {
        let block_number = header.block_number as i64;

        let block_hash_hex = format!("0x{}", hex::encode(&header.block_hash));

        let upsert_raw_event_statemant = "INSERT INTO raw_event (
								block_number, 
								block_hash, 
								block_time,
								extrinsic_hash,
								data,
								index,
								topic0, 
								topic1,
								topic2, 
								topic3,
								topic4,
								phase,
								pallet_name,
								pallet_index,
								event_name
							) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                            ON CONFLICT (block_number, index)
                            DO UPDATE SET
                                block_number = excluded.block_number,
                                block_hash = excluded.block_hash,
                                extrinsic_hash = excluded.extrinsic_hash,
                                data = excluded.data,
                                index = excluded.index,
                                topic0 = excluded.topic0,
                                topic1 = excluded.topic1,
                                topic2 = excluded.topic2,
                                topic3 = excluded.topic3,
                                topic4 = excluded.topic4,
                                phase = excluded.phase,
                                pallet_name = excluded.pallet_name,
                                event_name = excluded.event_name";

        let upsert_transfer_statemant = "INSERT INTO transfer (
                                block_number, 
                                block_hash, 
                                index, 
                                pallet_index, 
                                pallet_name, 
                                event_name, 
                                extrinsic_hash, 
                                \"from\", 
                                \"to\", 
                                amount, 
                                success
                            )VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)";
        // ON CONFLICT (block_number)
        // DO UPDATE SET
        //     block_hash = excluded.block_hash,
        //     index = excluded.index,
        //     pallet_index = excluded.pallet_index,
        //     pallet_name = excluded.pallet_name,
        //     hash = excluded.hash,
        //     \"from\" = excluded.from,
        //     \"to\" = excluded.to,
        //     amount = excluded.amount";

        let upsert_withdraw_statemant = "INSERT INTO withdraw (
                                block_number, 
                                block_hash, 
                                index, 
                                pallet_index, 
                                pallet_name, 
                                event_name,
                                extrinsic_hash, 
                                who, 
                                amount, 
                                success
                            )VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                   ON CONFLICT (block_number, index)
                   DO UPDATE SET
                        block_hash = excluded.block_hash,
                        index = excluded.index,
                        pallet_index = excluded.pallet_index,
                        pallet_name = excluded.pallet_name,
                        extrinsic_hash = excluded.extrinsic_hash,
                        who = excluded.who,
                        amount = excluded.amount";

        // ON CONFLICT (block_number)
        // DO UPDATE SET
        //     block_hash = excluded.block_hash,
        //     index = excluded.index,
        //     pallet_index = excluded.pallet_index,
        //     pallet_name = excluded.pallet_name,
        //     hash = excluded.hash,
        //     who = excluded.who,
        //     amount = excluded.amount";

        let mut stmts = vec![];
        for _ in 0..events.len() {
            // FIXME: make stream concurrent
            let stmt = self
                .base
                .pg_client
                .prepare(upsert_raw_event_statemant)
                .await?;
            stmts.push(stmt);
        }

        let mut extrinsic_success = false;
        for (_, event_desc) in events.iter_mut().enumerate() {
            let root_event = polkadot::Event::decode(&mut event_desc.root_bytes.as_ref()).unwrap();
            match root_event {
                polkadot::Event::System(system_event) => match system_event {
                    polkadot::system::Event::ExtrinsicSuccess { .. } => extrinsic_success = true,
                    _ => {}
                },
                _ => {}
            }
        }

        // upsert for raw_event
        for (i, event_desc) in events.iter_mut().enumerate() {
            let root_event = polkadot::Event::decode(&mut event_desc.root_bytes.as_ref())?;

            let (_, event_name) = root_event.name();
            let five_topics = FiveTopics::from(&event_desc.topics);
            let block_time: i64 = 0; // FIXME: which block time?
            let extrinsic_hash_hex = format!("0x{}", hex::encode(&event_desc.extrinsic_hash));
            let data_hex = format!("0x{}", hex::encode(&event_desc.data));
            let index = event_desc.index as i32;
            // FIXME: make stream concurrent
            let rows = self
                .base
                .pg_client
                .execute(&stmts[i], &[
                    &block_number,
                    &block_hash_hex,
                    &block_time, // FIXME: block_time
                    &extrinsic_hash_hex,
                    &data_hex,
                    &index,
                    &five_topics.t0,
                    &five_topics.t1,
                    &five_topics.t2,
                    &five_topics.t3,
                    &five_topics.t4,
                    &event_desc.phase.to_string(),
                    &event_desc.pallet_name,
                    &(event_desc.pallet_index as i16),
                    &event_name,
                ])
                .await?;
            // Check if the row count is 1, indicating a successful insert
            if rows == 1 {
                println!("raw_event inserted successfully");
            } else {
                println!("raw_event updated successfully");
            }

            match root_event {
                polkadot::Event::Balances(balance_event) => match balance_event {
                    polkadot::balances::Event::Transfer { from, to, amount } => {
                        let event_name = "Transfer";
                        let from_hex = format!("0x{}", hex::encode(from.0));
                        let to_hex = format!("0x{}", hex::encode(to.0));
                        let amount = Decimal::from_u128(amount)
                            .expect("parse transfer u128 to decimal error");

                        let rows = self
                            .base
                            .pg_client
                            .execute(upsert_transfer_statemant, &[
                                &block_number,
                                &block_hash_hex,
                                &index,
                                &(event_desc.pallet_index as i16),
                                &event_desc.pallet_name,
                                &event_name,
                                &extrinsic_hash_hex,
                                &from_hex,
                                &to_hex,
                                &amount,
                                &extrinsic_success,
                            ])
                            .await?; // FIXME: not fault process

                        if rows == 1 {
                            println!("insert transfer #{} success", block_number);
                        }
                    }
                    polkadot::balances::Event::Withdraw { who, amount } => {
                        let event_name = "Withdraw";
                        let who_hex = format!("0x{}", hex::encode(who.0));
                        let amount = Decimal::from_u128(amount)
                            .expect("parse withdraw u128 to decimal error");

                        let rows = self
                            .base
                            .pg_client
                            .execute(upsert_withdraw_statemant, &[
                                &block_number,
                                &block_hash_hex,
                                &index,
                                &(event_desc.pallet_index as i16),
                                &event_desc.pallet_name,
                                &event_name,
                                &extrinsic_hash_hex,
                                &who_hex,
                                &amount,
                                &extrinsic_success,
                            ])
                            .await?; // FIXME: not fault process

                        if rows == 1 {
                            println!("insert withdraw #{} success", block_number);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }
}

impl PolkadotPostgresStorageImpl {
    pub async fn write_block(
        &self,
        req: WriteBlockRequest<crate::types::polkadot::Block>,
    ) -> anyhow::Result<()> {
        let mut blocks = req
            .blocks
            .into_iter()
            .map(|b| b.clone())
            .collect::<Vec<_>>();
        tracing::info!(
            "🐘 PostgresStorage: writing blocks #{:?}",
            blocks
                .iter()
                .map(|blk| format!("#{}", blk.header.block_number))
                .collect::<Vec<_>>()
        );

        let upsert_statement =
            "INSERT INTO block (block_number, block_hash, parent_hash, state_root, extrinsics_root)
                            VALUES ($1, $2, $3, $4, $5)
                            ON CONFLICT (block_number)
                            DO UPDATE SET
                                block_hash = excluded.block_hash,
                                parent_hash = excluded.parent_hash,
                                state_root = excluded.state_root,
                                extrinsics_root = excluded.extrinsics_root";

        for block in blocks.iter_mut() {
            let block_number = block.header.block_number as i64;
            let block_hash_hex = format!("0x{}", hex::encode(&block.header.block_hash));
            let parent_hash_hex = format!("0x{}", hex::encode(&block.header.parent_hash));
            let state_root_hex = format!("0x{}", hex::encode(&block.header.state_root));
            let extrinsics_root_hex = format!("0x{}", hex::encode(&block.header.extrinsics_root));
            let rows = self
                .base
                .pg_client
                .execute(upsert_statement, &[
                    &block_number,
                    &block_hash_hex,
                    &parent_hash_hex,
                    &state_root_hex,
                    &extrinsics_root_hex,
                ])
                .await?;
            // Check if the row count is 1, indicating a successful insert
            if rows == 1 {
                println!("block inserted successfully");
            } else {
                println!("block updated successfully");
            }

            for extrinsic in block.extrinsics.iter_mut() {
                if extrinsic.events.is_empty() {
                    tracing::info!("{} events empty, skip it", extrinsic.pallet_name);
                    continue;
                }

                self.write_events(&block.header, &mut extrinsic.events)
                    .await?;
            }
        }
        Ok(())
    }
}
