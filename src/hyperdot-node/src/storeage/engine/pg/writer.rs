use std::sync::Arc;

use anyhow::anyhow;
use subxt::config::Header;

use super::pg::ConnectionState;
use crate::types::block::polkadot_chain;

pub(crate) struct SubstrateWriter;

const BLOCK_UPSERT_STMT: &'static str = r#"
INSERT INTO blocks (
    number, 
    hash, 
    parent_hash, 
    state_root, 
    extrinsics_root,
    hash_bytes, 
    parent_hash_bytes, 
    state_root_bytes, 
    extrinsics_root_bytes
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT (number) DO NOTHING
"#;

const EXTRINSICS_UPSERT_STMT: &'static str = r#"
INSERT INTO extrinsics (
    block_number, 
    block_hash, 
    index,
    is_signed,
    pallet_name,
    variant_name,
    pallet_index,
    variant_index,
    signed_address, 
    block_hash_bytes, 
    signed_address_bytes
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
ON CONFLICT (block_number, index)
DO UPDATE SET
    block_number = excluded.block_number,
    block_hash = excluded.block_hash,
    index = excluded.index,
    pallet_name = excluded.pallet_name,
    variant_name = excluded.variant_name,
    pallet_index = excluded.pallet_index,
    variant_index = excluded.variant_index,
    signed_address = excluded.signed_address,
    block_hash_bytes = excluded.block_hash_bytes,
    signed_address_bytes = excluded.signed_address_bytes
"#;

impl SubstrateWriter {
    pub(crate) async fn write_block(
        pg_conn_state: Arc<ConnectionState>,
        block: polkadot_chain::Block,
    ) -> anyhow::Result<()> {
        let row = pg_conn_state
            .client
            .execute(BLOCK_UPSERT_STMT, &[
                &(block.header.block_number as i64),
                &format!("0x{}", hex::encode(&block.header.block_hash)),
                &format!("0x{}", hex::encode(&block.header.parent_hash)),
                &format!("0x{}", hex::encode(&block.header.extrinsics_root)),
                &format!("0x{}", hex::encode(&block.header.state_root)),
                &block.header.block_hash,
                &block.header.parent_hash,
                &block.header.extrinsics_root,
                &block.header.state_root,
            ])
            .await
            .map_err(|err| {
                anyhow!(
                    "insert block #{} to blocks error: {}",
                    block.header.block_number,
                    err
                )
            })?;

        let body = match block.body.as_ref() {
            None => return Ok(()),
            Some(body) => body,
        };
        for ext in body.extrinsics.iter() {
            pg_conn_state
                .client
                .execute(EXTRINSICS_UPSERT_STMT, &[
                    &(block.header.block_number as i64),
                    &format!("0x{}", hex::encode(&block.header.block_hash)),
                    &(ext.index as i32),
                    &ext.is_signed,
                    &ext.pallet_name,
                    &ext.variant_name,
                    &(ext.pallet_index as i16),
                    &(ext.variant_index as i16),
                    &ext.signed_address
                        .as_ref()
                        .map(|bs| format!("0x{}", hex::encode(bs))),
                    &block.header.block_hash,
                    &ext.signed_address,
                ])
                .await
                .map_err(|err| {
                    anyhow!(
                        "insert block #{} to extrinsics error: {}",
                        block.header.block_number,
                        err
                    )
                })?;
        }
        Ok(())
    }
}
