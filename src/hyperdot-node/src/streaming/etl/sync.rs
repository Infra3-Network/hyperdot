use anyhow::anyhow;
use futures::StreamExt;
use hyperdot_core::config::ChainConfig;
use subxt::PolkadotConfig;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use url::Url;

use super::extracts::PolkadotBlockExtracter;
use crate::rpc::JseeRpcClient;
use crate::rpc::JseeRpcClientParams;
use crate::types::block::polkadot_chain;

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
