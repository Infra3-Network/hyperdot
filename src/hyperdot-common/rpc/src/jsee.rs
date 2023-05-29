use std::sync::Arc;
use std::time::Duration;

use anyhow::Result as AnyResult;
use futures::StreamExt;
use futures::TryStreamExt;
use jsonrpsee::client_transport::ws::WsTransportClientBuilder;
use jsonrpsee::core::client::Client;
use jsonrpsee::core::client::ClientBuilder;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::client::SubscriptionClientT;
use jsonrpsee::core::client::SubscriptionKind;
use jsonrpsee::core::traits::ToRpcParams;
use jsonrpsee::types::SubscriptionId;
use serde_json;
use serde_json::value::RawValue;
use subxt::error::RpcError;
use subxt::rpc::RpcClientT;
use subxt::rpc::RpcSubscription;
use subxt::Config;
use subxt::OnlineClient;

/// Constrant for polkadot main network endpoints
pub const POKLADOT_MAINNET: &'static str = "wss://westend-rpc.polkadot.io:443";

/// Constrant for polkadot test network endpoints
pub const POKLADOT_TESTNET: &'static str = "wss://westend-rpc.polkadot.io:443";

struct WrapJsonrpseeClient(Client);

impl WrapJsonrpseeClient {
    /// New client with url and params.
    async fn async_new(url: &str, params: &JseeRpcClientParams) -> AnyResult<Self> {
        let uri = url.parse()?;
        let (tx, rx) = WsTransportClientBuilder::default()
            .connection_timeout(params.connection_timeout.clone())
            .build(uri)
            .await?;
        let client = ClientBuilder::default()
            .max_concurrent_requests(params.max_concurrent_requests)
            .max_buffer_capacity_per_subscription(params.max_buffer_capacity_per_subscription)
            .build_with_tokio(tx, rx);
        Ok(WrapJsonrpseeClient(client))
    }
}

struct Params(Option<Box<RawValue>>);

impl ToRpcParams for Params {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, jsonrpsee::core::Error> {
        Ok(self.0)
    }
}

impl RpcClientT for WrapJsonrpseeClient {
    fn request_raw<'a>(
        &'a self,
        method: &'a str,
        params: Option<Box<RawValue>>,
    ) -> subxt::rpc::RpcFuture<'a, Box<RawValue>> {
        Box::pin(async move {
            let res = ClientT::request(&self.0, method, Params(params))
                .await
                .map_err(|e| RpcError::ClientError(Box::new(e)))?;
            Ok(res)
        })
    }

    fn subscribe_raw<'a>(
        &'a self,
        sub: &'a str,
        params: Option<Box<RawValue>>,
        unsub: &'a str,
    ) -> subxt::rpc::RpcFuture<'a, subxt::rpc::RpcSubscription> {
        Box::pin(async move {
            let stream = self
                .0
                .subscribe::<Box<RawValue>, _>(sub, Params(params), unsub)
                .await
                .map_err(|e| RpcError::ClientError(Box::new(e)))?;

            let id = match stream.kind() {
                SubscriptionKind::Subscription(SubscriptionId::Str(id)) => {
                    Some(id.clone().into_owned())
                }
                _ => None,
            };

            let stream = stream
                .map_err(|e| RpcError::ClientError(Box::new(e)))
                .boxed();
            Ok(RpcSubscription { stream, id })
        })
    }
}

/// Parameters needed to construct JseeRpcClient.
pub struct JseeRpcClientParams {
    /// Timeout for the connection.
    pub connection_timeout: Duration,

    /// The max concurrent requests (default is 256).
    pub max_concurrent_requests: usize,

    /// The max buffer capacity for each subscription; when the capacity is exceeded the subscription
    /// will be dropped (default is 1024).
    ///
    /// # Panics
    ///
    /// This function panics if `max` is 0
    pub max_buffer_capacity_per_subscription: usize,
}

impl Default for JseeRpcClientParams {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(1),
            max_concurrent_requests: 256,
            max_buffer_capacity_per_subscription: 1024,
        }
    }
}

/// Wrap jsonrpsee
#[derive(Clone)]
pub struct JseeRpcClient<C: Config> {
    inner: Arc<WrapJsonrpseeClient>,
    online: OnlineClient<C>,
}

impl<C> JseeRpcClient<C>
where C: Config
{
    /// New client with polkadot test network.
    pub async fn with_polkadot_testnet() -> AnyResult<Self> {
        Self::async_new(POKLADOT_TESTNET, &JseeRpcClientParams::default()).await
    }

    /// New client with polkadot main network.
    pub async fn with_polkadot_mainnet() -> AnyResult<Self> {
        Self::async_new(POKLADOT_MAINNET, &JseeRpcClientParams::default()).await
    }

    /// New client with default url ("ws://127.0.0.1:9944").
    pub async fn with_default_url() -> AnyResult<Self> {
        Self::async_new("ws://127.0.0.1:9944", &JseeRpcClientParams::default()).await
    }

    /// New client with url and params.
    pub async fn async_new(url: &str, params: &JseeRpcClientParams) -> AnyResult<Self> {
        let inner = Arc::new(WrapJsonrpseeClient::async_new(url, params).await?);
        let online = OnlineClient::<C>::from_rpc_client(inner.clone()).await?;
        Ok(Self {
            // inner: client,
            inner,
            online,
        })
    }

    /// Checks if the client is connected to the target.
    #[inline]
    pub fn is_connected(&self) -> bool {
        self.inner.0.is_connected()
    }

    /// Get online client.
    #[inline]
    pub fn get_online(&self) -> OnlineClient<C> {
        self.online.clone()
    }
}

#[tokio::test]
async fn test_jsee_rpc_clinet_connection() {
    use subxt::config::PolkadotConfig;
    let cli = JseeRpcClient::<PolkadotConfig>::with_polkadot_testnet()
        .await
        .unwrap();
    assert_eq!(cli.is_connected(), true)
}
