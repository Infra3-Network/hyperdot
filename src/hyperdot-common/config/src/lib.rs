use std::ffi::OsStr;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresDataEngineForChain {
    /// The chain id.
    pub id: usize,
    /// The chain alias name.
    pub name: String,
    /// The chain storage used connection name.
    pub use_connection: String,
    /// The chain database name.
    pub dbname: String,
    /// If true the storage enabled
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresDataEngineConnection {
    /// The postgres connection identify name.
    pub name: String,
    /// The postgres connection username.
    pub username: String,
    /// The postgres connection password.
    pub password: String,
    /// The postgres connection host.
    pub host: String,
    /// The postgres connection port.
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresDataEngine {
    /// The multiple connections for postgres. It colud
    /// are same database or multiple database.
    pub connections: Vec<PostgresDataEngineConnection>,
    /// The postgres support chains
    /// Note: connections[i] + support_chains[i].dbname = real connection.
    pub support_chains: Vec<PostgresDataEngineForChain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataEngine {
    pub postgres: Option<PostgresDataEngine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNode {
    pub id: usize,
    pub name: String,
    pub rpc_endpoint: String,
    pub http_endpoint: String,
    pub data_engines: Vec<DataEngine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub nodes: Vec<StorageNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotRuntime {
    pub config: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublicChain {
    Ethereum,
    Polkadot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    pub id: usize,
    pub name: String,
    pub url: String,
    pub kind: PublicChain,
    pub polkadot_runtime: Option<PolkadotRuntime>,
    pub storage_nodes: Option<Vec<String>>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub storage: Storage,
    pub chain: Vec<Chain>,
}

impl TryFrom<&Path> for Catalog {
    type Error = anyhow::Error;
    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        if p.extension().is_none() {
            return Err(anyhow::anyhow!("path extension invalid"))
        }

        // let yaml_ostr = OsStr::new("yaml");
        let ext = p.extension().unwrap().to_str().unwrap();
        match ext {
            "json" => {
                let file = std::fs::File::open(p)?;
                let reader = std::io::BufReader::new(file);
                match serde_json::from_reader(reader) {
                    Err(err) => Err(anyhow::anyhow!("{}", err)),
                    Ok(cl) => Ok(cl)
                }
            },
            _ => return Err(anyhow::anyhow!("{}: path extension unsupport", ext)) 
        }
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let config = r#"
        {
            "storage": {
                "nodes": [
                    {
                        "id": 1,
                        "name": "hyperdot-node",
                        "rpc_endpoint": "127.0.0.1:15722",
                        "http_endpoint": "127.0.0.1:3000",
                        "data_engines": [
                            {
                                "postgres": {
                                    "connections": [
                                        {
                                            "name": "pg1",
                                            "username": "postgres",
                                            "password": "postgres",
                                            "host": "127.0.0.1",
                                            "port": 5432
                                        },
                                        {
                                            "name": "pg2",
                                            "username": "postgres",
                                            "password": "postgres",
                                            "host": "127.0.0.1",
                                            "port": 5432
                                        }
                                    ],
                                    "support_chains": [
                                        {
                                            "id": 50,
                                            "name": "LocalSubstrate",
                                            "use_connection": "pg1",
                                            "dbname": "local_substrate",
                                            "enabled": true
                                        },
                                        {
                                            "id": 30,
                                            "name": "Westend",
                                            "use_connection": "pg2",
                                            "dbname": "westend",
                                            "enabled": false
                                        }
                                    ]
                                }
                            }
                        ]
                    }
                ]
            },
            "chain": [
                {
                    "id": 50,
                    "name": "LocalSubstrate",
                    "url": "ws://192.168.124.34:9944",
                    "enabled": true,
                    "kind": "Polkadot",
                    "polkadot_runtime": {
                        "config": "substrate"
                    },
                    
                    "storage_nodes": [
                        "hyperdot-node"
                    ]

                },
                {
                    "id": 10,
                    "name": "Polkadot",
                    "url": "wss://rpc.polkadot.io",
                    "enabled": false,
                    "kind": "Polkadot",
                    "polkadot_runtime": {
                        "config": "polkadot"
                    } 
                },
                {
                    "id": 20,
                    "name": "Kusama",
                    "url": "wss://kusama-rpc.polkadot.io",
                    "enabled": false,
                    "kind": "Polkadot",
                    "polkadot_runtime": {
                        "config": "polkadot"
                    } 
                },
                {
                    "id": 30,
                    "name": "Westend",
                    "url": "wss://westend-rpc.polkadot.io",
                    "enabled": false,
                    "kind": "Polkadot",
                    "polkadot_runtime": {
                        "config": "polkadot"
                    }   
                },
                {
                    "id": 40,
                    "name": "Westend",
                    "url": "wss://rococo-rpc.polkadot.io",
                    "enabled": false,
                    "kind": "Polkadot",
                    "polkadot_runtime": {
                        "config": "polkadot"
                    } 
                }
            ]
        }
        "#;

        let catalog: Catalog = serde_json::from_str(config).unwrap();
        println!("{:?}", catalog)
    }
}
