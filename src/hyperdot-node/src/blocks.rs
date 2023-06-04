use anyhow::Result as AnyResult;
use futures::StreamExt;
use subxt::blocks::Block;
use subxt::blocks::ExtrinsicEvents;
use subxt::client::OnlineClientT;
use subxt::config::Header;
use subxt::events::Phase;
use subxt::Config;
use subxt::PolkadotConfig;
use subxt::SubstrateConfig;

use super::types::Event;
use super::types::EventDecode;
use super::types::EventPhase;
use super::types::WriteBlock;
use super::types::WriteBlockHeaderRequest;
use crate::indexer::BlockIndexer;
use crate::indexer::IndexerImpl;
use crate::runtime_api::polkadot;

#[derive(Default)]
struct ParsedEventPair {
    raw_events: Vec<Event>,
    decoded_events: Vec<EventDecode>,
}

impl ParsedEventPair {
    fn fill<C: Config>(header: &C::Header, events: &ExtrinsicEvents<C>) -> AnyResult<Self> {
        let block_hash = header.hash();
        let block_number = header.number().try_into()?;
        let extrinsic_hash = events.extrinsic_hash();
        let block_events = events.all_events_in_block();

        println!("Extrisic {:?}", extrinsic_hash);
        let mut event_pair = ParsedEventPair::default();
        for block_event in block_events.iter() {
            let block_event = block_event?;
            let ev_index = block_event.index();
            let ev_data = block_event.bytes().to_vec();
            let ev_phase = match block_event.phase() {
                Phase::Initialization => EventPhase::Initialization,
                Phase::ApplyExtrinsic(val) => EventPhase::ApplyExtrinsic(val),
                Phase::Finalization => EventPhase::Finalization,
            };
            let ev_pallet_index = block_event.pallet_index();
            let ev_pallet_name = block_event.pallet_name().to_string();

            let topics = block_event.topics();
            let (topic0, topic1, topic2, topic3, topic4) = match topics.len() {
                0 => (vec![], vec![], vec![], vec![], vec![]),
                1 => (topics[0].as_ref().to_vec(), vec![], vec![], vec![], vec![]),
                2 => (
                    topics[0].as_ref().to_vec(),
                    topics[1].as_ref().to_vec(),
                    vec![],
                    vec![],
                    vec![],
                ),
                3 => (
                    topics[0].as_ref().to_vec(),
                    topics[1].as_ref().to_vec(),
                    topics[2].as_ref().to_vec(),
                    vec![],
                    vec![],
                ),

                4 => (
                    topics[0].as_ref().to_vec(),
                    topics[1].as_ref().to_vec(),
                    topics[2].as_ref().to_vec(),
                    topics[3].as_ref().to_vec(),
                    vec![],
                ),
                5 | _ => (
                    topics[0].as_ref().to_vec(),
                    topics[1].as_ref().to_vec(),
                    topics[2].as_ref().to_vec(),
                    topics[3].as_ref().to_vec(),
                    topics[4].as_ref().to_vec(),
                ),
            };
            let raw_event = Event {
                block_hash: block_hash.as_ref().to_vec(),
                block_number,
                block_time: 0,
                extrinsic_hash: extrinsic_hash.as_ref().to_vec(),
                data: ev_data,
                index: ev_index,
                topic0,
                topic1,
                topic2,
                topic3,
                topic4,
                phase: ev_phase.clone(),
            };

            let decode_event = EventDecode {
                block_hash: block_hash.as_ref().to_vec(),
                block_number,
                block_time: 0,
                extrinsic_hash: extrinsic_hash.as_ref().to_vec(),
                index: ev_index,
                pallet_index: ev_pallet_index,
                pallet_name: ev_pallet_name,
                phase: ev_phase,
                // TODO signature
            };

            event_pair.raw_events.push(raw_event);
            event_pair.decoded_events.push(decode_event);
        }

        Ok(event_pair)
    }
}

