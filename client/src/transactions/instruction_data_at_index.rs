use solana_address::Address;
use solana_instruction::Instruction;
use solana_sdk::transaction::Transaction;

pub trait InstructionDataAtIndex {
    fn program_id(&self, instruction_index: usize) -> Option<&Address>;

    fn data(&self, instruction_index: usize) -> Option<&[u8]>;
}

impl InstructionDataAtIndex for &[Instruction] {
    fn data(&self, instruction_index: usize) -> Option<&[u8]> {
        Some(&self.get(instruction_index)?.data)
    }

    fn program_id(&self, instruction_index: usize) -> Option<&Address> {
        Some(&self.get(instruction_index)?.program_id)
    }
}

impl InstructionDataAtIndex for Transaction {
    fn data(&self, instruction_index: usize) -> Option<&[u8]> {
        Some(&self.message.instructions.get(instruction_index)?.data)
    }

    fn program_id(&self, instruction_index: usize) -> Option<&Address> {
        self.message.program_id(instruction_index)
    }
}
