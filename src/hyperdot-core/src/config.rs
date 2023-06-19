use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use super::types::DataEngineInfo;
use crate::types::ChainKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRpcConfig {
    pub url: String,
    pub scheme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageApiServerConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNodeConfig {
    pub id: usize,
    pub name: String,
    pub rpc: StorageRpcConfig,
    pub apiserver: StorageApiServerConfig,
    pub data_engines: Vec<DataEngineInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub nodes: Vec<StorageNodeConfig>,
}

impl StorageConfig {
    pub fn get_node_config(&self, node_name: &str) -> Option<StorageNodeConfig> {
        self.nodes
            .iter()
            .find(|node| &node.name == node_name)
            .map(|node| node.clone())
    }
}

/// Representing different types of runtimes in the polkadot chain.
/// For Instance, substrate is probably the preferred choice for
/// most parallel chains, and there are some differences between
/// polkadot and kusama due to the addition of several extrinsics
/// to substrate.
///
/// # Note
///
/// If the runtime differs from the existing kind, you can
/// continue enumerating the type.
pub enum PolkadotRuntimeKind {
    Substrate,
    Polkadot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotRuntime {
    pub config: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub id: usize,
    pub name: String,
    pub url: String,
    pub kind: ChainKind,
    pub polkadot_runtime: Option<PolkadotRuntime>,
    pub storage_nodes: Option<Vec<String>>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub storage: StorageConfig,
    pub chain: Vec<ChainConfig>,
}

impl TryFrom<&Path> for Catalog {
    type Error = anyhow::Error;
    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        if p.extension().is_none() {
            return Err(anyhow::anyhow!("path extension invalid"));
        }

        // let yaml_ostr = OsStr::new("yaml");
        let ext = p.extension().unwrap().to_str().unwrap();
        match ext {
            "json" => {
                let file = std::fs::File::open(p)?;
                let reader = std::io::BufReader::new(file);
                match serde_json::from_reader(reader) {
                    Err(err) => Err(anyhow::anyhow!("{}", err)),
                    Ok(cl) => Ok(cl),
                }
            }
            _ => return Err(anyhow::anyhow!("{}: path extension unsupport", ext)),
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
                        "rpc": {
                            "url": "127.0.0.1:15722",
                            "scheme": "ws",
                        },
                        "apiserver": {
                            "url: "127.0.0.1:3000"
                        },
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
