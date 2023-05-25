#[cfg(feature = "aleph_client")]
mod aleph_client;
pub mod util;

use std::marker::PhantomData;

use async_trait::async_trait;
use ink_primitives::AccountId;

/// Represents a call to a contract constructor.
#[derive(Debug, Clone)]
pub struct InstantiateCall<T: Send> {
    /// The code hash of the contract to instantiate.
    pub code_hash: [u8; 32],
    /// The encoded data of the call.
    pub data: Vec<u8>,
    /// The salt to use for the contract.
    pub salt: Vec<u8>,
    /// A marker for the type of contract to instantiate.
    _contract: PhantomData<T>,
}

impl<T: Send> InstantiateCall<T> {
    /// Create a new instantiate call.
    pub fn new(code_hash: [u8; 32], data: Vec<u8>) -> Self {
        Self {
            code_hash,
            data,
            salt: vec![],
            _contract: Default::default(),
        }
    }

    /// Set the salt to use for the instantiation.
    pub fn with_salt(mut self, salt: Vec<u8>) -> Self {
        self.salt = salt;
        self
    }
}

/// Represents a mutating contract call to be made.
#[derive(Debug, Clone)]
pub struct ExecCall {
    /// The account id of the contract to call.
    pub account_id: AccountId,
    /// The encoded data of the call.
    pub data: Vec<u8>,
}

impl ExecCall {
    /// Create a new exec call.
    pub fn new(account_id: AccountId, data: Vec<u8>) -> Self {
        Self { account_id, data }
    }
}

/// Represents a read-only contract call to be made.
#[derive(Debug, Clone)]
pub struct ReadCall<T: scale::Decode + Send> {
    /// The account id of the contract to call.
    pub account_id: AccountId,
    /// The encoded data of the call.
    pub data: Vec<u8>,
    /// A marker for the type to decode the result into.
    _return_type: PhantomData<T>,
}

impl<T: scale::Decode + Send> ReadCall<T> {
    /// Create a new read call.
    pub fn new(account_id: AccountId, data: Vec<u8>) -> Self {
        Self {
            account_id,
            data,
            _return_type: Default::default(),
        }
    }
}

/// Contracts will use this trait to invoke mutating operations - constructor and mutating methods.
#[async_trait]
pub trait SignedConnection<TxInfo, E>: Sync {
    /// Upload the given WASM code to the chain.
    ///
    /// Implementation is optional, the default calls `unimplemented!()`.
    /// The implementor SHOULD verify that the code hash resulting from the upload is equal to the given `code_hash`.
    async fn upload(&self, _wasm: Vec<u8>, _code_hash: Vec<u8>) -> Result<TxInfo, E> {
        unimplemented!()
    }

    /// Instantiate a contract with the given code hash and salt.
    ///
    /// The constructor selector and arguments are already serialized into `data`.
    async fn instantiate<T: Send + From<AccountId>>(
        &self,
        call: InstantiateCall<T>,
    ) -> Result<T, E>;

    /// Invoke a mutating method on the `account_id` contract.
    ///
    /// The method selector and arguments are already serialized into `data`.
    async fn exec(&self, call: ExecCall) -> Result<TxInfo, E>;
}

/// Contracts will use this trait for reading data from the chain - non-mutating methods and fetching events.
#[async_trait]
pub trait Connection<TxInfo, E>: Sync {
    /// Read from a non-mutating method on the `account_id` contract.
    ///
    /// The method selector and arguments are already serialized into `data`.
    async fn read<T: scale::Decode + Send>(&self, call: ReadCall<T>) -> Result<T, E>;

    /// Fetch all events emitted by contracts in the transaction with the given `tx_info`.
    async fn get_contract_events(&self, tx_info: TxInfo) -> Result<ContractEvents, E>;
}

/// Represents a raw event emitted by a contract.
pub struct ContractEvent {
    /// The account id of the contract that emitted the event.
    pub account_id: AccountId,
    /// The unparsed data of the event.
    pub data: Vec<u8>,
}

/// Represents a collection of events emitted by contracts in a single transaction.
pub struct ContractEvents {
    pub events: Vec<ContractEvent>,
}

/// A trait that allows to decode events emitted by a specific contract.
pub trait EventSource: Copy + Into<AccountId> {
    /// The type to decode the emitted events into.
    type Event: scale::Decode;
}

impl ContractEvents {
    /// Returns the events emitted by a specific contract.
    ///
    /// Note that this method returns a `Vec<Result<_>>`. An error indicates that a particular event could not be
    /// decoded even though it was emitted byt the particular contract. This can happen if the metadata used to generate
    /// the contract wrapper is out of date. If you're sure that's not the case, then it might be a bug.
    pub fn for_contract<C: EventSource>(&self, contract: C) -> Vec<Result<C::Event, scale::Error>> {
        use scale::Decode as _;

        self.events
            .iter()
            .filter(|e| e.account_id == contract.into())
            .map(|e| C::Event::decode(&mut e.data.as_slice()))
            .collect()
    }
}

/// A wrapper around `ink_primitives::LangError` that implements `std::error::Error`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct InkLangError(ink_primitives::LangError);

impl From<ink_primitives::LangError> for InkLangError {
    fn from(e: ink_primitives::LangError) -> Self {
        Self(e)
    }
}

impl std::fmt::Display for InkLangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InkLangError({:?})", self.0)
    }
}

impl std::error::Error for InkLangError {}
