//! Codama IDL types and traits for generating Solana program IDL JSON.
//!
//! The node types defined here correspond to the Codama IDL specification:
//! <https://github.com/codama-idl/codama/tree/main/packages/nodes/docs>
//!
//! Only the subset of nodes required for instruction-level IDL generation is implemented.
//! For the full specification, see the individual node docs linked on each type.

extern crate alloc;

/// Re-export alloc for use in generated code (no_std contexts).
pub mod _alloc {
    extern crate alloc;
    pub use alloc::{
        string,
        vec,
    };
}

use alloc::{
    boxed::Box,
    string::String,
    vec::Vec,
};

use serde::Serialize;

/// Describes a type's Codama IDL representation. Implement this for any type that appears as an
/// instruction argument or account field so the IDL generator can produce the correct type node.
pub trait CodamaType {
    /// The camelCase name of this type in the Codama IDL.
    fn name() -> &'static str;

    /// The full type node describing this type's layout.
    fn type_node() -> TypeNode;
}

/// Describes a full program's Codama IDL. Derived on instruction enums to produce the complete
/// IDL tree (instructions, accounts, discriminators, defined types).
pub trait CodamaProgram {
    fn codama_root(program_name: &str, program_id: &str) -> RootNode;
}

/// Represents the data type of a field or argument in the IDL.
///
/// Each variant maps to a Codama type node:
/// <https://github.com/codama-idl/codama/tree/main/packages/nodes/docs/typeNodes>
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum TypeNode {
    #[serde(rename = "numberTypeNode")]
    Number(NumberTypeNode),
    #[serde(rename = "booleanTypeNode")]
    Boolean(BooleanTypeNode),
    #[serde(rename = "publicKeyTypeNode")]
    PublicKey(PublicKeyTypeNode),
    #[serde(rename = "structTypeNode")]
    Struct(StructTypeNode),
    #[serde(rename = "arrayTypeNode")]
    Array(ArrayTypeNode),
    #[serde(rename = "bytesTypeNode")]
    Bytes(BytesTypeNode),
    #[serde(rename = "definedTypeLinkNode")]
    DefinedTypeLink(DefinedTypeLinkNode),
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/typeNodes/NumberTypeNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct NumberTypeNode {
    pub format: NumberFormat,
    pub endian: Endian,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NumberFormat {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Endian {
    Le,
    Be,
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/typeNodes/BooleanTypeNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct BooleanTypeNode {
    pub size: NumberTypeNode,
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/typeNodes/PublicKeyTypeNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct PublicKeyTypeNode {}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/typeNodes/StructTypeNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct StructTypeNode {
    pub fields: Vec<StructFieldTypeNode>,
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/typeNodes/StructFieldTypeNode.md>
#[derive(Debug, Clone, Serialize)]
#[serde(rename = "structFieldTypeNode")]
pub struct StructFieldTypeNode {
    pub kind: &'static str,
    pub name: String,
    #[serde(rename = "type")]
    pub ty: TypeNode,
    pub docs: Vec<String>,
}

impl StructFieldTypeNode {
    pub fn new(field_name: impl Into<String>, ty: TypeNode) -> Self {
        Self {
            kind: "structFieldTypeNode",
            name: field_name.into(),
            ty,
            docs: Vec::new(),
        }
    }

    pub fn array<T: CodamaType>(field_name: impl Into<String>, count: usize) -> Self {
        Self::new(field_name, array_type(T::type_node(), count))
    }
}

#[rustfmt::skip]
impl StructFieldTypeNode {
    pub fn u8(name: impl Into<String>) -> Self { Self::new(name, NumberFormat::U8.le()) }
    pub fn u16(name: impl Into<String>) -> Self { Self::new(name, NumberFormat::U16.le()) }
    pub fn u32(name: impl Into<String>) -> Self { Self::new(name, NumberFormat::U32.le()) }
    pub fn u64(name: impl Into<String>) -> Self { Self::new(name, NumberFormat::U64.le()) }
    pub fn u128(name: impl Into<String>) -> Self { Self::new(name, NumberFormat::U128.le()) }
    pub fn bool(name: impl Into<String>) -> Self { Self::new(name, boolean_type()) }
    pub fn address(name: impl Into<String>) -> Self { Self::new(name, address_type()) }
    pub fn bytes(name: impl Into<String>) -> Self { Self::new(name, bytes_type()) }
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/typeNodes/ArrayTypeNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct ArrayTypeNode {
    pub item: Box<TypeNode>,
    pub count: CountNode,
}

/// <https://github.com/codama-idl/codama/tree/main/packages/nodes/docs/countNodes>
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum CountNode {
    #[serde(rename = "fixedCountNode")]
    Fixed(FixedCountNode),
}

#[derive(Debug, Clone, Serialize)]
pub struct FixedCountNode {
    pub value: usize,
}

/// Raw bytes with no inherent size. Size is determined by context (e.g. remainder of account data).
///
/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/typeNodes/BytesTypeNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct BytesTypeNode {}

/// References a reusable type defined in the program's `definedTypes` array.
///
/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/linkNodes/DefinedTypeLinkNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct DefinedTypeLinkNode {
    pub name: String,
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/RootNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct RootNode {
    pub kind: &'static str,
    pub standard: &'static str,
    pub version: &'static str,
    pub program: ProgramNode,
    #[serde(rename = "additionalPrograms")]
    pub additional_programs: Vec<ProgramNode>,
}

impl RootNode {
    pub fn new(program: ProgramNode) -> Self {
        Self {
            kind: "rootNode",
            standard: "codama",
            version: "1.0.0",
            program,
            additional_programs: Vec::new(),
        }
    }
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/ProgramNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct ProgramNode {
    pub kind: &'static str,
    pub name: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub version: &'static str,
    pub docs: Vec<String>,
    pub accounts: Vec<AccountNode>,
    pub instructions: Vec<InstructionNode>,
    #[serde(rename = "definedTypes")]
    pub defined_types: Vec<DefinedTypeNode>,
    // Not yet implemented. Serializes to `[]` in the IDL.
    pub pdas: Vec<()>,
    // Not yet implemented. Serializes to `[]` in the IDL.
    pub errors: Vec<()>,
    // Not yet implemented. Serializes to `[]` in the IDL.
    pub events: Vec<()>,
}

impl ProgramNode {
    pub fn new(
        name: String,
        public_key: String,
        instructions: Vec<InstructionNode>,
        defined_types: Vec<DefinedTypeNode>,
    ) -> Self {
        Self {
            kind: "programNode",
            name,
            public_key,
            version: "0.1.0",
            docs: Vec::new(),
            accounts: Vec::new(),
            instructions,
            defined_types,
            pdas: Vec::new(),
            errors: Vec::new(),
            events: Vec::new(),
        }
    }
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/InstructionNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct InstructionNode {
    pub kind: &'static str,
    pub name: String,
    pub docs: Vec<String>,
    #[serde(rename = "optionalAccountStrategy")]
    pub optional_account_strategy: &'static str,
    pub accounts: Vec<InstructionAccountNode>,
    pub arguments: Vec<InstructionArgumentNode>,
    pub discriminators: Vec<ConstantDiscriminatorNode>,
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/InstructionAccountNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct InstructionAccountNode {
    pub kind: &'static str,
    pub name: String,
    #[serde(rename = "isWritable")]
    pub is_writable: bool,
    #[serde(rename = "isSigner")]
    pub is_signer: bool,
    #[serde(rename = "isOptional")]
    pub is_optional: bool,
    pub docs: Vec<String>,
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/InstructionArgumentNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct InstructionArgumentNode {
    pub kind: &'static str,
    pub name: String,
    pub docs: Vec<String>,
    #[serde(rename = "type")]
    pub ty: TypeNode,
    #[serde(rename = "defaultValue", skip_serializing_if = "Option::is_none")]
    pub default_value: Option<NumberValueNode>,
    #[serde(
        rename = "defaultValueStrategy",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_value_strategy: Option<&'static str>,
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/valueNodes/NumberValueNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct NumberValueNode {
    pub kind: &'static str,
    pub number: u64,
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/discriminatorNodes/ConstantDiscriminatorNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct ConstantDiscriminatorNode {
    pub kind: &'static str,
    pub offset: u8,
    pub constant: ConstantValueNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConstantValueNode {
    pub kind: &'static str,
    #[serde(rename = "type")]
    pub ty: TypeNode,
    pub value: NumberValueNode,
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/DefinedTypeNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct DefinedTypeNode {
    pub kind: &'static str,
    pub name: String,
    pub docs: Vec<String>,
    #[serde(rename = "type")]
    pub ty: TypeNode,
}

/// <https://github.com/codama-idl/codama/blob/main/packages/nodes/docs/AccountNode.md>
#[derive(Debug, Clone, Serialize)]
pub struct AccountNode {
    pub kind: &'static str,
    pub name: String,
    pub docs: Vec<String>,
    pub data: TypeNode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub discriminators: Vec<ConstantDiscriminatorNode>,
}

impl AccountNode {
    pub fn new(name: impl Into<String>, data: TypeNode) -> Self {
        Self {
            kind: "accountNode",
            name: name.into(),
            docs: Vec::new(),
            data,
            size: None,
            discriminators: Vec::new(),
        }
    }
}

impl NumberFormat {
    /// Creates a little-endian [`TypeNode::Number`] from this format.
    pub fn le(self) -> TypeNode {
        TypeNode::Number(NumberTypeNode {
            format: self,
            endian: Endian::Le,
        })
    }
}

pub fn boolean_type() -> TypeNode {
    TypeNode::Boolean(BooleanTypeNode {
        size: NumberTypeNode {
            format: NumberFormat::U8,
            endian: Endian::Le,
        },
    })
}

pub fn address_type() -> TypeNode {
    TypeNode::PublicKey(PublicKeyTypeNode {})
}

pub fn bytes_type() -> TypeNode {
    TypeNode::Bytes(BytesTypeNode {})
}

pub fn defined_type_link(name: &str) -> TypeNode {
    TypeNode::DefinedTypeLink(DefinedTypeLinkNode {
        name: String::from(name),
    })
}

pub fn array_type(item: TypeNode, count: usize) -> TypeNode {
    TypeNode::Array(ArrayTypeNode {
        item: Box::new(item),
        count: CountNode::Fixed(FixedCountNode { value: count }),
    })
}

impl CodamaType for u8 {
    fn name() -> &'static str {
        "u8"
    }

    fn type_node() -> TypeNode {
        NumberFormat::U8.le()
    }
}

impl CodamaType for u16 {
    fn name() -> &'static str {
        "u16"
    }

    fn type_node() -> TypeNode {
        NumberFormat::U16.le()
    }
}

impl CodamaType for u32 {
    fn name() -> &'static str {
        "u32"
    }

    fn type_node() -> TypeNode {
        NumberFormat::U32.le()
    }
}

impl CodamaType for u64 {
    fn name() -> &'static str {
        "u64"
    }

    fn type_node() -> TypeNode {
        NumberFormat::U64.le()
    }
}

impl CodamaType for u128 {
    fn name() -> &'static str {
        "u128"
    }

    fn type_node() -> TypeNode {
        NumberFormat::U128.le()
    }
}

impl CodamaType for bool {
    fn name() -> &'static str {
        "bool"
    }

    fn type_node() -> TypeNode {
        boolean_type()
    }
}

impl CodamaType for solana_address::Address {
    fn name() -> &'static str {
        "publicKey"
    }

    fn type_node() -> TypeNode {
        address_type()
    }
}

pub fn discriminator(format: NumberFormat, value: u64) -> ConstantDiscriminatorNode {
    ConstantDiscriminatorNode {
        kind: "constantDiscriminatorNode",
        offset: 0,
        constant: ConstantValueNode {
            kind: "constantValueNode",
            ty: format.le(),
            value: NumberValueNode {
                kind: "numberValueNode",
                number: value,
            },
        },
    }
}
