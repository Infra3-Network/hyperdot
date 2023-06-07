//! Url is specific scheme to parse and create speaker child

use url::Url;
use anyhow::anyhow;

use super::StorageOps;
use super::postgres::PostgresStorage;
use super::postgres::PostgresStorageParams;


/// Parse and create StorageOps.
// pub async fn parse_child(urls: &[String]) -> anyhow::Result<Vec<Box<dyn SpeakerChild>>> {
pub async fn parse_storage_ops(urls: &[String]) -> anyhow::Result<Vec<Box<dyn StorageOps>>> {
    let mut stores = vec![];
	 for u in urls.iter() {
        let url = Url::parse(u)?;
        let storage: Box<dyn StorageOps> = match url.scheme() {
            "postgres" => Box::new(internal_parse_postgres(&url).await?),
            _ => return Err(anyhow!("unsupport scheme: {}", url.scheme())),
        };
        stores.push(storage);
    }
	Ok(stores)
}

async fn internal_parse_postgres(url: &Url) -> anyhow::Result<PostgresStorage> {
    // parse host + port
    let host = match url.host_str() {
        None => return Err(anyhow!("🐘 parse postgres url error: missing host")),
        Some(host) => host.to_string(),
    };

    let port = match url.port() {
        None => return Err(anyhow!("🐘 parse postgres url error: missing port")),
        Some(port) => port,
    };

    let (mut user, mut password, mut dbname) = ( None, None, None );
    let query_pairs = url.query_pairs();
    for query_pair in query_pairs {
        let (key, value) = query_pair;
        if key == "user" {
           user = Some(value.into_owned())
        }
        else if key == "password" {
            password = Some(value.into_owned())
        }
        else if key == "dbname" {
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

    let params = PostgresStorageParams {
        host,
        port,
        user,
        password,
        dbname,
    };
    match PostgresStorage::new(params).await {
        Err(err) => {
            tracing::error!("⚠ PostgresStorage: create error: {}", err);
            return Err(anyhow!("{}", err))
        },
        Ok(pg) => Ok(pg),
    }
}

