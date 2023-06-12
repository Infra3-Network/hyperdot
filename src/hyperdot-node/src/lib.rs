// mod block;
// mod indexer;
// pub mod speaker;
pub mod api;
pub mod rpc;
pub mod runtime_api;
pub mod storeage;
pub mod streaming;
pub mod types;

/// The hyperdot supported chain currently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportChain {
    Substrate,
    Polkadot,
    Kusama,
}

impl From<&str> for SupportChain {
    fn from(value: &str) -> Self {
        match value {
            "substrate" => SupportChain::Substrate,
            "polkadot" => SupportChain::Polkadot,
            "kusama" => SupportChain::Kusama,
            _ => panic!("unkown support chain: {}", value),
        }
    }
}

impl SupportChain {
    /// Get lowercase string with enum.
    pub fn to_string(&self) -> String {
        match self {
            SupportChain::Substrate => "substrate".to_string(),
            SupportChain::Polkadot => "polkadot".to_string(),
            SupportChain::Kusama => "kusama".to_string(),
        }
    }
}
