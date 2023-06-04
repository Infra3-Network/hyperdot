pub mod server {
    use std::net::SocketAddr;
    use std::sync::Arc;

    use anyhow::Result as AnyResult;
    use jsonrpsee::server::ServerBuilder;
    use jsonrpsee::server::ServerHandle;
    use jsonrpsee::types::error::ErrorCode;
    use jsonrpsee::types::ErrorObject;
    use jsonrpsee::types::ResponsePayload;
    use jsonrpsee::RpcModule;
    use tracing::info;

    use super::super::StorageOps;
    use super::super::StorageOpsParams;
    use crate::types::WriteBlockHeaderRequest;
    use crate::types::WriteBlockHeaderResponse;

    pub struct JsonRpcServerParams {
        pub address: String,
        pub storage_ops_params: StorageOpsParams,
    }

    impl JsonRpcServerParams {
        pub fn dev() -> Self {
            Self {
            address: String::from("127.0.0.1:15722"),
            storage_ops_params: StorageOpsParams { postgres_addr: "host=127.0.0.1 port=15721 user=noisepage_user password=noisepage_pass dbname=polkadot".to_string() }
        }
        }
    }

    pub struct JsonRpcServer {
        storage_ops: Arc<StorageOps>,
        params: JsonRpcServerParams,
    }

    pub struct JsonRpcServerHandle {
        handle: ServerHandle,
    }

    impl JsonRpcServerHandle {
        pub async fn stopped(self) -> AnyResult<()> {
            self.handle.stopped().await;
            Ok(())
        }
    }

    impl JsonRpcServer {
        pub async fn new(params: JsonRpcServerParams) -> AnyResult<Self> {
            let storage_ops = Arc::new(StorageOps::new(params.storage_ops_params.clone()).await?);

            Ok(Self {
                storage_ops,
                params,
            })
        }

        pub async fn start(&self) -> AnyResult<JsonRpcServerHandle> {
            let addr = self.params.address.parse::<SocketAddr>()?;
            let server = ServerBuilder::new().build(addr).await?;
            let rpc_module = register_methods(self.storage_ops.clone())?;
            info!("🌗 json-rpc server listening at {}", addr);
            let handle = server.start(rpc_module)?;

            Ok(JsonRpcServerHandle { handle })
        }
    }

    pub fn register_methods(ops: Arc<StorageOps>) -> AnyResult<RpcModule<Arc<StorageOps>>> {
        let mut rpc_module = RpcModule::new(ops);
        info!("🍾 register write_block_header method");
        let _ =
            rpc_module.register_async_method("write_block_header", |params, ctx| async move {
                let req = match params.parse::<WriteBlockHeaderRequest>() {
                    Err(err) => return ResponsePayload::Error(err),
                    Ok(req) => req,
                };

                info!("🌍 write block #{}", req.block_number);
                match ctx.write_block_header(&req).await {
                    Err(err) => {
                        tracing::error!("⚠️ write block #{} error: {}", req.block_number, err);
                        return ResponsePayload::Error(ErrorObject::from(ErrorCode::InternalError));
                    }
                    Ok(_) => {}
                }
                ResponsePayload::result(WriteBlockHeaderResponse {})
            })?;

        Ok(rpc_module)
    }
}

pub mod client {
    use anyhow::Result as AnyResult;
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::http_client::HttpClient;
    use jsonrpsee::http_client::HttpClientBuilder;

    use crate::types::WriteBlockHeaderRequest;
    use crate::types::WriteBlockHeaderResponse;

    pub struct JsonRpcClientParams {
        pub server_address: String,
    }

    pub struct JsonRpcClinet {
        params: JsonRpcClientParams,
        client: HttpClient,
    }

    impl JsonRpcClinet {
        pub fn new(params: JsonRpcClientParams) -> AnyResult<Self> {
            let client = HttpClientBuilder::default().build(&params.server_address)?;

            Ok(Self { params, client })
        }

        pub async fn write_block_header(
            &self,
            request: WriteBlockHeaderRequest,
        ) -> AnyResult<WriteBlockHeaderResponse> {
            let response = self
                .client
                .request("write_block_header", request)
                .await
                .unwrap();
            Ok(response)
        }
    }
}
