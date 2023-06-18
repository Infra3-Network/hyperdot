pub mod polkadot_chain {
    use serde::Deserialize;
    use serde::Serialize;
    use subxt::config::substrate::BlakeTwo256;
    use subxt::config::substrate::SubstrateHeader;
    use subxt::config::substrate::U256;
    use subxt::config::Hasher;
    use subxt::Config;

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub enum EventPhase {
        /// Applying an extrinsic.
        ApplyExtrinsic(u32),
        /// Finalizing the block.
        Finalization,
        /// Initializing the block.
        Initialization,
    }

    impl EventPhase {
        pub fn to_string(&self) -> String {
            match *self {
                EventPhase::ApplyExtrinsic(value) => format!("ApplyExtrinsic({})", value),
                EventPhase::Finalization => "Finalization".to_string(),
                EventPhase::Initialization => "Initialization".to_string(),
            }
        }
    }

    /// Raw event.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct EventDescribe {
        /// Unindexed data containing further information on the event
        pub data: Vec<u8>,
        /// What index is this event in the stored events for this block.
        pub index: u32,
        /// The hash of topics.
        pub topics: Vec<Vec<u8>>,
        /// The phase of event.
        pub phase: EventPhase,
        /// The pallet name of event.
        pub pallet_name: String,
        /// The pallet index of event.
        pub pallet_index: u8,
        pub root_bytes: Vec<u8>,
        /// The hash of extrinsic.
        pub extrinsic_hash: Vec<u8>,
    }

    /// The ExtrinsicDetails represent a single extrinsic in a block.
    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct ExtrinsicDetails {
        /// The index of the extrinsic in the block.
        pub index: u32,
        /// Is the extrinsic signed?
        pub is_signed: bool,
        /// The index of the pallet that the extrinsic originated from.
        pub pallet_index: u8,
        /// The name of the pallet from whence the extrinsic originated.
        pub pallet_name: Option<String>,
        /// The index of the extrinsic variant that the extrinsic originated from.
        pub variant_index: u8,
        /// The name of the call (ie the name of the variant that it corresponds to).
        pub variant_name: Option<String>,
        /// Return only the bytes of the address that signed this extrinsic.
        ///
        /// # Note
        ///
        /// It's None if is_signed equals false
        pub signed_address: Option<Vec<u8>>,
        /// Return _all_ of the bytes representing this extrinsic, which include, in order:
        /// - First byte: abbbbbbb (a = 0 for unsigned, 1 for signed, b = version)
        /// - SignatureType (if the payload is signed)
        ///   - Address
        ///   - Signature
        ///   - Extra fields
        /// - Extrinsic call bytes
        pub bytes: Vec<u8>,
        /// The root call bytes of extrinsic.
        pub root_extrinsic_bytes: Option<Vec<u8>>,
        /// pub events: Vec<ExtrinsicEventDescribe>,
        pub events: Option<Vec<EventDescribe>>,
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct BlockGenericBody {
        /// The decoded extrinsics record key of extrinsic in block.
        pub extrinsics: Vec<ExtrinsicDetails>,
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct BlockGenericHeader {
        pub block_number: u64,
        pub block_hash: Vec<u8>,
        pub parent_hash: Vec<u8>,
        pub extrinsics_root: Vec<u8>,
        pub state_root: Vec<u8>,
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct Block {
        pub header: BlockGenericHeader,
        pub body: Option<BlockGenericBody>,
    }
}
