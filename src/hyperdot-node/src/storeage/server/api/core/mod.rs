pub mod core;
pub mod model;

pub use super::route;
pub use super::API_ROOT_PATH;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ResponseCode {
    Success,
    Error,
}

impl Default for ResponseCode {
    fn default() -> Self {
        Self::Success
    }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct ResponseMetdata {
    pub code: Option<ResponseCode>,
    pub reason: Option<String>,
}



impl ResponseMetdata {
	pub fn set_code(&mut self, code: ResponseCode) {
		self.code = Some(code)
	}

	pub fn set_reason(&mut self, reason: String) {
		self.reason = Some(reason)
	}
}
