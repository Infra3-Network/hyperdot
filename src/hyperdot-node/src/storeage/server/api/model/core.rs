use std::collections::HashMap;

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct DataEngine {
    pub name: String,
}

#[derive(Clone, Serialize)]
pub struct ListDataEngine {
    pub engines: Vec<DataEngine>,
    pub support_chains: HashMap<String, Vec<String>>,
}

impl Default for ListDataEngine {
    fn default() -> Self {
        let engines = vec![DataEngine {
            name: "postgres".to_string(),
        }];

        let mut support_chains = HashMap::new();
        support_chains.insert("postgres".to_string(), vec!["polkadot".to_string()]);

        Self {
            engines,
            support_chains,
        }
    }
}

lazy_static::lazy_static! {
    pub static ref SUPPORT_DATA_ENGINES: ListDataEngine = {
        ListDataEngine::default()
    };
}
