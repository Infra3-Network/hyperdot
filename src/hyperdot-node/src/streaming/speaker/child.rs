use hyperdot_common_config::StorageNodeConfig;
use url::Url;

use crate::storeage::client::JsonRpcClientParams;
use crate::storeage::client::JsonRpcClinet;
use crate::types::rpc::WriteBlock;
use crate::types::rpc::WriteBlockRequest;
use crate::types::rpc::WriteBlockResponse;

// /// Represents a child speaker that implements the `SpeakerOps` trait.
// pub trait SpeakerChild: Send + Sync + SpeakerOps {
//     /// Get the speaker child name that deduplicating for controller.
//     fn name(&self) -> String;
// }

pub struct OpenSpeakerJsonRpcChildParams {
    /// Only either http or https, default is http
    pub scheme: Option<String>,
    /// The jsonrpc server host.
    pub host: String,
    /// The jsonrpc server port.
    pub port: Option<u16>,
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

/// Represents a child speaker for JSON-RPC communication.
pub struct SpeakerJsonRpcChild {
    name: String,
    remote_server_clinet: JsonRpcClinet,
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

    pub async fn write_block<T>(
        &self,
        request: WriteBlockRequest<T>,
    ) -> anyhow::Result<WriteBlockResponse>
    where
        T: Clone + Send + serde::Serialize,
    {
        self.remote_server_clinet.write_block(request).await
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

// #[async_trait::async_trait]
// impl SpeakerOps for SpeakerJsonRpcChild {
//     async fn write_block<T>(&self, request: WriteBlockRequest<T>) -> anyhow::Result<WriteBlockResponse>
//     where T: Clone + serde::Serialize{
//         self.remote_server_clinet.polkadot_write_block(request).await
//     }
// }

// impl SpeakerChild for SpeakerJsonRpcChild {
//     fn name(&self) -> String {
//         self.name.clone()
//     }
// }

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
