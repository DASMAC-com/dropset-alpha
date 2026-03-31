# Overview
This crate is a `shank`-like macro to
generate structs for instruction invocation in various contexts.

To see the different outputs based on the feature flags, view the
`test-fixtures` crate and run the tests to expand the generated output for each.

## Codama IDL Generation

The `codama` feature flag enables `#[derive(CodamaType)]` and auto-generates
`CodamaProgram` impls on instruction enums. These produce a
[Codama IDL](https://github.com/codama-idl/codama) for TypeScript client generation.
See [`codama-idl-gen/`](../codama-idl-gen/) for usage and details.
