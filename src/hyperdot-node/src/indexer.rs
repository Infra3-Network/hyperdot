use anyhow::Result as AnyResult;
use hyperdot_common_rpc::PolkadotConfiguredClient;

use crate::storeage::PolkadotStorageChannel;
use crate::storeage::PolkadotStorageChannelParams;

#[async_trait::async_trait]
pub trait BlockIndexer {
    async fn sync_blocks(&self) -> AnyResult<()>;
}

#[async_trait::async_trait]
pub trait Indexer {
    type Block: BlockIndexer;
}

pub struct PolkadotIndexer {
    pub(crate) client: PolkadotConfiguredClient,
    pub(crate) storage_channel: PolkadotStorageChannel,
}

impl PolkadotIndexer {
    /// Create an indexer for the test net
    pub async fn testnet() -> AnyResult<Self> {
        let client = PolkadotConfiguredClient::testnet().await?;
        let storage_channel =
            PolkadotStorageChannel::new(PolkadotStorageChannelParams::dev()).await?;
        // let storage = PolkadotStorage::new().await?;
        Ok(Self {
            client,
            storage_channel,
        })
    }
}
