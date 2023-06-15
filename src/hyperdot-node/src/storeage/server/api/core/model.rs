use std::collections::HashMap;

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct DataEngine {
    pub name: String,
}

#[derive(Clone, Serialize)]
pub struct ListDataEngine {
    pub engines: Vec<DataEngine>,
    /// The engine suppor chains.
    pub support_chains: HashMap<String, Vec<String>>,
}

impl ListDataEngine {
    pub fn is_support(&self, engine: &str) -> bool {
        self.support_chains.contains_key(engine)
    }

    pub fn is_support_chain(&self, engine: &str, chain: &str) -> bool {
        if !self.support_chains.contains_key(engine) {
            return false;
        }

        let chains = self.support_chains.get(engine).unwrap();
        println!("chains = {:?}", chains);

        for c in chains.iter() {
            if *c == *chain {
                return true;
            }
        }

        false
    }
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
