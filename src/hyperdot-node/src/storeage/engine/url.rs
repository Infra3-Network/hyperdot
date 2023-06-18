use std::any::Any;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;

use anyhow::anyhow;
use url::Url;


use super::postgres::PolkadotPostgresStorageImpl;
use super::postgres::PostgresStorage;
use super::postgres::PostgresStorageParams;
use crate::SupportChain;

/// Parse and create StorageOps.
// pub async fn parse_child(urls: &[String]) -> anyhow::Result<Vec<Box<dyn SpeakerChild>>> {
pub async fn parse_storage_ops(
    chain: &str,
    urls: &[String],
) -> anyhow::Result<HashMap<String, Arc<dyn Any + Send + Sync>>> {
    let mut ops = HashMap::new();
    let mut info = String::new();
    for u in urls.iter() {
        let support_chain = SupportChain::from(chain);
        write!(
            &mut info,
            "💻 initialized {} chain storages: [",
            support_chain.to_string()
        )?;
        let url = Url::parse(u)?;
        let _storage = match url.scheme() {
            "postgres" => {
                write!(&mut info, "🐘 postgres storage ")?;
                let ops_vptr = internal_parse_postgres(support_chain, &url).await?;
                ops.insert("postgres".to_string(), ops_vptr);
            }
            _ => return Err(anyhow!("unsupport scheme: {}", url.scheme())),
        };
    }
    write!(&mut info, "]")?;
    tracing::info!("{}", info);
    Ok(ops)
}

async fn internal_parse_postgres(
    chain: SupportChain,
    url: &Url,
) -> anyhow::Result<Arc<dyn Any + Send + Sync>> {
    // parse host + port
    let host = match url.host_str() {
        None => return Err(anyhow!("🐘 parse postgres url error: missing host")),
        Some(host) => host.to_string(),
    };

    let port = match url.port() {
        None => return Err(anyhow!("🐘 parse postgres url error: missing port")),
        Some(port) => port,
    };

    let (mut user, mut password, mut dbname) = (None, None, None);
    let query_pairs = url.query_pairs();
    for query_pair in query_pairs {
        let (key, value) = query_pair;
        if key == "user" {
            user = Some(value.into_owned())
        } else if key == "password" {
            password = Some(value.into_owned())
        } else if key == "dbname" {
            dbname = Some(value.into_owned())
        }
    }

    let user = match user {
        None => return Err(anyhow!("🐘 parse postgres url error: missing user")),
        Some(user) => user,
    };

    let password = match password {
        None => return Err(anyhow!("🐘 parse postgres url error: missing password")),
        Some(password) => password,
    };

    let dbname = match dbname {
        None => return Err(anyhow!("🐘 parse postgres url error: missing dbname")),
        Some(dbname) => dbname,
    };

    // create base pg with params.
    let base = match PostgresStorage::new(PostgresStorageParams {
        host,
        port,
        user,
        password,
        dbname,
    })
    .await
    {
        Err(err) => {
            tracing::error!("⚠ PostgresStorage: create error: {}", err);
            return Err(anyhow!("{}", err));
        }
        Ok(pg) => pg,
    };

    match chain {
        SupportChain::Substrate => unimplemented!(),
        SupportChain::Polkadot => {
            let polkadot_pg_impl = PolkadotPostgresStorageImpl { base };

            Ok(Arc::new(polkadot_pg_impl))
        }
        SupportChain::Kusama => unimplemented!(),
    }
}
