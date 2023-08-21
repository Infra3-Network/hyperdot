use std::sync::Arc;

use anyhow::anyhow;
use futures::future::try_join_all;
use subxt::config::Header;
use tokio_postgres::types::ToSql;

use super::pg::ConnectionState;
use crate::types::block::polkadot_chain;

pub(crate) struct SubstrateWriter;

const BLOCK_UPSERT_STMT: &'static str = r#"
INSERT INTO blocks (
    "number", 
    "timestamp", 
    "hash", 
    parent_hash, 
    extrinsics_root, 
    state_root, 
    is_finalized, 
    validator, 
    spec_version, 
    hash_bytes, 
    parent_hash_bytes, 
    extrinsics_root_bytes, 
    state_root_bytes, 
    validator_bytes
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
ON CONFLICT ("number") DO UPDATE
    SET
        "timestamp" = excluded."timestamp",
        "hash" = excluded."hash",
        parent_hash = excluded.parent_hash,
        extrinsics_root = excluded.extrinsics_root,
        state_root = excluded.state_root,
        is_finalized = excluded.is_finalized,
        validator = excluded.validator,
        spec_version = excluded.spec_version,
        hash_bytes = excluded.hash_bytes,
        parent_hash_bytes = excluded.parent_hash_bytes,
        extrinsics_root_bytes = excluded.extrinsics_root_bytes,
        state_root_bytes = excluded.state_root_bytes,
        validator_bytes = excluded.validator_bytes;
"#;

const LOG_UPSERT_STMT: &'static str = r#"
INSERT INTO block_logs (
    id, 
    block_number, 
    "type", 
    "data", 
    engine
) VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (id) DO UPDATE
    SET block_number = EXCLUDED.block_number,
        "type" = EXCLUDED.type,
        "data" = EXCLUDED.data,
        engine = EXCLUDED.engine
"#;

const EXTRINSICS_UPSERT_STMT: &'static str = r#"
INSERT INTO extrinsics (
    id, 
    block_number, 
    extrinsic_hash, 
    is_signed, 
    mod_name, 
    call_name, 
    result, 
    call_params, 
    extrinsic_hash_bytes
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT (id) DO UPDATE
SET
    block_number = EXCLUDED.block_number,
    extrinsic_hash = EXCLUDED.extrinsic_hash,
    is_signed = EXCLUDED.is_signed,
    mod_name = EXCLUDED.mod_name,
    call_name = EXCLUDED.call_name,
    result = EXCLUDED.result,
    call_params = EXCLUDED.call_params,
    extrinsic_hash_bytes = EXCLUDED.extrinsic_hash_bytes;
"#;

const EVENT_UPSERT_STMT: &'static str = r#"
INSERT INTO events (
    id, 
    block_number, 
    extrinsic_id, 
    mod_name, 
    event_name, 
    phase, 
    values
) VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (id) DO UPDATE
    SET block_number = EXCLUDED.block_number,
        extrinsic_id = EXCLUDED.extrinsic_id,
        mod_name = EXCLUDED.mod_name,
        event_name = EXCLUDED.event_name,
        phase = EXCLUDED.phase,
        values = EXCLUDED.values;

"#;

impl SubstrateWriter {
    pub(crate) async fn write_header(
        pg_conn_state: &Arc<ConnectionState>,
        block: &polkadot_chain::Block,
    ) -> anyhow::Result<()> {
        let validator = block.header.validator.as_ref().map_or(None, |validator| {
            Some(format!("0x{}", hex::encode(validator)))
        });
        let values: [&(dyn ToSql + Sync); 14] = [
            &(block.header.block_number as i64),
            &(block.header.block_timestamp as i64),
            &format!("0x{}", hex::encode(&block.header.block_hash)),
            &format!("0x{}", hex::encode(&block.header.parent_hash)),
            &format!("0x{}", hex::encode(&block.header.extrinsics_root)),
            &format!("0x{}", hex::encode(&block.header.state_root)),
            &block.header.is_finished,
            &validator,
            &(block.header.spec_version as i32),
            &block.header.block_hash,
            &block.header.parent_hash,
            &block.header.extrinsics_root,
            &block.header.state_root,
            &block.header.validator,
        ];
        let row = pg_conn_state
            .client
            .execute(BLOCK_UPSERT_STMT, &values)
            .await
            .map_err(|err| anyhow!("insert block #{} error: {}", block.header.block_number, err))?;

        if row == 0 {
            tracing::debug!("block #{} updated", block.header.block_number)
        } else {
            tracing::debug!("block #{} inserted", block.header.block_number)
        }

        Ok(())
    }

    pub(crate) async fn write_log(
        pg_conn_state: &Arc<ConnectionState>,
        block: &polkadot_chain::Block,
    ) -> anyhow::Result<()> {
        let logs = match block.logs.as_ref() {
            None => return Ok(()),
            Some(logs) => logs,
        };

        // make prepare stmts
        let mut prepare_futs = vec![];
        for i in 0..logs.len() {
            prepare_futs.push(pg_conn_state.client.prepare(LOG_UPSERT_STMT))
        }

        let stmts = try_join_all(prepare_futs).await.map_err(|err| {
            anyhow!(
                "prepare insert logs of block #{} error: {}",
                block.header.block_number,
                err
            )
        })?;

        // execute stmts
        assert_eq!(stmts.len(), logs.len());
        for (i, stmt) in stmts.iter().enumerate() {
            let log = &logs[i];
            let data = log
                .data
                .as_ref()
                .map_or(None, |v| Some(format!("0x{}", hex::encode(v))));

            let values: [&(dyn ToSql + Sync); 5] = [
                &log.id,
                &(log.block_number as i64),
                &log.r#type,
                &data,
                &log.engine,
            ];
            let row = pg_conn_state
                .client
                .execute(stmt, &values)
                .await
                .map_err(|err| {
                    anyhow!(
                        "execute insert logs of block #{} error: {}",
                        block.header.block_number,
                        err
                    )
                })?;

            if row == 0 {
                tracing::debug!("block #{}: log #{} updated", log.block_number, log.id)
            } else {
                tracing::debug!("block #{}: log #{} inserted", log.block_number, log.id)
            }
        }

        Ok(())
    }

    pub(crate) async fn write_extrinsics(
        pg_conn_state: &Arc<ConnectionState>,
        block: &polkadot_chain::Block,
    ) -> anyhow::Result<()> {
        let exts = match block.body.extrinsics.as_ref() {
            None => return Ok(()),
            Some(exts) => exts,
        };

        // make prepare stmts
        let mut prepare_futs = vec![];
        for i in 0..exts.len() {
            prepare_futs.push(pg_conn_state.client.prepare(EXTRINSICS_UPSERT_STMT))
        }

        let stmts = try_join_all(prepare_futs).await.map_err(|err| {
            anyhow!(
                "prepare insert extrinsics of block #{} error: {}",
                block.header.block_number,
                err
            )
        })?;

        // execute stmts
        assert_eq!(stmts.len(), exts.len());
        for (i, stmt) in stmts.iter().enumerate() {
            let ext = &exts[i];
            let is_signature = if ext.signature.is_none() { false } else { true };
            let extrinsic_hash = format!("0x{}", hex::encode(&ext.extrinsic_hash));
            let values: [&(dyn ToSql + Sync); 9] = [
                &ext.id,
                &(ext.block_number as i64),
                &extrinsic_hash,
                &is_signature,
                &ext.mod_name,
                &ext.call_name,
                &ext.result,
                &ext.call_params,
                &ext.extrinsic_hash,
            ];
            let row = pg_conn_state
                .client
                .execute(stmt, &values)
                .await
                .map_err(|err| {
                    anyhow!(
                        "execute insert extrinsics of block #{} error: {}",
                        block.header.block_number,
                        err
                    )
                })?;

            if row == 0 {
                tracing::debug!("block #{}: extrinsic #{} updated", ext.block_number, ext.id)
            } else {
                tracing::debug!(
                    "block #{}: extrinsic #{} inserted",
                    ext.block_number,
                    ext.id
                )
            }
        }

        Ok(())
    }

    pub(crate) async fn write_events(
        pg_conn_state: &Arc<ConnectionState>,
        block: &polkadot_chain::Block,
    ) -> anyhow::Result<()> {
        let events = match block.body.events.as_ref() {
            None => return Ok(()),
            Some(events) => events,
        };

        // make prepare stmts
        let mut prepare_futs = vec![];
        for i in 0..events.len() {
            prepare_futs.push(pg_conn_state.client.prepare(EVENT_UPSERT_STMT))
        }

        let stmts = try_join_all(prepare_futs).await.map_err(|err| {
            anyhow!(
                "prepare insert events of block #{} error: {}",
                block.header.block_number,
                err
            )
        })?;

        // execute stmts
        assert_eq!(stmts.len(), events.len());
        for (i, stmt) in stmts.iter().enumerate() {
            let event = &events[i];
            let values: [&(dyn ToSql + Sync); 7] = [
                &event.id,
                &(event.block_number as i64),
                &event.extrinsic_id,
                &event.mod_name,
                &event.event_name,
                &(event.phase as i16),
                &event.values,
            ];
            let row = pg_conn_state
                .client
                .execute(stmt, &values)
                .await
                .map_err(|err| {
                    anyhow!(
                        "execute insert events of block #{} error: {}",
                        block.header.block_number,
                        err
                    )
                })?;

            if row == 0 {
                tracing::debug!("block #{}: event #{} updated", event.block_number, event.id)
            } else {
                tracing::debug!(
                    "block #{}: event #{} inserted",
                    event.block_number,
                    event.id
                )
            }
        }
        Ok(())
    }

    pub(crate) async fn write_block(
        pg_conn_state: Arc<ConnectionState>,
        block: polkadot_chain::Block,
    ) -> anyhow::Result<()> {
        Self::write_header(&pg_conn_state, &block).await?;
        Self::write_log(&pg_conn_state, &block).await?;
        Self::write_extrinsics(&pg_conn_state, &block).await?;
        Self::write_events(&pg_conn_state, &block).await?;

        Ok(())
    }
}
