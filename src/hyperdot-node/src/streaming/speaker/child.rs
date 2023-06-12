use super::ops::SpeakerOps;
use crate::storeage::client::JsonRpcClientParams;
use crate::storeage::client::JsonRpcClinet;
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

pub struct OpenSpeakerJsonRpcChildParams {
    /// Only either http or https, default is http
    pub scheme: Option<String>,
    /// The jsonrpc server host.
    pub host: String,
    /// The jsonrpc server port.
    pub port: Option<u16>,
}

impl Default for OpenSpeakerJsonRpcChildParams {
    fn default() -> Self {
        Self {
            scheme: Some("http".to_string()),
            host: "127.0.0.1".to_string(),
            port: Some(15726),
        }
    }
}

impl OpenSpeakerJsonRpcChildParams {
    pub fn to_url(&self) -> String {
        format!(
            "{}://{}:{}",
            self.scheme.as_ref().map_or("http", |s| s.as_str()),
            self.host,
            self.port.map_or(15722, |p| p)
        )
    }
}

impl SpeakerJsonRpcChild {
    /// Opens a child speaker for JSON-RPC communication.
    pub fn open(params: OpenSpeakerJsonRpcChildParams) -> anyhow::Result<Self> {
        let url = format!(
            "{}://{}:{}",
            params.scheme.as_ref().map_or("http", |s| s.as_str()),
            params.host,
            params.port.map_or(15722, |p| p)
        );
        let client = JsonRpcClinet::new(&url, JsonRpcClientParams::default())?;
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
