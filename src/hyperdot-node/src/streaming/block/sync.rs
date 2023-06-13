use futures::StreamExt;
use subxt::blocks::ExtrinsicDetails;
use subxt::blocks::ExtrinsicEvents;
use subxt::client::OfflineClientT;
use subxt::Config;
use subxt::OnlineClient;
use subxt::PolkadotConfig;
use tokio::sync::mpsc::UnboundedSender;

use super::handle::BlockHandleImpl;
use super::handle::BlockHandler;
use crate::rpc::JseeRpcClient;
use crate::rpc::JseeRpcClientParams;
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
