pub mod support {
    use serde::Deserialize;
    use serde::Serialize;

    /// The number of bytes of the module-specific `error` field defined in [`ModuleError`].
    /// In FRAME, this is the maximum encoded size of a pallet error type.
    pub const MAX_MODULE_ERROR_ENCODED_SIZE: usize = 4;

    /// Errors related to transactional storage layers.
    /// see https://paritytech.github.io/substrate/master/src/sp_runtime/lib.rs.html#501-506
    #[derive(Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
    pub enum TransactionalError {
        /// Too many transactional layers have been spawned.
        LimitReached,
        /// A transactional layer was expected, but does not exsist.
        NoLayer,
    }

    /// Arithmetic errors.
    /// see https://paritytech.github.io/substrate/master/src/sp_arithmetic/lib.rs.html#62
    #[derive(Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
    pub enum ArithmeticError {
        /// Underflow.
        Underflow,
        /// Overflow.
        Overflow,
        /// Division by zero.
        DivisionByZero,
    }

    /// Description of what went wrong when trying to complete an operation on a token.
    /// see https://paritytech.github.io/substrate/master/src/sp_runtime/lib.rs.html#611-633
    #[derive(Eq, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
    pub enum TokenError {
        /// Funds are unavailable.
        FundsUnavailable,
        /// Some part of the balance gives the only provider reference to the account and thus cannot
        /// be (re)moved.
        OnlyProvider,
        /// Account cannot exist with the funds that would be given.
        BelowMinimum,
        /// Account cannot be created.
        CannotCreate,
        /// The asset in question is unknown.
        UnknownAsset,
        /// Funds exist but are frozen.
        Frozen,
        /// Operation is not supported by the asset.
        Unsupported,
        /// Account cannot be created for a held balance.
        CannotCreateHold,
        /// Withdrawal would cause unwanted loss of account.
        NotExpendable,
        /// Account cannot receive the assets.
        Blocked,
    }

    /// Reason why a pallet call failed.
    #[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
    pub struct ModuleError {
        /// Module index, matching the metadata module index.
        pub index: u8,
        /// Module specific error value.
        pub error: [u8; MAX_MODULE_ERROR_ENCODED_SIZE],
        /// Optional error message.
        pub message: Option<String>,
    }

    /// Reason why a dispatch call failed.
    /// see https://paritytech.github.io/substrate/master/src/sp_runtime/lib.rs.html#526-560
    #[derive(Eq, Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub enum DispatchError {
        /// Some error occurred.
        Other(String),
        /// Failed to lookup some data.
        CannotLookup,
        /// A bad origin.
        BadOrigin,
        /// A custom error in a module.
        Module(ModuleError),
        /// At least one consumer is remaining so the account cannot be destroyed.
        ConsumerRemaining,
        /// There are no providers so the account cannot be created.
        NoProviders,
        /// There are too many consumers so the account cannot be created.
        TooManyConsumers,
        /// An error to do with tokens.
        Token(TokenError),
        /// An arithmetic error.
        Arithmetic(ArithmeticError),
        /// The number of transactional layers has been reached, or we are not in a transactional
        /// layer.
        Transactional(TransactionalError),
        /// Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
        Exhausted,
        /// The state is corrupt; this is generally not going to fix itself.
        Corruption,
        /// Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
        Unavailable,
        /// Root origin is not allowed.
        RootNotAllowed,
    }

    /// Explicit enum to denote if a transaction pays fee or not.
    /// see https://docs.rs/frame-support/21.0.0/src/frame_support/dispatch.rs.html#115-120
    #[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
    pub enum Pays {
        /// Transactor will pay related fees.
        Yes,
        /// Transactor will NOT pay related fees.
        No,
    }

    impl Default for Pays {
        fn default() -> Self {
            Self::Yes
        }
    }

    /// see https://docs.rs/frame-support/21.0.0/src/frame_support/dispatch.rs.html#141-160
    #[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
    pub enum DispatchClass {
        /// A normal dispatch.
        Normal,
        /// An operational dispatch.
        Operational,
        /// A mandatory dispatch. These kinds of dispatch are always included regardless of their
        /// weight, therefore it is critical that they are separately validated to ensure that a
        /// malicious validator cannot craft a valid but impossibly heavy block. Usually this just
        /// means ensuring that the extrinsic can only be included once and that it is always very
        /// light.
        ///
        /// Do *NOT* use it for extrinsics that can be heavy.
        ///
        /// The only real use case for this is inherent extrinsics that are required to execute in a
        /// block for the block to be valid, and it solves the issue in the case that the block
        /// initialization is sufficiently heavy to mean that those inherents do not fit into the
        /// block. Essentially, we assume that in these exceptional circumstances, it is better to
        /// allow an overweight block to be created than to not allow any block at all to be created.
        Mandatory,
    }

    impl Default for DispatchClass {
        fn default() -> Self {
            Self::Normal
        }
    }

    /// see https://docs.rs/sp-weights/19.0.0/src/sp_weights/weight_v2.rs.html#29
    #[derive(Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
    pub struct Weight {
        /// The weight of computational time used based on some reference hardware.
        pub ref_time: u64,
        /// The weight of storage space used by proof of validity.
        pub proof_size: u64,
    }

    // see https://docs.rs/frame-support/21.0.0/src/frame_support/dispatch.rs.html#207-214
    #[derive(Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
    pub struct DispatchInfo {
        /// Weight of this transaction.
        pub weight: Weight,
        /// Class of this transaction.
        pub class: DispatchClass,
        /// Does this transaction pay fees.
        pub pays_fee: Pays,
    }
}

pub mod event {
    use super::support::DispatchError;
    use super::support::DispatchInfo;

    #[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
    pub struct ExtrinsicSuccess {
        pub dispatch_info: DispatchInfo,
    }

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    pub struct ExtrinsicFailed {
        dispatch_error: DispatchError,
        dispatch_info: DispatchInfo,
    }
}
