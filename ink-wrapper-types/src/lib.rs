#[cfg(feature = "aleph_client")]
mod aleph_client;

use async_trait::async_trait;
use ink_primitives::AccountId;

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
    async fn instantiate(
        &self,
        code_hash: [u8; 32],
        salt: Vec<u8>,
        data: Vec<u8>,
    ) -> Result<AccountId, E>;

    /// Invoke a mutating method on the `account_id` contract.
    ///
    /// The method selector and arguments are already serialized into `data`.
    async fn exec(&self, account_id: AccountId, data: Vec<u8>) -> Result<TxInfo, E>;
}

/// Contracts will use this trait for reading data from the chain - non-mutating methods and fetching events.
#[async_trait]
pub trait Connection<TxInfo, E>: Sync {
    /// Read from a non-mutating method on the `account_id` contract.
    ///
    /// The method selector and arguments are already serialized into `data`.
    async fn read<T: scale::Decode>(&self, account_id: AccountId, data: Vec<u8>) -> Result<T, E>;

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
