use std::sync::Arc;

use axum::Router;
use http::Method;
use hyperdot_core::config::StorageNodeConfig;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;

use super::v1;
use crate::storeage::engine;

#[derive(Clone)]
pub struct Context {
    pub cfg: StorageNodeConfig,
    pub engine_controller: Arc<engine::Controller>,
    // pub controllers: Arc<RwLock<HashMap<String, Arc<StorageController>>>>, // TODO: make as weak
}

pub fn init(ctx: Context) -> anyhow::Result<Router> {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        // allow requests from any origin
        .allow_origin(Any);

    let mut router = Router::new();
    router = v1::query::QueryRouteBuilder::new().build(router)?;
    router = v1::system::SystemRouteBuilder::new().build(router)?;
    router = v1::dataengine::DataEngineRouteBuilder::new().build(router)?;
    Ok(router.with_state(ctx).layer(cors))
}
