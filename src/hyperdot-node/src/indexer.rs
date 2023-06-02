use anyhow::Result as AnyResult;
use hyperdot_common_rpc::JseeRpcClient;
use hyperdot_common_rpc::JseeRpcClientParams;
use hyperdot_common_rpc::POKLADOT_MAINNET;
use hyperdot_common_rpc::POKLADOT_TESTNET;
use subxt::Config;

// use hyperdot_common_rpc::PolkadotConfiguredClient;
use crate::storeage::PolkadotStorageChannel;
use crate::storeage::PolkadotStorageChannelParams;

#[async_trait::async_trait]
pub trait BlockIndexer<C: Config> {
    async fn sync_blocks(&self) -> AnyResult<()>;
}

#[async_trait::async_trait]
pub trait Indexer<C: Config> {
    type Block: BlockIndexer<C>;
}

pub struct IndexerImpl<C: Config>
where
    Self: 'static
{
    pub(crate) client: JseeRpcClient<C>,
    // pub(crate) storage_channel: PolkadotStorageChannel,
}

// pub struct PolkadotIndexer {
//     pub(crate) client: PolkadotConfiguredClient,
//     pub(crate) storage_channel: PolkadotStorageChannel,
// }

impl<C:Config> IndexerImpl<C> {
    /// Create an indexer for the test net
    pub async fn dev() -> AnyResult<Self> {
        let client = JseeRpcClient::<C>::async_new(POKLADOT_MAINNET, &JseeRpcClientParams::default()).await?;
        // let client = PolkadotConfiguredClient::testnet().await?;
        // let storage_channel =
            // PolkadotStorageChannel::new(PolkadotStorageChannelParams::dev()).await?;
        // let storage = PolkadotStorage::new().await?;
        Ok(Self {
            client,
            // storage_channel,
        })
    }
}
