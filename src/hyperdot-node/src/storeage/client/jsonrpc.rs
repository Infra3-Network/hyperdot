use anyhow::Result as AnyResult;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::http_client::HttpClientBuilder;

use crate::types::rpc::WriteBlock;
use crate::types::rpc::WriteBlockRequest;
use crate::types::rpc::WriteBlockResponse;

pub struct JsonRpcClientParams {}
impl Default for JsonRpcClientParams {
    fn default() -> Self {
        Self {}
    }
}

pub struct JsonRpcClinet {
    params: JsonRpcClientParams,
    client: HttpClient,
}

impl JsonRpcClinet {
    pub fn new(url: &str, params: JsonRpcClientParams) -> AnyResult<Self> {
        let client = HttpClientBuilder::default().build(url)?;

        Ok(Self { params, client })
    }

    pub async fn write_block<T>(
        &self,
        request: WriteBlockRequest<T>,
    ) -> anyhow::Result<WriteBlockResponse>
    where
        T: Clone + Send + serde::Serialize,
    {
        match request.chain.as_str() {
            "polkadot" => {
                let response = self
                    .client
                    .request("/polkadot/write_block", request)
                    .await?;
                Ok(response)
            }
            _ => unimplemented!(),
        }
    }

    pub async fn write_block2(&self, request: WriteBlock) -> anyhow::Result<WriteBlockResponse> {
        let response = self.client.request("write_block", request).await?;
        Ok(response)
    }
}
