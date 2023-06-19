use std::sync::Arc;

use hyperdot_core::config::StorageNodeConfig;

use super::api::ApiServer;
use super::jsonrpc::JsonRpcServer;
use crate::storeage::engine;

pub struct Server {
    jsonrpc_server: JsonRpcServer,
    api_server: ApiServer,
}

impl Server {
    pub async fn async_new(cfg: StorageNodeConfig) -> anyhow::Result<Self> {
        let engine_controller =
            Arc::new(engine::Controller::async_new(cfg.data_engines.clone()).await?);
        let jsonrpc_server =
            JsonRpcServer::async_new(cfg.clone(), engine_controller.clone()).await?;
        let api_server = ApiServer::new(cfg.clone(), engine_controller).await?;
        Ok(Self {
            jsonrpc_server,
            api_server,
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.jsonrpc_server.start().await?;
        self.api_server.start().await?;
        Ok(())
    }

    pub async fn stopped(self) -> anyhow::Result<()> {
        self.jsonrpc_server.stopped().await?;
        self.api_server.stopped().await?;
        Ok(())
    }
}
