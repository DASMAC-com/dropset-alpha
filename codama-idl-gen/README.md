# codama-idl-gen

Generates a [Codama IDL](https://github.com/codama-idl/codama) JSON file from
the instruction macro metadata defined in `dropset-interface` which can be used
to produce a TypeScript client from it.

## Usage

From the workspace root:

```sh
pnpm run codama:gen
```

This runs the full pipeline:
1. IDL generation
2. TypeScript client generation
3. Format + lint all generated files

The generated TypeScript code is written to `ts-sdk/src/generated/`.

## How it works

1. `#[derive(CodamaType)]` on structs generates a `CodamaType` impl that
   describes the struct's fields as a Codama [StructTypeNode].

2. `#[derive(CodamaProgram)]` on the instruction enum generates a
   `CodamaProgram` impl that produces the full IDL tree: [InstructionNode]s
   with accounts, arguments, and [discriminators][ConstantDiscriminatorNode].

3. Custom argument types are automatically collected from instruction variants
   and included as [DefinedTypeNode]s. If a type doesn't implement
   `CodamaType`, a compile error is produced.

4. The binary calls `DropsetInstruction::codama_root()`, serializes the result,
   and writes to [`idl.json`].

## Adding new types

### To the IDL

Types that appear as instruction arguments are picked up automatically via the
derive macros. For types that need to be registered manually (e.g. account
state types), add them in [`main.rs`] where `root.program.defined_types` is
extended.

Derive examples:

- `#[derive(CodamaType)]`
- `#[cfg_attr(feature = "codama", derive(CodamaType))]`

For manual implementations, see `UnvalidatedOrders` in
[`orders.rs`].

### To the Codama Rust types

The Rust-side Codama type definitions live in [`codama.rs`]. This is not a
comprehensive mapping of the full [Codama type system][CodamaNodes] – only the
subset needed for the current IDL is implemented.

If you need a Codama node type that isn't represented yet, add it there and
implement `CodamaType` for it.

[CodamaNodes]: https://github.com/codama-idl/codama/tree/main/packages/nodes/docs
[ConstantDiscriminatorNode]: https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/discriminatorNodes/ConstantDiscriminatorNode.md
[InstructionNode]: https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/InstructionNode.md
[StructTypeNode]: https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/typeNodes/StructTypeNode.md
[DefinedTypeNode]: https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/DefinedTypeNode.md
[`idl.json`]: idl.json
[`codama.rs`]: ../instruction-macros/crates/instruction-macros-traits/src/codama.rs
[`main.rs`]: src/main.rs
[`orders.rs`]: ../interface/src/instructions/orders.rs
