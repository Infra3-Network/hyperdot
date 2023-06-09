//! Substrate chain extracter

use anyhow::anyhow;
// use hyperdot_core::runtime_api::kusama;
// use hyperdot_core::runtime_api::polkadot;
use hyperdot_core::runtime_api::polkadot::Polkadot as default_runtime;
use hyperdot_core::runtime_api::polkadot::Polkadot::system::events::ExtrinsicSuccess;
use subxt::blocks::Block as OnlineBlock;
use subxt::blocks::BlockBody;
use subxt::blocks::ExtrinsicDetails;
use subxt::blocks::ExtrinsicEvents;
use subxt::config::substrate::DigestItem;
use subxt::constants::ConstantsClient;
use subxt::events::EventDetails;
use subxt::events::Phase;
use subxt::ext::sp_runtime::key_types;
use subxt::ext::sp_runtime::ConsensusEngineId;
use subxt::OnlineClient;
use subxt::PolkadotConfig;

// use subxt::SubstrateConfig;
use crate::types::block::polkadot_chain;

struct BodyBuilder {
    block_number: Option<u64>,
    block_timestamp: Option<u64>,
    // is_finish: bool,
    is_success: bool,
    evs: Option<Vec<polkadot_chain::Event>>,
    exts: Option<Vec<polkadot_chain::Extrinsic>>,
}

impl BodyBuilder {
    pub(crate) fn new() -> Self {
        Self {
            block_number: None,
            block_timestamp: None,
            // is_finish: false,
            is_success: false,
            evs: None,
            exts: None,
        }
    }

    pub fn set_block_number(&mut self, num: u64) {
        self.block_number = Some(num)
    }

    pub fn set_block_timestamp(&mut self, timestamp: u64) {
        self.block_timestamp = Some(timestamp)
    }

    pub fn get_block_number_uncheck(&self) -> u64 {
        self.block_number
            .expect("cannot builder body, block_number is none")
    }

    pub fn get_block_timestamp_uncheck(&self) -> u64 {
        self.block_timestamp
            .expect("cannot builder body, block_timestamp is none")
    }

    pub(crate) async fn build(
        &mut self,
        online_body: BlockBody<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ) -> anyhow::Result<()> {
        let block_number = self.get_block_number_uncheck();
        for (i, online_ext) in online_body.extrinsics().iter().enumerate() {
            let online_ext = online_ext
                .map_err(|err| anyhow!("block #{} get extrinsic error: {}", block_number, err))?;

            let online_evs = online_ext.events().await.map_err(|err| {
                anyhow!(
                    "block #{} get extrinsic #{} events error: {}",
                    block_number,
                    online_ext.index(),
                    err
                )
            })?;

            let extrinsic_index = online_evs.extrinsic_index();
            let extrinsic_hash = online_evs.extrinsic_hash().as_bytes().to_vec();

            let mut events = vec![];
            for event in online_evs.iter() {
                let event = event.map_err(|err| {
                    anyhow!(
                        "block #{} get extrinsic #{} event error: {}",
                        block_number,
                        extrinsic_index,
                        err
                    )
                })?;
                events.push(event);
            }
            println!("extrisic_index = {} -> {}", extrinsic_index, events.len());

            self.computing_event_state(&events);
            self.add_extrinisc(&extrinsic_hash, &online_ext);
            self.add_events(extrinsic_index, &extrinsic_hash, &events);
        }

        Ok(())
    }

    pub fn add_extrinisc(
        &mut self,
        extrinsic_hash: &[u8],
        online_ext: &ExtrinsicDetails<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ) {
        let block_number = self.get_block_number_uncheck();
        let block_timestamp = self.get_block_timestamp_uncheck();

        let exts = match self.exts.as_mut() {
            None => {
                self.exts = Some(vec![]);
                self.exts.as_mut().unwrap()
            }
            Some(exts) => exts,
        };
        let call_params = online_ext
            .field_values()
            .and_then(|values| Ok(serde_json::to_value(values).map_or(None, |v| Some(v))))
            .map_or(None, |v| v);

        exts.push(polkadot_chain::Extrinsic {
            id: format!("{}-{}", block_number, online_ext.index()),
            block_number,
            block_timestamp,
            extrinsic_hash: extrinsic_hash.to_vec(),
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
            result: self.is_success,
        });
    }

    pub fn add_events(
        &mut self,
        extrinsic_index: u32,
        extrinsic_hash: &[u8],
        events: &[EventDetails<PolkadotConfig>],
    ) {
        let block_number = self.get_block_number_uncheck();
        let block_timestamp = self.get_block_timestamp_uncheck();
        let evs = match self.evs.as_mut() {
            None => {
                self.evs = Some(vec![]);
                self.evs.as_mut().unwrap()
            }
            Some(evs) => evs,
        };

        for online_ev in events.iter() {
            // let online_ev = online_ev.map_err(|err| {
            //     anyhow!(
            //         "block #{} get extrinsic #{} event error: {}",
            //         block_number,
            //         extrinsic_index,
            //         err
            //     )
            // })?;

            let values = online_ev
                .field_values()
                .and_then(|values| Ok(serde_json::to_value(values).map_or(None, |v| Some(v))))
                .map_or(None, |v| v);

            let phase_value = match online_ev.phase() {
                Phase::ApplyExtrinsic(_) => 0,
                Phase::Finalization => 1,
                Phase::Initialization => 2,
            };

            let mod_name = online_ev.pallet_name().to_string();
            let event_name = online_ev.variant_name().to_string();
            if &mod_name == <ExtrinsicSuccess as subxt::events::StaticEvent>::PALLET
                && &event_name == <ExtrinsicSuccess as subxt::events::StaticEvent>::EVENT
            {
            }

            evs.push(polkadot_chain::Event {
                id: format!("{}-{}", block_number, online_ev.index()),
                block_number,
                block_timestamp,
                mod_name: online_ev.pallet_name().to_string(),
                event_name: online_ev.variant_name().to_string(),
                event_index: online_ev.index(),
                phase: phase_value,
                extrinsic_hash: extrinsic_hash.to_vec(),
                extrinsic_index: extrinsic_index,
                extrinsic_id: format!("{}-{}", block_number, extrinsic_index),
                values,
            })
        }
    }

