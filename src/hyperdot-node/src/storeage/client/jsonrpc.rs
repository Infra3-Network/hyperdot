    use anyhow::Result as AnyResult;
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::http_client::HttpClient;
    use jsonrpsee::http_client::HttpClientBuilder;

    
    
    
    use crate::types::WriteBlockRequest;
    use crate::types::WriteBlockResponse;

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

        pub async fn write_block(
            &self,
            request: WriteBlockRequest,
        ) -> AnyResult<WriteBlockResponse> {
            let response = self.client.request("write_block", request).await?;
            Ok(response)
        }
    }
