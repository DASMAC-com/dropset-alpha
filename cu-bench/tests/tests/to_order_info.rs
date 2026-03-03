use cu_bench_tests::new_cu_bench_mollusk;
use solana_address::Address;
use solana_instruction::Instruction;

#[test]
fn pow10_cu() {
    let program_id = Address::new_unique();
    let mollusk = new_cu_bench_mollusk(&program_id, "cu_bench_to_order_info.so");

    let instruction = Instruction::new_with_bytes(program_id, &[], vec![]);
    let result = mollusk.process_instruction(&instruction, &[]);
    assert!(
        result.program_result.is_ok(),
        "Instruction failed: {:?}",
        result.program_result
    );
    let total = result.compute_units_consumed;
    println!(
        "Compute units consumed: {} total, {} per call (10 calls)",
        total,
        total / 10
    );
}
