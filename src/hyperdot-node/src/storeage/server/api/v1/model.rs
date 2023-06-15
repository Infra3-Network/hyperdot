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

pub use super::core;

pub mod query {
    use serde::Deserialize;
    use serde::Serialize;

    use super::core::ResponseMetdata;
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
        pub meta: ResponseMetdata,
        pub rows: PostgresRows,
    }
}
