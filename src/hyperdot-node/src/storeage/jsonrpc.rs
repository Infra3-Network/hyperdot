pub mod server {
    use std::net::SocketAddr;

    use std::sync::Arc;

    use anyhow::Result as AnyResult;
    use jsonrpsee::server::ServerBuilder;
    use jsonrpsee::server::ServerHandle;
    
    
    use jsonrpsee::types::ResponsePayload;
    use jsonrpsee::RpcModule;
    use tracing::info;

    
    use crate::types::WriteBlockRequest;
    use crate::types::WriteBlockResponse;
    use super::super::StorageController;
    use super::super::StorageControllerParams;

    pub struct JsonRpcServerParams {
        pub address: String,
        pub stores: Vec<String>,
    }

    impl JsonRpcServerParams {
        pub fn dev() -> Self {
            Self {
                address: String::from("127.0.0.1:15722"),
                stores: vec![
                    "postgres://hyperdot:5432?user=postgres&password=postgres&dbname=polkadot".to_string(),
                ],
            }
        }
    }

    #[derive(Clone)]
    pub struct JsonRpcServerContext {
        storage_controller: Arc<StorageController>, // TODO: make as weak
    }

    pub struct JsonRpcServer {
        storage_controller: Arc<StorageController>,
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
            let controller = StorageController::new(StorageControllerParams {
                store_urls: params.stores.clone(),
            }).await?;

            Ok(Self {
                storage_controller:  Arc::new(controller),
                params,
            })
        }

        pub async fn start(&self) -> AnyResult<JsonRpcServerHandle> {
            let addr = self.params.address.parse::<SocketAddr>()?;
            let server = ServerBuilder::new().build(addr).await?;
            let ctx = JsonRpcServerContext {
                storage_controller: self.storage_controller.clone(),
            };
            let rpc_module = register_methods(ctx)?;
            info!("🌗 json-rpc server listening at {}", addr);
            let handle = server.start(rpc_module)?;

            Ok(JsonRpcServerHandle { handle })
        }
    }

    pub fn register_methods(ctx: JsonRpcServerContext) -> AnyResult<RpcModule<JsonRpcServerContext>> {
        let mut rpc_module = RpcModule::new(ctx);
        info!("🍾 register write_block method");
        // rpc_module.register_async_method("write_block_header", |params, ctx| async move {
            //     let req = match params.parse::<BlockHeaderDescribe>() {
            //         Err(err) => return ResponsePayload::Error(err),
            //         Ok(req) => req,
            //     };

            //     info!("🌍 write block #{}", req.block_number);
            //     match ctx.write_block_header(&req).await {
            //         Err(err) => {
            //             tracing::error!("⚠️ write block #{} error: {}", req.block_number, err);
            //             return ResponsePayload::Error(ErrorObject::from(ErrorCode::InternalError));
            //         }
            //         Ok(_) => {}
            //     }
            //     ResponsePayload::result(WriteBlockHeaderResponse {})
            // })?;
        let _ = rpc_module.register_async_method("write_block", |params, ctx| async move {
                let req = match params.parse::<WriteBlockRequest>() {
                    Err(err) => return ResponsePayload::Error(err),
                    Ok(req) => req,
                };
                let block_numbers = req.block_numbers();
                info!("🌍 json-rpc server: recv blocks #{:?}", block_numbers);
                ctx.storage_controller.write_block(&req.blocks).await.unwrap();
                // match ctx.write_block_header(&req).await {
                //     Err(err) => {
                //         tracing::error!("⚠️ write block #{} error: {}", req.block_number, err);
                //         return ResponsePayload::Error(ErrorObject::from(ErrorCode::InternalError));
                //     }
                //     Ok(_) => {}
                // }

                ResponsePayload::result(WriteBlockResponse {})
            })?;

        Ok(rpc_module)
    }
}

pub mod client {
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
}
