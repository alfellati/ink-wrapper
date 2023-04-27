# ink-wrapper

`ink-wrapper` is a tool that generates type-safe code for calling a substrate smart contract based on the metadata
(`.json`) file for that contract.

## Installation

Install the tool from [crates.io](https://crates.io):

```bash
cargo install ink-wrapper
```

## Usage

### Setup

Given some metadata file like `my_contract.json` run the tool and save the output to a file in your project:

```bash
ink-wrapper -m my_contract.json > src/my_contract.rs
```

We only take minimal steps to format the output of the tool, so we recommend that you run it through a formatter when
(re)generating:

```bash
ink-wrapper -m my_contract.json | rustfmt --edition 2021 > src/my_contract.rs
```

The output should compile with no warnings, please create an issue if any warnings pop up in your project in the
generated code.

Make sure the file you generated is included in your module structure:

```rust
mod my_contract;
```

You will need the following dependencies for the wrapper to work:

```toml
ink-wrapper-types = "0.2.0"
scale = { package = "parity-scale-codec", version = "3", default-features = false, features = ["derive"] }
ink_primitives = "4.0.1"

# You only need this one if you have messages of the form `Trait::message`, like the ones generated by openbrush, for
# example.
async-trait = "0.1.68"

# This one is optional, but you most likely need it as well if you're using the default `aleph_client` implementation
# for actually making calls. Otherwise, you will need to implement `ink_wrapper_types::Connection` and
# `ink_wrapper_types::SignedConnection` yourself.
aleph_client = "3.0.0"
```

### Basic usage

With that, you're ready to use the wrappers in your code. The generated module will have an `Instance` struct that
represents an instance of your contract. You can either talk to an existing instance by converting an `account_id` to
an `Instance`:

```rust
let account_id: ink_primitives::AccountId = ...;
let instance: my_contract::Instance = account_id.into();
```

Or (assuming the contract code has already been uploaded) create an instance using one of the generated constructors:

```rust
let instance = my_contract::Instance::some_constructor(&conn, arg1, arg2).await?;
```

And then call methods on your contract:

```rust
let result = instance.some_getter(&conn, arg1, arg2).await?;
let tx_info = instance.some_mutator(&conn, arg1, arg2).await?;
```

Note that any methods that have names like `Trait::method_name` will be grouped into traits in the generated module. You
might encounter this if you're using openbrush, for example their `PSP22` implementation generates method names like
`PSP22::balance_of`. You need to `use` the generated traits to access these:

```rust
use my_contract::PSP22 as _;
instance.balance_of(&conn, account_id).await?
```

In the examples above, `conn` is anything that implements `ink_wrapper_types::Connection` (and
`ink_wrapper_types::SignedConnection` if you want to use constructors or mutators). Default implementations are provided
for the connection in `aleph_client`.

### Events

`ink_wrapper_types::Connection` also allows you to fetch events for a given `TxInfo`:

```rust
use ink_wrapper_types::Connection as _;

let tx_info = instance.some_mutator(&conn, arg1, arg2).await?;
let all_events = conn.get_contract_events(tx_info).await?;
let contract_events = all_events.for_contract(instance);
let sub_contract_events = all_events.for_contract(sub_contract);
```

The `all_events` object above may contain events from multiple contracts if the contract called into them. In that case,
you can filter and parse these events by calling `for_contract` on it, with the various contracts you're interested in.

### Code upload

If you provide a compile-time path to the compiled `WASM`:

```bash
ink-wrapper -m my_contract.json --wasm-path ../contracts/target/ink/my_contract.wasm
```

you will also be able to use the generated wrapper to upload the contract:

```rust
my_contract::upload(&conn).await
```

Note, that the generated `upload` function will return `Ok(TxInfo)` so long as the transaction was
submitted successfully and the code hash of the metadata matches the uploaded code. If the code already existed on the
chain, no error is returned. You can verify this condition yourself by looking at the events at the returned `TxInfo` and
checking if they contain a `CodeStored` event.

### Example

Look at `test-project` in the project's repo for a fuller example. Note that `test-project` is missing the actual
wrappers, which are normally generated when testing. The easiest way to regenerate them is by running
`make all-dockerized` (requires docker) - see [Development](#development) for more on that.

## Development

Use the commands provided in the `Makefile` to replicate the build process run on CI:

```bash
make help
```

The most hassle-free is to just run everything in docker:

```bash
make all-dockerized
```

If you have the tooling installed on your host and start a node yourself, you can also run the build on your host:

```bash
make all
```

In case there are any runaway containers from `all-dockerized` you can kill them:

```bash
make kill
```
