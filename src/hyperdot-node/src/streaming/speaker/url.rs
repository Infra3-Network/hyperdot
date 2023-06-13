//! Url is specific scheme to parse and create speaker child

use url::Url;
use anyhow::anyhow;

// use super::SpeakerChild;
use super::child::SpeakerJsonRpcChild;

/// Parse and create SpeakerChilds.
pub async fn parse_childs(urls: &[String]) -> anyhow::Result<Vec<SpeakerJsonRpcChild>> {
	let mut childs = vec![];
	 for u in urls.iter() {
        let url = Url::parse(u)?;
        let child = match url.scheme() {
            "jsonrpc" => internal_parse_jsonrpc_child(&url)?,
            _ => return Err(anyhow!("unsupport scheme: {}", url.scheme())),
        };
        childs.push(child);
    }
	Ok(childs)
}

fn internal_parse_jsonrpc_child(url: &Url) -> anyhow::Result<SpeakerJsonRpcChild> {
	let mut params = super::child::OpenSpeakerJsonRpcChildParams::default();

    // parse host + port
    match url.host_str() {
        None => return Err(anyhow!("missing host for {}", url.scheme())),
        Some(host) => params.host = host.to_string(),
    };
    if let Some(port) = url.port() {
        params.port = Some(port);
    }

    // parse scheme
    let query_pairs = url.query_pairs();
    for query_pair in query_pairs {
        let (key, value) = query_pair;
        if key == "scheme" {
            if value != "http" && value != "https" {
                return Err(anyhow!("invalid scheme {} for {}", value, url.scheme()))
            }
            params.scheme = Some(value.into_owned())
        }
    }

    tracing::info!("🔊 speaker: open jsonrpc child from server {}", params.to_url());

    SpeakerJsonRpcChild::open(params)
}

