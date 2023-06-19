// pub mod core;
pub mod route;
pub mod server;
mod v1;

pub use server::ApiServer;

pub const API_ROOT_PATH: &'static str = "/apis";
