use anyhow::Result as AnyResult;
use subxt::Config;
use subxt::OnlineClient;
use subxt::PolkadotConfig;

use crate::jsee::JseeRpcClient;
use crate::jsee::JseeRpcClientParams;

/// Constrant for polkadot main network endpoints
pub const POKLADOT_MAINNET: &'static str = "wss://westend-rpc.polkadot.io:443";

/// Constrant for polkadot test network endpoints
pub const POKLADOT_TESTNET: &'static str = "wss://westend-rpc.polkadot.io:443";

/// Constrant for polkadot local network endpoints
pub const POKLADOT_LOCALNET: &'static str = "ws://localhost:9944";

/// The trait define rpc how does do what
#[async_trait::async_trait]
pub trait ConfiguredClient {
    type C: Config;

    /// New jsee client with url and params.
    async fn new_client(
        url: &str,
        params: &JseeRpcClientParams,
    ) -> AnyResult<JseeRpcClient<Self::C>> {
        JseeRpcClient::async_new(url, params).await
    }

    /// New default jsee client without params.`
    async fn defualt_client(url: &str) -> AnyResult<JseeRpcClient<Self::C>> {
        JseeRpcClient::async_new(url, &JseeRpcClientParams::default()).await
    }

    /// Checks if the client is connected to the target.
    fn is_connected(&self) -> bool;
}

/// Impl ConfiguredClient with PolkaConfig
pub struct PolkadotConfiguredClient {
    client: JseeRpcClient<PolkadotConfig>,
}

impl PolkadotConfiguredClient {
    pub async fn new(url: &str, params: &JseeRpcClientParams) -> AnyResult<Self> {
        let client = Self::new_client(url, params).await?;
        Ok(Self { client })
    }

    pub async fn testnet() -> AnyResult<Self> {
        let client = Self::defualt_client(POKLADOT_TESTNET).await?;
        Ok(Self { client })
    }

    pub async fn mainnet() -> AnyResult<Self> {
        let client = Self::defualt_client(POKLADOT_MAINNET).await?;
        Ok(Self { client })
    }

    pub async fn localnet() -> AnyResult<Self> {
        let client = Self::defualt_client(POKLADOT_LOCALNET).await?;
        Ok(Self { client })
    }

    /// Get online client.
    pub fn get_online(&self) -> OnlineClient<PolkadotConfig> {
        self.client.online.clone()
    }
}

#[async_trait::async_trait]
impl ConfiguredClient for PolkadotConfiguredClient {
    type C = PolkadotConfig;

    fn is_connected(&self) -> bool {
        self.client.inner.0.is_connected()
    }
}

#[tokio::test]
async fn test_polka_rpc_clinet_connection() {
    let cli = PolkadotConfiguredClient::testnet().await.unwrap();
    assert_eq!(cli.is_connected(), true)
}
