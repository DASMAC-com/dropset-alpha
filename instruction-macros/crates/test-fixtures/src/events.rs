use instruction_macros::ProgramInstructionEvent;

#[repr(u8)]
#[derive(ProgramInstructionEvent)]
#[program_id(crate::ID)]
#[rustfmt::skip]
pub enum DropsetEvent {
    #[args(instruction_tag: u8, "The tag of the instruction that emitted the following events.")]
    #[args(market: Address, "The market's address.")]
    #[args(sender: Address, "The sender's address.")]
    #[args(nonce: u64, "The market nonce.")]
    #[args(emitted_count: u16, "The number of events in the following event buffer.")]
    Header,
    #[args(trader: Address, "The trader's address.")]
    #[args(amount: u64, "The amount deposited.")]
    #[args(is_base: bool, "Which token, i.e., `true` => base token, `false` => quote token.")]
    Deposit,
    #[args(trader: Address, "The trader's address.")]
    #[args(amount: u64, "The amount withdrawn.")]
    #[args(is_base: bool, "Which token, i.e., `true` => base token, `false` => quote token.")]
    Withdraw,
}
