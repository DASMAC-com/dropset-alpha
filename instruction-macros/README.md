# Overview
This crate is a `shank`-like macro to
generate structs for instruction invocation in various contexts.

To see the different outputs based on the feature flags, view the
`test-fixtures` crate and run the tests to expand the generated output for each.

## Codama IDL Generation

To produce a [Codama IDL], add the appropriate Codama derives to your types:
- For structs: add `#[derive(CodamaType)]`
- For program instruction enums: `#[derive(CodamaProgram)]`

And then run the [`codama-idl-gen`] binary to generate a TypeScript client.

The `codama` feature flag must be enabled on `instruction-macros` to make the
Codama types available.

See [`codama-idl-gen`] for usage.

[`codama-idl-gen`]: ../codama-idl-gen/
[Codama IDL]: https://github.com/codama-idl/codama
