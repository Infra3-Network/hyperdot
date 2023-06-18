use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::anyhow;
use hyperdot_common_config::Chain;
use hyperdot_common_config::PublicChain;
use hyperdot_common_config::StorageNodeConfig;
use subxt::Config;
use subxt::PolkadotConfig;
use subxt::SubstrateConfig;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;

use super::sync::PolkadotSyncer;
use super::sync::PolkadotSyncerHandle;
use super::Syncer;
use crate::streaming::speaker;
use crate::streaming::speaker::SpeakerController;
use crate::types::block::polkadot_chain;
use crate::types::polkadot;
use crate::types::rpc::WriteBlock;
use crate::types::rpc::WriteBlockRequest;

// use crate::types::WriteBlockRequest;
// use crate::speaker::SpeakerController;

// pub trait StreamFilter {
//     type Data;

//     fn filter(data: Self::Data) -> anyhow::Result<Self::Data>;
// }

pub struct OpenParams {
    pub child_urls: Vec<String>,
}

pub struct SpawnPolkadotParams {
    pub scheme: String,
    pub host: String,
    pub port: u16,
}

pub struct BlockStreamingHandle {
    task_handle: JoinHandle<anyhow::Result<()>>,
    speaker: Arc<SpeakerController>,
}

impl BlockStreamingHandle {
    pub async fn stopped(self) -> anyhow::Result<()> {
        self.task_handle.await?
    }
}

pub struct BlockStreaming<T>
where T: Config
{
    speaker: Arc<SpeakerController>,
    _m: PhantomData<T>,
}

impl<T> BlockStreaming<T>
where T: Config
{
    pub async fn open(params: &OpenParams) -> anyhow::Result<Self> {
        let speaker = SpeakerController::new(&params.child_urls).await?;
        Ok(Self {
            speaker: Arc::new(speaker),
            _m: PhantomData,
        })
    }
}

impl BlockStreaming<PolkadotConfig> {
    pub async fn spawn(self, params: &SpawnPolkadotParams) -> anyhow::Result<BlockStreamingHandle> {
        let (tx, mut rx) = unbounded_channel();
        // for u in params.block_sync_urls.iter() {
        //     let url = Url::parse(u)?;
        //     match url.scheme() {
        //         "polkadot" => {
        //             let query_pairs = url.query_pairs();
        //             // parse scheme + host + port
        //             let mut scheme = None;
        //             for query_pair in query_pairs {
        //                 let (key, value) = query_pair;
        //                 if key == "scheme" {
        //                     if value != "ws" && value != "wss" {
        //                         return Err(anyhow!("invalid scheme {} for {}", value, url.scheme()))
        //                     }
        //                     scheme = Some(value.into_owned());
        //                 }
        //             }
        //             if scheme.is_none() {
        //                 return Err(anyhow!("can not prased block sync url: missing scheme for {}", url.scheme()))
        //             }
        //             let host = match url.host_str() {
        //                 None => return Err(anyhow!("can not prased block sync url: missing host for {}", url.scheme())),
        //                 Some(host) => host,
        //             };
        //             let port = match url.port() {
        //                 None => return Err(anyhow!("can not prased block sync url: missing port for {}", url.scheme())),
        //                 Some(port) => port,
        //             };

        //             // create polkadot syncer
        //             let syncer_params = SyncerParams {
        //                 scheme: scheme.unwrap(),
        //                 host: host.to_string(),
        //                 port: port,
        //             };
        //             let syncer = Syncer::<PolkadotConfig>::new(syncer_params).await?;
        //             let _syncer_handle = syncer.spawn(tx.clone())?;
        //         },
        //         _ => return Err(anyhow!("unsupport syncer scheme: {}", url.scheme())),
        //     };
        // }

        let url = format!("{}://{}:{}", params.scheme, params.host, params.port); // TOOD: add mainnet etc..
        tracing::info!("🤳🏼 start sync polkadot at {}", url);
        let syncer = Syncer::<PolkadotConfig>::new(&url).await?;
        let _syncer_handle = syncer.spawn(tx.clone())?;

        let speaker = self.speaker.clone();
        let task_handle = tokio::spawn(async move {
            tracing::info!("🍓 streaming start");
            loop {
                let block_desc = match rx.recv().await {
                    None => {
                        tracing::error!("block channel closed");
                        return Err(anyhow!("channel of syncer closed"));
                    }
                    Some(block_desc) => block_desc,
                };

                tracing::info!(
                    "📞 streaming: recv block #{}",
                    block_desc.header.block_number
                );

                let request = WriteBlockRequest::<polkadot::Block> {
                    chain: "polkadot".to_string(),
                    blocks: vec![block_desc],
                };

                // let block_numbers = request.block_numbers();
                match self.speaker.write_block::<polkadot::Block>(request).await {
                    Err(err) => {
                        tracing::error!("write block failed"); // TODO: given more error info
                    }
                    Ok(_) => {
                        tracing::info!("🔚 write block success"); // TODO: given more info
                    }
                }
            }

            Ok(())
        });

        Ok(BlockStreamingHandle {
            task_handle,
            speaker,
        })
    }
}

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
    chain: Chain,
    storage_nodes: Vec<StorageNodeConfig>,
}

impl BlockStreaming2 {
    pub async fn spawn(
        chain: &Chain,
        storage_nodes: &Vec<StorageNodeConfig>,
        speaker_controller: Arc<crate::streaming::speaker::Controller>,
    ) -> anyhow::Result<BlockStreamingHandle2> {
        let bs = BlockStreaming2 {
            chain: chain.clone(),
            storage_nodes: storage_nodes.clone(),
        };
        match chain.kind {
            PublicChain::Ethereum => {
                unimplemented!("unsupport ethereum public chain streaming currently")
            }
            PublicChain::Polkadot => bs.spawn_polkadot_chain(speaker_controller.clone()).await,
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
                chain: self.chain.clone(),
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
                    tracing::error!("{}: write block #{} success", self.chain.name, block_number);
                }
            }
        }
    }
}
