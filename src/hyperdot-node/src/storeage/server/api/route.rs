
use axum::Router;
use tokio::sync::RwLock;

use std::collections::HashMap;
use std::sync::Arc;

use crate::storeage::ServerArgs;
use crate::storeage::StorageController;

use super::v1;

#[derive(Clone)]
pub struct Context {
    pub controllers: Arc<RwLock<HashMap<String, Arc<StorageController>>>>, // TODO: make as weak
}


pub fn init(args: &ServerArgs, ctx: Context) -> anyhow::Result<Router> {
    let mut router = Router::new();
    for chain_arg in args.chains.iter() {
        router = match chain_arg.chain.as_str() {
            "polkadot" =>  {
                v1::PolkadotRouteBuild::new().build(router)?
                // ...v2 if need
                // ...v3 if need
            },
            _ => unimplemented!(),
        };
    }

    Ok(router.with_state(ctx))
}
