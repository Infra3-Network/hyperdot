use anyhow::anyhow;
use futures::Future;
use futures::StreamExt;
use hyper::body::HttpBody;
use hyperdot_core::config::ChainConfig;
use subxt::blocks::ExtrinsicDetails;
use subxt::blocks::ExtrinsicEvents;
use subxt::client::OfflineClientT;
use subxt::Config;
use subxt::OnlineClient;
use subxt::PolkadotConfig;
use subxt::SubstrateConfig;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use url::Url;

use super::handle::BlockHandleImpl;
use super::handle::BlockHandler;
use crate::rpc::JseeRpcClient;
use crate::rpc::JseeRpcClientParams;
use crate::types::block::polkadot_chain;
use crate::types::block::polkadot_chain::BlockGenericHeader;
use crate::types::polkadot;

pub struct CachedBody<T, C>
where
    T: Config,
    C: OfflineClientT<T>,
{
    pub details: Vec<ExtrinsicDetails<T, C>>,
    pub events: Vec<ExtrinsicEvents<T>>,
}

pub struct Syncer<T: Config> {
    client: JseeRpcClient<T>,
}

impl<T> Syncer<T>
where T: Config
{
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let client = JseeRpcClient::<T>::async_new(&url, &JseeRpcClientParams::default()).await?;
        Ok(Self { client })
    }
}

async fn sync_blocks_fut(
    online: OnlineClient<PolkadotConfig>,
    tx: UnboundedSender<polkadot::Block>,
) -> anyhow::Result<()> {
    let mut blocks_sub = online.blocks().subscribe_finalized().await?;
    while let Some(block) = blocks_sub.next().await {
        let block = block?;
        println!(
            "header: {}",
            serde_json::to_string_pretty(&block.header()).unwrap()
        );
        let body = block.body().await?;
        let mut extrinsic_details = vec![];
        let mut extrinsic_events = vec![];
        for ext in body.extrinsics().iter() {
            let extrinsic_detail = ext?;
            let events = extrinsic_detail.events().await?;
            extrinsic_details.push(extrinsic_detail);
            extrinsic_events.push(events);
        }
        let body = CachedBody::<PolkadotConfig, OnlineClient<PolkadotConfig>> {
            details: extrinsic_details,
            events: extrinsic_events,
        };
        let block_handle = BlockHandleImpl::<PolkadotConfig, OnlineClient<PolkadotConfig>>::new(
            block.header().clone(),
            body,
        );
        let block_desc = block_handle.handle()?;
        if let Err(_) = tx.send(block_desc) {
            panic!("block channel receiver closed")
        }
    }
    Ok(())
}

impl Syncer<PolkadotConfig> {
    pub fn spawn(self, tx: UnboundedSender<polkadot::Block>) -> anyhow::Result<()> {
        tracing::info!("🔥 spawnning polkadot syncer"); // TODO: add name for syncer
        let online = self.client.get_online();
        tokio::spawn(async move { sync_blocks_fut(online, tx).await });
        Ok(())
    }
}

pub struct PolkadotSyncerHandle {
    tg: JoinHandle<anyhow::Result<()>>,
}

impl PolkadotSyncerHandle {
    pub async fn stopped(self) -> anyhow::Result<()> {
        self.tg.await?
    }
}

pub struct PolkadotSyncer {
    tx: UnboundedSender<polkadot_chain::Block>,
}

impl PolkadotSyncer {
    pub async fn spawn_polkadot(
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
        let tg = tokio::spawn(async move { Self::polkadot_handle(tx, client.online).await });

        Ok(PolkadotSyncerHandle { tg })
    }

    pub async fn spawn_substrate(
        chain: &ChainConfig,
        tx: UnboundedSender<polkadot_chain::Block>,
    ) -> anyhow::Result<PolkadotSyncerHandle> {
        let client = JseeRpcClient::<SubstrateConfig>::async_new(
            &chain.url,
            &JseeRpcClientParams::default(),
        )
        .await?;
        let tg = tokio::spawn(async move { Self::substrate_handle(tx, client.online).await });

        Ok(PolkadotSyncerHandle { tg })
    }

