use serde::Deserialize;
use serde::Serialize;


use jsonrpsee_core::traits::ToRpcParams;
use serde_json::value::RawValue;
use jsonrpsee_core::Error;

#[derive(Clone, Serialize, Deserialize)]
pub struct WriteBlockRequest<T> 
where T: Clone + Serialize{
    pub chain: String,
    pub blocks: Vec<T>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WriteBlockResponse {}


impl<T> ToRpcParams for WriteBlockRequest<T>  
where T: Clone + Serialize  {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, Error> {
        let s = String::from_utf8(serde_json::to_vec(&self)?).expect("valid UTF8 format");
        serde_json::value::RawValue::from_string(s).map(Some).map_err(jsonrpsee_core::Error::ParseError)
    }
}