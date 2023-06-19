use hyperdot_core::types::ChainKind;
use jsonrpsee_core::traits::ToRpcParams;
use jsonrpsee_core::Error;
use serde::Deserialize;
use serde::Serialize;
use serde_json::value::RawValue;

use super::block::polkadot_chain;

#[derive(Clone, Serialize, Deserialize)]
pub struct WriteBlockResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteBlock {
    pub chain: String,
    pub chain_kind: ChainKind,
    pub polkadot_blocks: Option<Vec<polkadot_chain::Block>>,
}

impl ToRpcParams for WriteBlock {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, Error> {
        let s = String::from_utf8(serde_json::to_vec(&self)?).expect("valid UTF8 format");
        serde_json::value::RawValue::from_string(s)
            .map(Some)
            .map_err(jsonrpsee_core::Error::ParseError)
    }
}
