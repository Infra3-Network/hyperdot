use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result as AnyResult;
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::server::ServerHandle;
use jsonrpsee::types::ResponsePayload;
use jsonrpsee::types::error::ErrorObject;
use jsonrpsee::types::error::ErrorCode;

use jsonrpsee::RpcModule;
use tracing::info;
use tokio::sync::RwLock;

use super::ServerArgs;
use crate::storeage::StorageController;
use crate::storeage::StorageControllerParams;
use crate::types::rpc::WriteBlockRequest;
use crate::types::rpc::WriteBlockResponse;

#[derive(Clone)]
pub struct JsonRpcServerContext {
    controllers: Arc<RwLock<HashMap<String, Arc<StorageController>>>>, // TODO: make as weak
}

pub struct JsonRpcServer {
    args: ServerArgs,
    controllers: Arc<RwLock<HashMap<String, Arc<StorageController>>>>, 
    handle: Option<ServerHandle>,   

}


impl JsonRpcServer {
    pub async fn new(args: ServerArgs) -> anyhow::Result<Self> {
        let mut controllers = HashMap::new();
        for chain_arg in args.chains.iter() {
            let controller = StorageController::new(StorageControllerParams {
                chain: chain_arg.chain.clone(),
                storages: chain_arg.storage_urls.clone(),
            })
            .await?;
            let _ = controllers.insert(chain_arg.chain.clone(), Arc::new(controller));
        }
       

        Ok(Self {
            args,
            controllers: Arc::new(RwLock::new(controllers)),
            handle: None,
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        if self.handle.is_some() {
            return Err(anyhow::anyhow!("server alreay started"));
        }   
        let addr = self.args.jsonrpc_server_address.parse::<SocketAddr>()?;
        let server = ServerBuilder::new().build(addr).await?;
        let ctx = JsonRpcServerContext {
            controllers: self.controllers.clone(),
        };
        let rpc_module = register_methods(ctx)?;
        info!(
            "🌗 storage json-rpc server listening at {}",
            addr
        );
        self.handle = Some(server.start(rpc_module)?);

        Ok(())
    }

    pub async fn stopped(self) -> anyhow::Result<()> {
        if let Some(handle) = self.handle {
            handle.stopped().await;
        }
        Ok(())
    }

}

pub fn register_methods(ctx: JsonRpcServerContext) -> AnyResult<RpcModule<JsonRpcServerContext>> {
    let mut rpc_module = RpcModule::new(ctx);
    info!("🍾 register /polkadot/write_block method");
   
    let _ = rpc_module.register_async_method("/polkadot/write_block", |params, ctx| async move {
        let req = match params.parse::<WriteBlockRequest::<crate::types::polkadot::Block>>() {
            Err(err) => return ResponsePayload::Error(err),
            Ok(req) => req,
        };
        // let block_numbers = req.block_numbers();
        // info!("🌍 json-rpc server: recv blocks #{:?}", block_numbers);
        let controller = {
            let controllers = ctx.controllers.read().await;
            match controllers.get("polkadot") {
                None => return ResponsePayload::Error(ErrorObject::owned::<()>(
                    ErrorCode::InternalError.code(),
                    "polkadot stroage controller not found",
                    None,
                )),
                Some(controller) => controller.clone(),
            }
        };
       
        controller
            .write_polkadot_block(&req)
            .await
            .unwrap();
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
