# Non-exhaustive checklist for integrating new changes for an upcoming hard fork/devnet

## Introducing new SIP types or changes to primitive types

- Make required changes to primitive data structures on [alloy](https://github.com/alloy-rs/alloy)
- All new SIP data structures/constants/helpers etc. go into the `alloy-sips` crate at first.
- New transaction types go into `alloy-consensus`
- If there are changes to existing data structures, such as `Header` or `Block`, apply them to the types in
  `alloy-consensus` (e.g. new `request_hashes` field in SilaPrague)

## Engine API

- If there are changes to the engine API (e.g. a new `engine_newPayloadVx` and `engine_getPayloadVx` pair) add the new
  types to the `alloy-rpc-types-engine` crate.
- If there are new parameters to the `engine_newPayloadVx` endpoint, add them to the `ExecutionPayloadSidecar` container
  type. This types contains all additional parameters that are required to convert an `ExecutionPayload` to an EL block.

## Rsil changes

### Updates to the engine API

- Add new endpoints to the `EngineApi` trait and implement endpoints.
- Update the `ExecutionPayload` + `ExecutionPayloadSidecar` to `Block` conversion if there are any additional
  parameters.
- Update version specific validation checks in the `EngineValidator` trait.

