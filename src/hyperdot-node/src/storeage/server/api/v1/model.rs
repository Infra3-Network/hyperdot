pub mod support {
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
    pub struct ResponseMetadata {
        pub code: Option<ResponseCode>,
        pub message: Option<String>,
        pub reason: Option<String>,
    }

    impl ResponseMetadata {
        pub fn success(message: &str) -> Self {
            Self {
                code: Some(ResponseCode::Success),
                message: Some(message.to_string()),
                reason: None,
            }
        }
        pub fn set_code(&mut self, code: ResponseCode) {
            self.code = Some(code)
        }

        pub fn set_reason(&mut self, reason: String) {
            self.reason = Some(reason)
        }
    }
}

pub mod system {
    use std::collections::HashMap;

    use serde::Deserialize;
    use serde::Serialize;

    use super::support::ResponseMetadata;

    const DATA_ENGINE_JSON_CONFIG: &'static str = r#"
        {
            "postgres": {
                "support_chains": {
                    "polkadot": {
                        "name": "polkadot"
                    }
                }
            }
        }
    "#;

    const CHAIN_JSON_CONFIG: &'static str = r#"
        {
            "polkadot": {
                "name": "polkadot"
            }
        }"#;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Chain {
        pub name: String,
    }

    pub type ConfiguredChain = HashMap<String, Chain>;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct DataEngine {
        pub support_chains: ConfiguredChain,
    }

    pub type ConfiguredDataEngine = HashMap<String, DataEngine>;

    lazy_static::lazy_static! {
        pub static ref SUPPORT_CHAINS: ConfiguredChain = {
            serde_json::from_str(CHAIN_JSON_CONFIG).expect("initial SUPPORT_CHAINS failed")
        };

        pub static ref SUPPORT_DATA_ENGINES: ConfiguredDataEngine = {
            serde_json::from_str(DATA_ENGINE_JSON_CONFIG).expect("initial SUPPORT_DATA_ENGINES failed")
        };
    }

    #[test]
    fn test_init_config() {
        println!("{:?}", *SUPPORT_CHAINS);
        println!("{:?}", *SUPPORT_DATA_ENGINES);
    }

    #[derive(Default, Clone, Serialize, Deserialize)]
    pub struct ListDataEngineResponse {
        pub meta: ResponseMetadata,
        pub engines: ConfiguredDataEngine,
    }
}

pub mod polkadot {
    use std::collections::HashMap;

    use serde::Serialize;

    #[derive(Clone, Default, Serialize)]
    pub struct TableInfo {
        pub column_name: String,
        pub data_type: String,
    }

    #[derive(Clone, Default, Serialize)]
    pub struct ListPostgresTables {
        pub tables: Vec<String>,
        pub tables_info: HashMap<String, Vec<TableInfo>>,
    }
}

pub mod dataengine {
    use std::collections::HashMap;

    use serde::Deserialize;
    use serde::Serialize;

    use super::support::ResponseMetadata;
    use super::system::SUPPORT_DATA_ENGINES;

    #[derive(Clone, Default, Serialize)]
    pub struct PostgresTableInfo {
        pub column_name: String,
        pub data_type: String,
    }

    #[derive(Default, Serialize)]
    pub struct GetPostgresSchemeResponse {
        pub meta: ResponseMetadata,
        pub chain: String,
        pub tables: HashMap<String, Vec<PostgresTableInfo>>,
    }
}

pub use super::core;

pub mod query {
    use serde::Deserialize;
    use serde::Serialize;

    use super::support::ResponseMetadata;
    use crate::storeage::controller::postgres::PostgresRows;

    #[derive(Deserialize)]
    pub struct RunPostgresQueryPayload {
        /// which engine is queried.
        pub engine: String,
        /// Which chain to query.
        pub chain: String,
        /// What query it is.
        pub query: String,
    }

    #[derive(Clone, Default, Serialize)]
    pub struct PolkadotBlock {
        pub block_number: i64,
        pub block_hash: String,
        pub parent_hash: String,
        pub state_root: String,
        pub extrinsics_root: String,
    }

    #[derive(Default, Serialize)]
    pub struct RunPostgresQueryResponse {
        pub meta: ResponseMetadata,
        pub rows: PostgresRows,
    }
}
