#![no_std]

use pinocchio::{no_allocator, nostd_panic_handler, program_entrypoint};

mod entrypoint;
mod instructions;

program_entrypoint!(entrypoint::process_instruction);
no_allocator!();
nostd_panic_handler!();