#[async_trait::async_trait]
impl BlockIndexer<PolkadotConfig> for IndexerImpl<PolkadotConfig> {
    async fn sync_blocks(&self) -> AnyResult<()> {
        let online = self.client.get_online();
        // let _ = tokio::spawn(async move {
        //     let genesis_block = online.blocks().at(online.genesis_hash()).await.unwrap();
        //     let finalized_block_hash = online.rpc().finalized_head().await.unwrap();
        //     let finlized_block = online
        //         .rpc()
        //         .block(Some(finalized_block_hash))
        //         .await
        //         .unwrap()
        //         .unwrap()
        //         .block;
        //     println!(
        //         "Start sync gap {} -> {}",
        //         genesis_block.number(),
        //         finlized_block.header().number()
        //     );
        //     println!("Genesis Block #{}", genesis_block.number());
        //     println!("Genesis Hash: {:?}", online.genesis_hash());

        //     // for block_number in genesis_block.number()..=finlized_block.header.number() {
        //     //     let block_hash = online.rpc().block_hash(block_number).await.unwrap();
        //     //     // println!("Block #{}:", block_number);
        //     //     // println!("  Hash: {:?}", block_hash);
        //     // }
        // });

        let online = self.client.get_online();
        println!("Genesis Hash: {:?}", online.genesis_hash());
        let genesis_block = online.blocks().at(online.genesis_hash()).await?;
        let mut blocks_sub = online.blocks().subscribe_finalized().await?;
        // For each block, print a bunch of information about it:
        while let Some(block) = blocks_sub.next().await {
            let block = block?;
            let block_header = block.header();
            let block_hash = block.hash();
            let block_number = block.header().number().try_into()?;
            let req = WriteBlockHeaderRequest {
                block_number,
                block_hash: block_hash.as_ref().to_vec(),
                parent_hash: block_header.parent_hash.as_bytes().to_vec(),
                state_root: block_header.state_root.as_bytes().to_vec(),
                extrinsics_root: block_header.extrinsics_root.as_bytes().to_vec(),
            };
            // let _ = self.storage_channel.write_block(req).await?;

            // println!("Block #{block_number}:");
            // println!("  Hash: {:?}", block_hash);
            // println!("  Extrinsics:");

            // TODO: the subx hold ExtrinsicPartTypeIds(account, signature, extra) when
            // fetch block body, but the subx don't expose ExtrinsicPartTypeIds and parse
            // this type method, so, three solutions
            // 1. patch some code and push to upstream github with subx
            // 2. fetah metadata at block_hash, parse metadata balabla..., but it slowly.
            // 3. To intergrate the subxt into hyperdot, modify some...
            // let metadta = online.rpc().metadata_legacy(Some(block_hash)).await?;

            // Log each of the extrinsic with it's associated events:
            let body = block.body().await?;
            for ext in body.extrinsics().iter() {
                // let idx = ext.index();
                let ext = ext?;

                let events = ext.events().await?;
                // let bytes_hex = format!("0x{}", hex::encode(ext.bytes()));

                // See the API docs for more ways to decode extrinsics:
                // let decoded_ext = ext.as_root_extrinsic::<polkadot::Call>();

                // println!("    Extrinsic #{idx}:");
                // println!("      Bytes: {bytes_hex}");
                // println!("      Decoded: {decoded_ext:?}");
                // println!("      Events:");
                // let extrinsic_hash = events.extrinsic_hash();
                ParsedEventPair::fill(block_header, &events)?;
                let decoded_ext = ext.as_root_extrinsic::<polkadot::Call>()?;
                match decoded_ext {
                    polkadot::Call::Balances(call) => {
                        println!("{:?}", call);
                    },
                    _ => {},
                }

                // println!("      Decoded: {decoded_ext:?}");

                // let write_events = fill_events(block_header, &events)?;
                // for evt in events.iter() {
                //     let evt = evt?;

                //     let event_data = evt.bytes().to_vec();
                //     let event_index = evt.index();

                //     let pallet_name = evt.pallet_name();
                //     let event_name = evt.variant_name();
                //     let event_values = evt.field_values()?;
                //     println!("        {pallet_name}_{event_name}");
                //     println!("          {}", event_values);
                // }
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl BlockIndexer<SubstrateConfig> for IndexerImpl<SubstrateConfig> {
    async fn sync_blocks(&self) -> AnyResult<()> {
        todo!()
    }
}

#[tokio::test]
async fn it_works() {
    let index = IndexerImpl::<PolkadotConfig>::dev().await.unwrap();
    index.sync_blocks().await.unwrap();
}
