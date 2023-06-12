use axum::routing::post;
use axum::Router;

use super::handle::polkadot_psql_query_block;
use super::handle::Context;

const API_VERSION: &'static str = "v1";

pub fn build(ctx: Context) -> Router {
    Router::new()
        .route(
            &polkadot_query_route_group("psql/block"),
            post(polkadot_psql_query_block),
        )
        .with_state(ctx)
}

#[inline]
fn polkadot_query_route_group(path: &str) -> String {
    format!("/apis/{}/{}/query/{}", API_VERSION, "polkadot", path)
}
