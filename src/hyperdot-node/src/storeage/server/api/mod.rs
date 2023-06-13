pub mod route;
pub mod server;
mod v1;


pub use super::ServerArgs;
pub use server::ApiServer;

pub const API_ROOT_PATH: &'static str = "/apis";