use super::ops::SpeakerOps;
use crate::storeage::jsonrpc::client::JsonRpcClientParams;
use crate::storeage::jsonrpc::client::JsonRpcClinet;
use crate::types::WriteBlockRequest;
use crate::types::WriteBlockResponse;

/// Represents a child speaker that implements the `SpeakerOps` trait.
pub trait SpeakerChild: Send + Sync + SpeakerOps {
    /// Get the speaker child name that deduplicating for controller.
    fn name(&self) -> String;
}

/// Represents a child speaker for JSON-RPC communication.
pub struct SpeakerJsonRpcChild {
    name: String,
    remote_server_clinet: JsonRpcClinet,
}

impl SpeakerJsonRpcChild {
    /// Opens a child speaker for JSON-RPC communication.
    pub fn open(url: &str) -> anyhow::Result<Self> {
        let client = JsonRpcClinet::new(url, JsonRpcClientParams::default())?;
        Ok(Self {
            name: String::from("speaker_jsonrpc_child"),
            remote_server_clinet: client,
        })
    }
}

#[async_trait::async_trait]
impl SpeakerOps for SpeakerJsonRpcChild {
    async fn write_block(&self, request: WriteBlockRequest) -> anyhow::Result<WriteBlockResponse> {
        self.remote_server_clinet.write_block(request).await
    }
}

impl SpeakerChild for SpeakerJsonRpcChild {
    fn name(&self) -> String {
        self.name.clone()
    }
}
