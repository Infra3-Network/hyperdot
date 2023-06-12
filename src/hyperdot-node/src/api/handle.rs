use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone)]
pub struct Context {
    pub polkadot_pg_client: Arc<tokio_postgres::Client>,
}

#[derive(Deserialize)]
pub struct PsqlQueryPayload {
    /// Which chain to query.
    chain: String,
    /// Which table of chain to query.
    table: String,
    /// The sql statement to query.
    sql: String,
}
#[derive(Serialize)]
pub struct PolkadotBlock {
    pub block_number: i64,
    pub block_hash: String,
    pub parent_hash: String,
    pub state_root: String,
    pub extrinsics_root: String,
}

#[derive(Serialize)]
pub struct PolkadotPsqlBlockQueryResponse {
    data: Vec<PolkadotBlock>,
}

pub async fn polkadot_psql_query_block(
    State(ctx): State<Context>,
    Json(payload): Json<PsqlQueryPayload>,
) -> Result<Json<PolkadotPsqlBlockQueryResponse>, (StatusCode, String)> {
    assert_eq!(payload.chain.as_str(), "polkadot");
    assert_eq!(payload.table.as_str(), "block");

    let sql = payload.sql;
    let rows = ctx.polkadot_pg_client.query(&sql, &[]).await.unwrap();

    let mut data = vec![];
    for row in rows.into_iter() {
        let block_number: i64 = row.get(0);
        let block_hash: Vec<u8> = row.get(1);
        let parent_hash: Vec<u8> = row.get(2);
        let state_root: Vec<u8> = row.get(3);
        let extrinsics_root: Vec<u8> = row.get(4);

        let block_hash = format!("0x{}", hex::encode(&block_hash));
        let parent_hash = format!("0x{}", hex::encode(&parent_hash));
        let state_root = format!("0x{}", hex::encode(&state_root));
        let extrinsics_root = format!("0x{}", hex::encode(&extrinsics_root));

        data.push(PolkadotBlock {
            block_number,
            block_hash,
            parent_hash,
            state_root,
            extrinsics_root,
        });
    }

    Ok(Json(PolkadotPsqlBlockQueryResponse { data }))
}
