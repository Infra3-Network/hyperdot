use hyperdot_core::config::StorageNodeConfig;

use crate::storeage::client::JsonRpcClientParams;
use crate::storeage::client::JsonRpcClinet;
use crate::types::rpc::WriteBlock;
// use crate::types::rpc::WriteBlockRequest;
use crate::types::rpc::WriteBlockResponse;

/// Represents a child speaker for JSON-RPC communication.
pub struct JsonRpcChild {
    name: String,
    node_cfg: StorageNodeConfig,
    remote_server_clinet: JsonRpcClinet,
}

impl JsonRpcChild {
    /// Opens a child speaker for JSON-RPC communication.
    pub async fn open(node_cfg: &StorageNodeConfig) -> anyhow::Result<Self> {
        let url = node_cfg
            .rpc
            .scheme
            .as_ref()
            .map_or(format!("ws://{}", node_cfg.rpc.url), |s| {
                format!("{}://{}", s, node_cfg.rpc.url)
            });
        let client = JsonRpcClinet::new(&url, JsonRpcClientParams::default())?;
        Ok(Self {
            name: format!("speaker_jsonrpc_child_{}", node_cfg.name),
            node_cfg: node_cfg.clone(),
            remote_server_clinet: client,
        })
    }

    pub async fn write_block(&self, request: WriteBlock) -> anyhow::Result<WriteBlockResponse> {
        self.remote_server_clinet.write_block2(request).await
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}
