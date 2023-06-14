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

