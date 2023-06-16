use subxt::Config;
use subxt::config::substrate::BlakeTwo256;
use subxt::config::substrate::SubstrateHeader;
use serde::Serialize;
use serde::Deserialize;

type GenericHeader = SubstrateHeader<u32, BlakeTwo256>;


#[derive(Serialize, Deserialize)]
pub struct PolkadotChainBlock<T: Config> {
    pub header: T::Header,
}



#[cfg(test)]
mod test {
    use super::*;

    // Smoldot returns numeric block numbers in the header at the time of writing;
    // ensure we can deserialize them properly.
    #[test]
    fn can_deserialize_numeric_block_number() {
        let numeric_block_number_json = r#"
            {
                "digest": {
                    "logs": []
                },
                "extrinsicsRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "number": 4,
                "parentHash": "0xcb2690b2c85ceab55be03fc7f7f5f3857e7efeb7a020600ebd4331e10be2f7a5",
                "stateRoot": "0x0000000000000000000000000000000000000000000000000000000000000000"
            }
        "#;

        let header: GenericHeader =
            serde_json::from_str(numeric_block_number_json).expect("valid block header");
        assert_eq!(header.number, 4);
    }

    // Substrate returns hex block numbers; ensure we can also deserialize those OK.
    #[test]
    fn can_deserialize_hex_block_number() {
        let numeric_block_number_json = r#"
            {
                "digest": {
                    "logs": []
                },
                "extrinsicsRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "number": "0x04",
                "parentHash": "0xcb2690b2c85ceab55be03fc7f7f5f3857e7efeb7a020600ebd4331e10be2f7a5",
                "stateRoot": "0x0000000000000000000000000000000000000000000000000000000000000000"
            }
        "#;

        let header: GenericHeader =
            serde_json::from_str(numeric_block_number_json).expect("valid block header");
        assert_eq!(header.number, 4);
    }
}