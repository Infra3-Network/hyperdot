use std::collections::HashMap;
use std::sync::Arc;

use http::Method;
use axum::Router;
use tokio::sync::RwLock;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;

use super::core;
use super::v1;
use crate::storeage::ServerArgs;
use crate::storeage::StorageController;

#[derive(Clone)]
pub struct Context {
    pub controllers: Arc<RwLock<HashMap<String, Arc<StorageController>>>>, // TODO: make as weak
}

pub fn init(args: &ServerArgs, ctx: Context) -> anyhow::Result<Router> {
   let cors = CorsLayer::new()
    // allow `GET` and `POST` when accessing the resource
    .allow_methods([Method::GET, Method::POST])
    // allow requests from any origin
    .allow_origin(Any);

    let mut router = Router::new();
    router = core::core::CoreRouteBuild::new().build(router)?;
    for chain_arg in args.chains.iter() {
        router = match chain_arg.chain.as_str() {
            "polkadot" => {
                v1::PolkadotRouteBuild::new().build(router)?
                // ...v2 if need
                // ...v3 if need
            }
            _ => unimplemented!(),
        };
    }

    Ok(router.with_state(ctx).layer(cors))
}
