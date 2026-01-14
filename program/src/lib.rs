//! On-chain program logic entry module.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod context;
mod debug;
mod events;
mod instructions;
mod shared;
mod validation;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

pub const ID: solana_address::Address =
    solana_address::Address::from_str_const("TESTnXwv2eHoftsSd5NEdpH4zEu7XRC8jviuoNPdB2Q");