    async fn substrate_handle(
        tx: UnboundedSender<polkadot_chain::Block>,
        client: OnlineClient<SubstrateConfig>,
    ) -> anyhow::Result<()> {
        let mut blocks_sub = client.blocks().subscribe_finalized().await?;
        while let Some(online_block) = blocks_sub.next().await {
            let online_block = match online_block {
                Err(err) => {
                    tracing::warn!("sub block body: {}", err);
                    continue;
                }
                Ok(b) => b,
            };

            let body: anyhow::Result<polkadot_chain::BlockGenericBody> = {
                let online_body = online_block.body().await?;
                let mut exts = vec![];
                for online_ext in online_body.extrinsics().iter() {
                    let online_ext = match online_ext {
                        Err(err) => {
                            tracing::warn!("handle block ext error: {}", err);
                            continue;
                        }
                        Ok(ext) => ext,
                    };

                    exts.push(Self::handle_extrinsic_detial(online_ext).await?);
                }

                Ok(polkadot_chain::BlockGenericBody { extrinsics: exts })
            };

            let body = (match body {
                Err(err) => {
                    tracing::warn!("can't handle block body error: {}", err);
                    None
                }
                Ok(body) => Some(body),
            });

            let block = polkadot_chain::Block {
                header: BlockGenericHeader {
                    block_number: online_block.header().number as u64,
                    block_hash: online_block.hash().as_bytes().to_vec(),
                    parent_hash: online_block.header().parent_hash.as_bytes().to_vec(),
                    extrinsics_root: online_block.header().extrinsics_root.as_bytes().to_vec(),
                    state_root: online_block.header().state_root.as_bytes().to_vec(),
                },
                body,
            };

            if let Err(err) = tx.send(block) {
                tracing::error!("streaming channel closed");
                break;
            }
        }

        Ok(())
    }

    async fn polkadot_handle(
        tx: UnboundedSender<polkadot_chain::Block>,
        client: OnlineClient<PolkadotConfig>,
    ) -> anyhow::Result<()> {
        let mut blocks_sub = client.blocks().subscribe_finalized().await?;
        while let Some(online_block) = blocks_sub.next().await {
            let online_block = match online_block {
                Err(err) => {
                    tracing::warn!("sub block body: {}", err);
                    continue;
                }
                Ok(b) => b,
            };

            let body: anyhow::Result<polkadot_chain::BlockGenericBody> = {
                let online_body = online_block.body().await?;
                let mut exts = vec![];
                for online_ext in online_body.extrinsics().iter() {
                    let online_ext = match online_ext {
                        Err(err) => {
                            tracing::warn!("handle block ext error: {}", err);
                            continue;
                        }
                        Ok(ext) => ext,
                    };

                    exts.push(Self::handle_extrinsic_detial(online_ext).await?);
                }

                Ok(polkadot_chain::BlockGenericBody { extrinsics: exts })
            };

            let body = (match body {
                Err(err) => {
                    tracing::warn!("can't handle block body error: {}", err);
                    None
                }
                Ok(body) => Some(body),
            });

            let block = polkadot_chain::Block {
                header: BlockGenericHeader {
                    block_number: online_block.header().number as u64,
                    block_hash: online_block.hash().as_bytes().to_vec(),
                    parent_hash: online_block.header().parent_hash.as_bytes().to_vec(),
                    extrinsics_root: online_block.header().extrinsics_root.as_bytes().to_vec(),
                    state_root: online_block.header().state_root.as_bytes().to_vec(),
                },
                body,
            };

            if let Err(err) = tx.send(block) {
                tracing::error!("streaming channel closed");
                break;
            }
        }

        Ok(())
    }

    async fn handle_extrinsic_detial<T: subxt::Config>(
        online_ext: subxt::blocks::ExtrinsicDetails<T, OnlineClient<T>>,
    ) -> anyhow::Result<polkadot_chain::ExtrinsicDetails> {
        let mut ext = polkadot_chain::ExtrinsicDetails::default();
        ext.index = online_ext.index();
        ext.is_signed = online_ext.is_signed();
        ext.pallet_index = online_ext.pallet_index();
        ext.pallet_name = online_ext
            .pallet_name()
            .map_or(None, |pname| Some(pname.to_string()));
        ext.variant_index = online_ext.variant_index();
        ext.variant_name = online_ext
            .variant_name()
            .map_or(None, |vname| Some(vname.to_string()));
        ext.signed_address = online_ext
            .address_bytes()
            .map_or(None, |bs| Some(bs.to_vec()));
        ext.bytes = online_ext.bytes().to_vec();
        ext.root_extrinsic_bytes = None; // TODO;
        ext.events = None; // TODO;
        Ok(ext)
    }
}
