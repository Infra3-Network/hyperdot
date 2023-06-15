mod polkadot;
pub mod query;

pub mod model;
pub use polkadot::PolkadotRouteBuild;

pub use super::core;
pub use super::route;
pub use super::API_ROOT_PATH;

const API_VERSION: &'static str = "v1";
