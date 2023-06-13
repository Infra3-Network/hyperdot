use std::collections::HashMap;
use std::sync::Arc;


use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use super::route;
use super::route::Context;

use crate::storeage::ServerArgs;
use crate::storeage::StorageController;
use crate::storeage::StorageControllerParams;

pub struct ApiServer {
    args: ServerArgs,
    http_serv_handle: Option<JoinHandle<anyhow::Result<()>>>,
}

impl ApiServer {
    pub async fn new(args: ServerArgs) -> anyhow::Result<Self> {
        Ok(Self {
            args,
            http_serv_handle: None,
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        if self.http_serv_handle.is_some() {
            return Err(anyhow::anyhow!(
                "🙅 the http server not empty, it's already running?"
            ));
        }

        let mut controllers = HashMap::new();
        for chain_arg in self.args.chains.iter() {
            let controller = StorageController::new(StorageControllerParams {
                chain: chain_arg.chain.clone(),
                storages: chain_arg.storage_urls.clone(),
            })
            .await?;
            let _ = controllers.insert(chain_arg.chain.clone(), Arc::new(controller));
        }
       

        let ctx = Context {
            controllers: Arc::new(RwLock::new(controllers)),
        };

        let app = route::init(&self.args, ctx)?;

        let addr = self.args.http_server_address.as_str().parse()?;

        let handle = tokio::spawn(async move {
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .map_err(|err| anyhow::anyhow!("{}", err))
        });

        tracing::info!(
            "🏃 http apiserver has been listend at {}",
            self.args.http_server_address,
        );
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
