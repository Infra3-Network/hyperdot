use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result as AnyResult;
use hyperdot_core::config::StorageConfig;
use hyperdot_core::config::StorageNodeConfig;
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::server::ServerHandle;
use jsonrpsee::types::error::ErrorCode;
use jsonrpsee::types::error::ErrorObject;
use jsonrpsee::types::ResponsePayload;
use jsonrpsee::RpcModule;
use tracing::info;

use crate::storeage::engine;
use crate::types::rpc::WriteBlock;
use crate::types::rpc::WriteBlockResponse;

#[derive(Clone)]
pub struct JsonRpcServerContext {
    // controllers: Arc<RwLock<HashMap<String, Arc<StorageController>>>>,
    engine_controlelr: Arc<engine::Controller>, // TODO: make as weak
    cfg: StorageNodeConfig,
}

pub struct JsonRpcServer {
    // args: ServerArgs,
    cfg: StorageNodeConfig,
    engine_controller: Arc<engine::Controller>,
    // controllers: Arc<RwLock<HashMap<String, Arc<StorageController>>>>,
    handle: Option<ServerHandle>,
}

impl JsonRpcServer {
    pub async fn async_new(
        cfg: StorageNodeConfig,
        engine_controller: Arc<engine::Controller>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            cfg,
            engine_controller,
            handle: None,
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        if self.handle.is_some() {
            return Err(anyhow::anyhow!("server alreay started"));
        }
        let addr = self.cfg.rpc.url.parse::<SocketAddr>()?;
        let server = ServerBuilder::new().build(addr).await?;
        let ctx = JsonRpcServerContext {
            engine_controlelr: self.engine_controller.clone(),
            cfg: self.cfg.clone(),
        };
        let rpc_module = register_methods(ctx)?;
        info!("🌗 storage json-rpc server listening at {}", addr);
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
    let _ = rpc_module.register_async_method("write_block", |params, ctx| async move {
        info!("🍾 register write_block method");
        let req = match params.parse::<WriteBlock>() {
            Err(err) => return ResponsePayload::Error(err),
            Ok(req) => req,
        };

        let chain_name = req.chain.clone(); // let block_numbers = req.block_numbers();

        match ctx.engine_controlelr.write_block(req).await {
            Err(err) => {
                tracing::error!("⚠️ {}: write block error: {}", chain_name, err);
                ResponsePayload::Error(ErrorObject::from(ErrorCode::InternalError))
            }
            Ok(_) => {
                tracing::trace!("🌍 {}: write block success", chain_name);
                ResponsePayload::result(WriteBlockResponse {})
            }
        }
    })?;

    Ok(rpc_module)
}

// fn test() -> impl Fn(Params<'static>, Arc<JsonRpcServerContext>) -> Pin<Box<dyn Future<Output = ()>>>
// {
//     todo!()
// }

// fn make_async_adder() -> Box<
//     dyn Fn(
//         Params<'static>,
//         Arc<JsonRpcServerContext>,
//     ) -> Pin<
//         Box<
//             dyn Future<Output = ResponsePayload<'static, WriteBlockResponse>>
//                 + Clone
//                 + Send
//                 + Sync
//                 + 'static,
//         >,
//     >,
// > {
//     Box::new(move |params, ctx| {
//         Box::pin(async move {
//             info!("🍾 register write_block method");
//             let req = match params.parse::<WriteBlock>() {
//                 Err(err) => return ResponsePayload::Error(err),
//                 Ok(req) => req,
//             };

//             let chain_name = req.chain.name.clone(); // let block_numbers = req.block_numbers();

//             match ctx.engine_controlelr.write_block(req).await {
//                 Err(err) => {
//                     tracing::error!("⚠️ {}: write block error: {}", chain_name, err);
//                     ResponsePayload::Error(ErrorObject::from(ErrorCode::InternalError))
//                 }
//                 Ok(_) => {
//                     tracing::trace!("🌍 {}: write block success", chain_name);
//                     ResponsePayload::result(WriteBlockResponse {})
//                 }
//             }
//         })
//     })
// }
