use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::anyhow;
use hyperdot_core::config::ChainConfig;
use hyperdot_core::config::StorageNodeConfig;
use hyperdot_core::types::ChainKind;
use subxt::Config;
use subxt::PolkadotConfig;
use subxt::SubstrateConfig;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;

use super::sync::PolkadotSyncer;
use super::sync::PolkadotSyncerHandle;
// use super::Syncer;
use crate::streaming::speaker;
// use crate::streaming::speaker::SpeakerController;
use crate::types::block::polkadot_chain;
use crate::types::polkadot;
use crate::types::rpc::WriteBlock;

pub struct BlockStreamingHandle2 {
    sync_handle: PolkadotSyncerHandle,
    streaming_tg: JoinHandle<anyhow::Result<()>>,
}

impl BlockStreamingHandle2 {
    pub async fn stopped(self) -> anyhow::Result<()> {
        self.sync_handle.stopped().await?;
        self.streaming_tg.await?
    }
}

pub struct BlockStreaming2 {
    chain: ChainConfig,
    storage_nodes: Vec<StorageNodeConfig>,
}

impl BlockStreaming2 {
    pub async fn spawn(
        chain: &ChainConfig,
        storage_nodes: &Vec<StorageNodeConfig>,
        speaker_controller: Arc<crate::streaming::speaker::Controller>,
    ) -> anyhow::Result<BlockStreamingHandle2> {
        let bs = BlockStreaming2 {
            chain: chain.clone(),
            storage_nodes: storage_nodes.clone(),
        };
        match chain.kind {
            ChainKind::Ethereum => {
                unimplemented!("unsupport ethereum public chain streaming currently")
            }
            ChainKind::Polkadot => bs.spawn_polkadot_chain(speaker_controller.clone()).await,
        }
    }

    async fn spawn_polkadot_chain(
        self,
        speaker_controller: Arc<speaker::Controller>,
    ) -> anyhow::Result<BlockStreamingHandle2> {
        let runtime = match self.chain.polkadot_runtime.as_ref() {
            Some(runtime) => runtime.config.as_ref(),
            None => "substrate",
        };

        tracing::info!("🤔 {}: using {} runtime config", self.chain.name, runtime);
        let (tx, rx) = unbounded_channel();
        let sync_handle = match runtime {
            "polkadot" => PolkadotSyncer::spawn_polkadot(&self.chain, tx).await?,
            _ => PolkadotSyncer::spawn_substrate(&self.chain, tx).await?,
        };

        let tg =
            tokio::spawn(async move { self.polkadot_runtime_loop(rx, speaker_controller).await });
        return Ok(BlockStreamingHandle2 {
            streaming_tg: tg,
            sync_handle,
        });
    }

    async fn polkadot_runtime_loop(
        self,
        mut rx: UnboundedReceiver<polkadot_chain::Block>,
        speaker_controller: Arc<speaker::Controller>,
    ) -> anyhow::Result<()> {
        loop {
            let block = match rx.recv().await {
                None => {
                    tracing::error!("block channel closed");
                    return Err(anyhow!("channel of syncer closed"));
                }
                Some(block) => block,
            };

            let block_number = block.header.block_number;

            println!(
                "{} \n block #{}, size {}",
                self.chain.name,
                block.header.block_number,
                std::mem::size_of_val(&block.body)
            );

            let request = WriteBlock {
                chain: self.chain.name.clone(),
                chain_kind: self.chain.kind.clone(),
                polkadot_blocks: Some(vec![block]),
            };

            match speaker_controller.write_block(request).await {
                Err(err) => {
                    tracing::error!(
                        "{}: write block #{} error: {}",
                        self.chain.name,
                        block_number,
                        err
                    );
                    continue;
                }
                Ok(_) => {
                    tracing::info!("{}: write block #{} success", self.chain.name, block_number);
                }
            }
        }
    }
}
