//! Substrate chain extracter

use anyhow::anyhow;
// use hyperdot_core::runtime_api::kusama;
// use hyperdot_core::runtime_api::polkadot;
// use hyperdot_core::runtime_api::polkadot::Polkadot as default_runtime;
use subxt::blocks::Block as OnlineBlock;
use subxt::config::substrate::DigestItem;
use subxt::events::Phase;
use subxt::ext::sp_runtime::key_types;
use subxt::ext::sp_runtime::ConsensusEngineId;
use subxt::OnlineClient;
use subxt::SubstrateConfig;

use crate::types::block::polkadot_chain;

pub struct ExtractBlock;

impl ExtractBlock {
    pub async fn extract(
        online_block: OnlineBlock<SubstrateConfig, OnlineClient<SubstrateConfig>>,
    ) -> anyhow::Result<polkadot_chain::Block> {
        let block_storage: subxt::storage::Storage<SubstrateConfig, OnlineClient<SubstrateConfig>> =
            online_block.storage();
        let block_timestamp = block_storage
            .fetch(&default_runtime::storage().timestamp().now())
            .await?
            .map_or(0, |v| v);

        let header = polkadot_chain::Header {
            block_number: online_block.header().number as u64,
            block_timestamp,
            block_hash: online_block.hash().as_bytes().to_vec(),
            parent_hash: online_block.header().parent_hash.as_bytes().to_vec(),
            extrinsics_root: online_block.header().extrinsics_root.as_bytes().to_vec(),
            state_root: online_block.header().state_root.as_bytes().to_vec(),
        };

        let extraced_logs = Self::extract_logs(&header, &online_block.header().digest.logs);

        let online_body = online_block
            .body()
            .await
            .map_err(|err| anyhow!("block #{} get body error: {}", header.block_number, err))?;

        let mut extracted_exts = vec![];
        let mut extract_evs = vec![];
        for online_ext in online_body.extrinsics().iter() {
            let online_ext = online_ext.map_err(|err| {
                anyhow!(
                    "block #{} get extrinsic error: {}",
                    header.block_number,
                    err
                )
            })?;

            let online_evs = online_ext.events().await.map_err(|err| {
                anyhow!(
                    "block #{} get extrinsic #{} events error: {}",
                    header.block_number,
                    online_ext.index(),
                    err
                )
            })?;

            let call_params = online_ext
                .field_values()
                .and_then(|values| Ok(serde_json::to_value(values).map_or(None, |v| Some(v))))
                .map_or(None, |v| v);

            extracted_exts.push(polkadot_chain::Extrinsic {
                id: format!("{}-{}", header.block_number, online_ext.index()),
                block_number: header.block_number,
                block_timestamp,
                mod_name: online_ext
                    .pallet_name()
                    .map_or(String::new(), |s| s.to_string()),
                call_name: online_ext
                    .variant_name()
                    .map_or(String::new(), |s| s.to_string()),
                call_params, // TODO: add params
                signature: online_ext
                    .address_bytes()
                    .map_or(None, |bs| Some(bs.to_vec())),
            });

            for online_ev in online_evs.iter() {
                let online_ev = online_ev.map_err(|err| {
                    anyhow!(
                        "block #{} get extrinsic #{} event error: {}",
                        header.block_number,
                        online_ext.index(),
                        err
                    )
                })?;

                let values = online_ev
                    .field_values()
                    .and_then(|values| Ok(serde_json::to_value(values).map_or(None, |v| Some(v))))
                    .map_or(None, |v| v);

                extract_evs.push(polkadot_chain::Event {
                    id: format!("{}-{}", header.block_number, online_ev.index()),
                    block_number: header.block_number,
                    block_timestamp,
                    mod_name: online_ev.pallet_name().to_string(),
                    event_name: online_ev.variant_name().to_string(),
                    event_index: online_ev.index(),
                    phase: Self::extract_event_phase(online_ev.phase()),
                    extrinsic_hash: online_evs.extrinsic_hash().as_bytes().to_vec(),
                    extrinsic_index: online_evs.extrinsic_index(),
                    values,
                })
            }
        }

        Ok(polkadot_chain::Block {
            header,
            extrinsics: Some(extracted_exts),
            events: Some(extract_evs),
            logs: extraced_logs,
        })
    }

    fn extract_logs(
        header: &polkadot_chain::Header,
        logs: &[DigestItem],
    ) -> Option<Vec<polkadot_chain::Log>> {
        let mut result = vec![];
        for (i, log) in logs.iter().enumerate() {
            let (r#type, engine, data): (String, Option<String>, Option<Vec<u8>>) = match log {
                DigestItem::PreRuntime(engine_id, data) => (
                    "PreRuntime".to_string(),
                    Self::extrat_consensus_engine_id(engine_id),
                    Some(data.clone()),
                ),
                DigestItem::Consensus(engine_id, data) => (
                    "Consensus".to_string(),
                    Self::extrat_consensus_engine_id(engine_id),
                    Some(data.clone()),
                ),
                DigestItem::Seal(engine_id, data) => (
                    "Seal".to_string(),
                    Self::extrat_consensus_engine_id(engine_id),
                    Some(data.clone()),
                ),
                DigestItem::Other(data) => {
                    let r#type = "Other".to_string();
                    ("Other".to_string(), None, Some(data.clone()))
                }
                DigestItem::RuntimeEnvironmentUpdated => {
                    ("RuntimeEnvironmentUpdated".to_string(), None, None)
                }
            };

            result.push(polkadot_chain::Log {
                id: format!("{}-{}", header.block_number, i),
                block_number: header.block_number,
                r#type,
                engine,
                data,
            })
        }

        if result.is_empty() {
            return None;
        }

        Some(result)
    }

    fn extrat_consensus_engine_id(engine_id: &ConsensusEngineId) -> Option<String> {
        match subxt::ext::sp_runtime::KeyTypeId(*engine_id) {
            key_types::BABE => Some("Babe".to_string()),
            key_types::GRANDPA => Some("Grandpa".to_string()),
            key_types::AURA => Some("Aura".to_string()),
            _ => None,
        }
    }

    fn extract_event_phase(phase: Phase) -> u16 {
        match phase {
            Phase::ApplyExtrinsic(_) => 0,
            Phase::Finalization => 1,
            Phase::Initialization => 2,
        }
    }
}
