use anyhow::Result as AnyResult;

use super::jsonrpc::client::JsonRpcClientParams;
use super::jsonrpc::client::JsonRpcClinet;
use crate::types::BlockHeaderDescribe;
use crate::types::WriteBlockHeaderResponse;

/// StorageChannel touch the storage and provide a
/// unified access point to the upper layer.
#[async_trait::async_trait]
pub trait StorageChannel {
    async fn write_block(
        &self,
        request: BlockHeaderDescribe,
    ) -> anyhow::Result<WriteBlockHeaderResponse>;
}

pub struct PolkadotStorageChannelParams {
    pub jsonrpc_server_address: String,
}

impl PolkadotStorageChannelParams {
    pub fn dev() -> Self {
        Self {
            jsonrpc_server_address: "http://127.0.0.1:15722".to_owned(),
        }
    }
}

pub struct PolkadotStorageChannel {
    jsonrpc_client: JsonRpcClinet,
}

impl PolkadotStorageChannel {
    pub async fn new(params: PolkadotStorageChannelParams) -> AnyResult<Self> {
        let client_params = JsonRpcClientParams {
            server_address: params.jsonrpc_server_address,
        };

        let jsonrpc_client = JsonRpcClinet::new(client_params)?;
        Ok(Self { jsonrpc_client })
    }
}

#[async_trait::async_trait]
impl StorageChannel for PolkadotStorageChannel {
    async fn write_block(
        &self,
        request: BlockHeaderDescribe,
    ) -> anyhow::Result<WriteBlockHeaderResponse> {
        self.jsonrpc_client.write_block_header(request).await
    }
}