    pub fn finish(&mut self) -> polkadot_chain::Body {
        let evs = self.evs.take();
        let exts = self.exts.take();
        self.clear();
        polkadot_chain::Body {
            extrinsics: exts,
            events: evs,
        }
    }

    // pub fn block_is_finish(&self) -> bool {
    //     self.is_finish
    // }

    fn computing_event_state(&mut self, events: &[EventDetails<PolkadotConfig>]) {
        for event in events.iter() {
            let mod_name = event.pallet_name().to_string();
            let event_name = event.variant_name().to_string();
            if &mod_name == <ExtrinsicSuccess as subxt::events::StaticEvent>::PALLET
                && &event_name == <ExtrinsicSuccess as subxt::events::StaticEvent>::EVENT
            {
                self.is_success = true
            }
        }
    }

    fn clear(&mut self) {
        self.block_number = None;
        self.block_timestamp = None;
        // self.is_finish = false;
        self.is_success = false;
    }
}

pub struct StorageExtracter;

impl StorageExtracter {
    pub fn new() -> Self {
        StorageExtracter {}
    }

    pub async fn block_timestamp(
        &self,
        online_block: &OnlineBlock<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ) -> anyhow::Result<u64> {
        online_block
            .storage()
            .fetch(&default_runtime::storage().timestamp().now())
            .await?
            .map_or(Ok(0), |v| Ok(v))
    }

    pub async fn validator(
        &self,
        online_block: &OnlineBlock<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        // let active_era = online_block
        //     .storage()
        //     .fetch(&default_runtime::storage().para_scheduler().validator_groups() ))
        //     .await
        //     .unwrap()
        //     .unwrap();
        // let addr = &default_runtime::storage().session().validators();
        // let account = online_block.storage().fetch(addr).await.unwrap();

        // println!(
        //     "block # {} validator account = {:?}, active_era_index = {}",
        //     online_block.header().number,
        //     account.unwrap().len(),
        // );

        online_block
            .storage()
            .fetch(&default_runtime::storage().authorship().author())
            .await?
            .map_or(Ok(None), |v| Ok(Some(v.0.to_vec())))
    }
}

pub struct ConstantExtracter {
    constrants: ConstantsClient<PolkadotConfig, OnlineClient<PolkadotConfig>>,
}

impl ConstantExtracter {
    pub fn new(online_clinet: &OnlineClient<PolkadotConfig>) -> Self {
        Self {
            constrants: online_clinet.constants(),
        }
    }

    pub fn runtime_version(
        &self,
    ) -> anyhow::Result<default_runtime::runtime_types::sp_version::RuntimeVersion> {
        self.constrants
            .at(&default_runtime::constants().system().version())
            .map_err(|err| anyhow!("{}", err))
    }
}

pub struct BlockExtracter {
    online_client: OnlineClient<PolkadotConfig>,
    body_builder: BodyBuilder,
    storage: StorageExtracter,
    constant: ConstantExtracter,
}

impl BlockExtracter {
    pub fn new(online_client: &OnlineClient<PolkadotConfig>) -> Self {
        Self {
            online_client: online_client.clone(),
            storage: StorageExtracter::new(),
            constant: ConstantExtracter::new(&online_client),
            body_builder: BodyBuilder::new(),
        }
    }

    pub async fn extract(
        &mut self,
        online_block: OnlineBlock<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ) -> anyhow::Result<polkadot_chain::Block> {
        let block_timestamp = self.storage.block_timestamp(&online_block).await?;
        let runtime_version = self.constant.runtime_version()?;
        let validator = self.storage.validator(&online_block).await?;

        println!(
            "block # {} validator = {:?}",
            online_block.header().number,
            validator
        );
        // extract header
        let mut header = polkadot_chain::Header {
            block_number: online_block.header().number as u64,
            block_timestamp,
            block_hash: online_block.hash().as_bytes().to_vec(),
            parent_hash: online_block.header().parent_hash.as_bytes().to_vec(),
            extrinsics_root: online_block.header().extrinsics_root.as_bytes().to_vec(),
            state_root: online_block.header().state_root.as_bytes().to_vec(),
            is_finished: true,
            validator,
            spec_version: runtime_version.spec_version,
        };

        // extract logs
        let extraced_logs = Self::extract_logs(&header, &online_block.header().digest.logs);

        let online_body = online_block
            .body()
            .await
            .map_err(|err| anyhow!("block #{} get body error: {}", header.block_number, err))?;

        self.body_builder.set_block_number(header.block_number);
        self.body_builder
            .set_block_timestamp(header.block_timestamp);
        self.body_builder.build(online_body).await?;

        // if self.body_builder.block_is_finish() {
        //     header.is_finished = true;
        // }

        let body = self.body_builder.finish();
        Ok(polkadot_chain::Block {
            header,
            body,
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
        std::str::from_utf8(engine_id).map_or(None, |v| Some(v.to_string()))
    }
}
