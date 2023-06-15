pub mod core;
pub mod route;
pub mod server;
mod v1;

pub use server::ApiServer;

pub use super::ServerArgs;

pub const API_ROOT_PATH: &'static str = "/apis";
