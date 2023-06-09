// use std::collections::HashMap;
use std::sync::Arc;

use hyperdot_core::config::StorageNodeConfig;
// use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use super::route;
use super::route::Context;
use crate::storeage::engine;

pub struct ApiServer {
    cfg: StorageNodeConfig,
    engine_controller: Arc<engine::Controller>,
    http_serv_handle: Option<JoinHandle<anyhow::Result<()>>>,
}

impl ApiServer {
    pub async fn new(
        cfg: StorageNodeConfig,
        engine_controller: Arc<engine::Controller>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            cfg,
            engine_controller,
            http_serv_handle: None,
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        if self.http_serv_handle.is_some() {
            return Err(anyhow::anyhow!(
                "🙅 the http server not empty, it's already running?"
            ));
        }

        // let mut controllers = HashMap::new();
        // for chain_arg in self.args.chains.iter() {
        //     let controller = StorageController::new(StorageControllerParams {
        //         chain: chain_arg.chain.clone(),
        //         storages: chain_arg.storage_urls.clone(),
        //     })
        //     .await?;
        //     let _ = controllers.insert(chain_arg.chain.clone(), Arc::new(controller));
        // }

        let ctx = Context {
            cfg: self.cfg.clone(),
            engine_controller: self.engine_controller.clone(),
        };

        let app = route::init(ctx)?;

        let url = self.cfg.apiserver.url.as_str();
        let addr = url.parse()?;

        let handle = tokio::spawn(async move {
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .map_err(|err| anyhow::anyhow!("{}", err))
        });

        tracing::info!("🏃 http apiserver has been listend at {}", url);
        self.http_serv_handle = Some(handle);

        Ok(())
    }

    pub async fn stopped(self) -> anyhow::Result<()> {
        if let Some(serv) = self.http_serv_handle {
            serv.await?;
        }

        Ok(())
    }
}
