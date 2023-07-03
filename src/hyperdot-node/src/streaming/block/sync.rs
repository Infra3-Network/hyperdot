use anyhow::anyhow;
// use futures::Future;
use futures::StreamExt;
// use hyper::body::HttpBody;
use hyperdot_core::config::ChainConfig;
// use subxt::blocks::ExtrinsicDetails;
// use subxt::blocks::ExtrinsicEvents;
// use subxt::client::OfflineClientT;
// use subxt::Config;
// use subxt::OnlineClient;
use subxt::PolkadotConfig;
// use subxt::SubstrateConfig;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use url::Url;

use super::extracts::PolkadotBlockExtracter;
// use super::handle::BlockHandleImpl;
// use super::handle::BlockHandler;
use crate::rpc::JseeRpcClient;
use crate::rpc::JseeRpcClientParams;
use crate::types::block::polkadot_chain;
// use crate::types::block::polkadot_chain::Header;
// use crate::types::polkadot;

// pub struct CachedBody<T, C>
// where
//     T: Config,
//     C: OfflineClientT<T>,
// {
//     pub details: Vec<ExtrinsicDetails<T, C>>,
//     pub events: Vec<ExtrinsicEvents<T>>,
// }

// pub struct Syncer<T: Config> {
//     client: JseeRpcClient<T>,
// }

// impl<T> Syncer<T>
// where T: Config
// {
//     pub async fn new(url: &str) -> anyhow::Result<Self> {
//         let client = JseeRpcClient::<T>::async_new(&url, &JseeRpcClientParams::default()).await?;
//         Ok(Self { client })
//     }
// }

// async fn sync_blocks_fut(
//     online: OnlineClient<PolkadotConfig>,
//     tx: UnboundedSender<polkadot::Block>,
// ) -> anyhow::Result<()> {
//     let mut blocks_sub = online.blocks().subscribe_finalized().await?;
//     while let Some(block) = blocks_sub.next().await {
//         let block = block?;
//         println!(
//             "header: {}",
//             serde_json::to_string_pretty(&block.header()).unwrap()
//         );
//         let body = block.body().await?;
//         let mut extrinsic_details = vec![];
//         let mut extrinsic_events = vec![];
//         for ext in body.extrinsics().iter() {
//             let extrinsic_detail = ext?;
//             let events = extrinsic_detail.events().await?;
//             extrinsic_details.push(extrinsic_detail);
//             extrinsic_events.push(events);
//         }
//         let body = CachedBody::<PolkadotConfig, OnlineClient<PolkadotConfig>> {
//             details: extrinsic_details,
//             events: extrinsic_events,
//         };
//         let block_handle = BlockHandleImpl::<PolkadotConfig, OnlineClient<PolkadotConfig>>::new(
//             block.header().clone(),
//             body,
//         );
//         let block_desc = block_handle.handle()?;
//         if let Err(_) = tx.send(block_desc) {
//             panic!("block channel receiver closed")
//         }
//     }
//     Ok(())
// }

// impl Syncer<PolkadotConfig> {
//     pub fn spawn(self, tx: UnboundedSender<polkadot::Block>) -> anyhow::Result<()> {
//         tracing::info!("🔥 spawnning polkadot syncer"); // TODO: add name for syncer
//         let online = self.client.get_online();
//         tokio::spawn(async move { sync_blocks_fut(online, tx).await });
//         Ok(())
//     }
// }

pub struct PolkadotSyncerHandle {
    tg: JoinHandle<anyhow::Result<()>>,
}

impl PolkadotSyncerHandle {
    pub async fn stopped(self) -> anyhow::Result<()> {
        self.tg.await?
    }
}

pub struct PolkadotSyncer {
    client: JseeRpcClient<PolkadotConfig>,
    block_extractor: PolkadotBlockExtracter,
}

impl PolkadotSyncer {
    pub async fn spawn(
        chain: &ChainConfig,
        tx: UnboundedSender<polkadot_chain::Block>,
    ) -> anyhow::Result<PolkadotSyncerHandle> {
        // TODO: move to util
        let url = Url::parse(&chain.url)
            .map_err(|err| anyhow!("{} parse url({}) error: {}", chain.name, chain.url, err))?;
        let client =
            JseeRpcClient::<PolkadotConfig>::async_new(&chain.url, &JseeRpcClientParams::default())
                .await
                .map_err(|err| anyhow!("{}: new rpc client error: {}", chain.name, err))?;

        let block_extractor = PolkadotBlockExtracter::new(client.get_online());
        let syncer = PolkadotSyncer {
            client,
            block_extractor,
        };

        let tg = tokio::spawn(async move { syncer.main_loop(tx).await });

        Ok(PolkadotSyncerHandle { tg })
    }

    async fn main_loop(mut self, tx: UnboundedSender<polkadot_chain::Block>) -> anyhow::Result<()> {
        let mut blocks_sub = self.client.online.blocks().subscribe_finalized().await?;
        while let Some(online_block) = blocks_sub.next().await {
            let online_block = match online_block {
                Err(err) => {
                    tracing::warn!("sub block body: {}", err);
                    continue;
                }
                Ok(b) => b,
            };

            let extracted_block = match self.block_extractor.extract(online_block).await {
                Err(err) => {
                    tracing::warn!("handle block ext error: {}", err);
                    continue;
                }
                Ok(b) => b,
            };

            if let Err(err) = tx.send(extracted_block) {
                tracing::error!("streaming channel closed");
                break;
            }
        }

        Ok(())
    }
}
