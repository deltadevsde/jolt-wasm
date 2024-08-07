use common::constants::virtual_register_index;
use tracer::{ELFInstruction, RVTraceRow, RegisterState, RV32IM};

use super::VirtualInstructionSequence;
use crate::jolt::instruction::{
    add::ADDInstruction, beq::BEQInstruction, mulu::MULUInstruction,
    virtual_advice::ADVICEInstruction, virtual_assert_lte::ASSERTLTEInstruction,
    virtual_assert_valid_div0::AssertValidDiv0Instruction,
    virtual_assert_valid_unsigned_remainder::AssertValidUnsignedRemainderInstruction,
    JoltInstruction,
};
/// Perform unsigned division and return quotient
pub struct DIVUInstruction<const WORD_SIZE: usize>;

impl<const WORD_SIZE: usize> VirtualInstructionSequence for DIVUInstruction<WORD_SIZE> {
    fn virtual_sequence(trace_row: RVTraceRow) -> Vec<RVTraceRow> {
        assert_eq!(trace_row.instruction.opcode, RV32IM::DIVU);
        // DIVU operands
        let x = trace_row.register_state.rs1_val.unwrap();
        let y = trace_row.register_state.rs2_val.unwrap();
        // DIVU source registers
        let r_x = trace_row.instruction.rs1;
        let r_y = trace_row.instruction.rs2;
        // Virtual registers used in sequence
        let v_0 = Some(virtual_register_index(0));
        let v_r: Option<u64> = Some(virtual_register_index(1));
        let v_qy = Some(virtual_register_index(2));

        let mut virtual_sequence = vec![];

        let quotient = x / y;
        let remainder = x - quotient * y;

        let q = ADVICEInstruction::<WORD_SIZE>(quotient).lookup_entry();
        virtual_sequence.push(RVTraceRow {
            instruction: ELFInstruction {
                address: trace_row.instruction.address,
                opcode: RV32IM::VIRTUAL_ADVICE,
                rs1: None,
                rs2: None,
                rd: trace_row.instruction.rd,
                imm: None,
                virtual_sequence_index: Some(virtual_sequence.len()),
            },
            register_state: RegisterState {
                rs1_val: None,
                rs2_val: None,
                rd_post_val: Some(q),
            },
            memory_state: None,
            advice_value: Some(quotient),
        });

        let r = ADVICEInstruction::<WORD_SIZE>(remainder).lookup_entry();
        virtual_sequence.push(RVTraceRow {
            instruction: ELFInstruction {
                address: trace_row.instruction.address,
                opcode: RV32IM::VIRTUAL_ADVICE,
                rs1: None,
                rs2: None,
                rd: v_r,
                imm: None,
                virtual_sequence_index: Some(virtual_sequence.len()),
            },
            register_state: RegisterState {
                rs1_val: None,
                rs2_val: None,
                rd_post_val: Some(r),
            },
            memory_state: None,
            advice_value: Some(remainder),
        });

        let q_y = MULUInstruction::<WORD_SIZE>(q, y).lookup_entry();
        virtual_sequence.push(RVTraceRow {
            instruction: ELFInstruction {
                address: trace_row.instruction.address,
                opcode: RV32IM::MULU,
                rs1: trace_row.instruction.rd,
                rs2: r_y,
                rd: v_qy,
                imm: None,
                virtual_sequence_index: Some(virtual_sequence.len()),
            },
            register_state: RegisterState {
                rs1_val: Some(q),
                rs2_val: Some(y),
                rd_post_val: Some(q_y),
            },
            memory_state: None,
            advice_value: None,
        });

        let is_valid = AssertValidUnsignedRemainderInstruction(r, y).lookup_entry();
        assert_eq!(is_valid, 1);
        virtual_sequence.push(RVTraceRow {
            instruction: ELFInstruction {
                address: trace_row.instruction.address,
                opcode: RV32IM::VIRTUAL_ASSERT_VALID_UNSIGNED_REMAINDER,
                rs1: v_r,
                rs2: r_y,
                rd: None,
                imm: None,
                virtual_sequence_index: Some(virtual_sequence.len()),
            },
            register_state: RegisterState {
                rs1_val: Some(r),
                rs2_val: Some(y),
                rd_post_val: None,
            },
            memory_state: None,
            advice_value: None,
        });

        let lte = ASSERTLTEInstruction(q_y, x).lookup_entry();
        assert_eq!(lte, 1);
        virtual_sequence.push(RVTraceRow {
            instruction: ELFInstruction {
                address: trace_row.instruction.address,
                opcode: RV32IM::VIRTUAL_ASSERT_LTE,
                rs1: v_qy,
                rs2: r_x,
                rd: None,
                imm: None,
                virtual_sequence_index: Some(virtual_sequence.len()),
            },
            register_state: RegisterState {
                rs1_val: Some(q_y),
                rs2_val: Some(x),
                rd_post_val: None,
            },
            memory_state: None,
            advice_value: None,
        });

        let is_valid = AssertValidDiv0Instruction::<WORD_SIZE>(y, q).lookup_entry();
        assert_eq!(is_valid, 1);
        virtual_sequence.push(RVTraceRow {
            instruction: ELFInstruction {
                address: trace_row.instruction.address,
                opcode: RV32IM::VIRTUAL_ASSERT_VALID_DIV0,
                rs1: r_y,
                rs2: trace_row.instruction.rd,
                rd: None,
                imm: None,
                virtual_sequence_index: Some(virtual_sequence.len()),
            },
            register_state: RegisterState {
                rs1_val: Some(y),
                rs2_val: Some(q),
                rd_post_val: None,
            },
            memory_state: None,
            advice_value: None,
        });

        let add_0 = ADDInstruction::<WORD_SIZE>(q_y, r).lookup_entry();
        virtual_sequence.push(RVTraceRow {
            instruction: ELFInstruction {
                address: trace_row.instruction.address,
                opcode: RV32IM::ADD,
                rs1: v_qy,
                rs2: v_r,
                rd: v_0,
                imm: None,
                virtual_sequence_index: Some(virtual_sequence.len()),
            },
            register_state: RegisterState {
                rs1_val: Some(q_y),
                rs2_val: Some(r),
                rd_post_val: Some(add_0),
            },
            memory_state: None,
            advice_value: None,
        });

        let _assert_eq = BEQInstruction(add_0, x).lookup_entry();
        virtual_sequence.push(RVTraceRow {
            instruction: ELFInstruction {
                address: trace_row.instruction.address,
                opcode: RV32IM::VIRTUAL_ASSERT_EQ,
                rs1: v_0,
                rs2: r_x,
                rd: None,
                imm: None,
                virtual_sequence_index: Some(virtual_sequence.len()),
            },
            register_state: RegisterState {
                rs1_val: Some(add_0),
                rs2_val: Some(x),
                rd_post_val: None,
            },
            memory_state: None,
            advice_value: None,
        });

        virtual_sequence
    }
}

#[cfg(test)]
mod test {
    use ark_std::test_rng;
    use common::constants::REGISTER_COUNT;
    use rand_chacha::rand_core::RngCore;

    use crate::jolt::vm::rv32i_vm::RV32I;

    use super::*;

    #[test]
    // TODO(moodlezoup): Turn this into a macro, similar to the `jolt_instruction_test` macro
    fn divu_virtual_sequence_32() {
        let mut rng = test_rng();

        let r_x = rng.next_u64() % 32;
        let r_y = rng.next_u64() % 32;
        let rd = rng.next_u64() % 32;

        let x = rng.next_u32() as u64;
        let y = if r_y == r_x { x } else { rng.next_u32() as u64 };
        let result = x / y;

        let divu_trace_row = RVTraceRow {
            instruction: ELFInstruction {
                address: rng.next_u64(),
                opcode: RV32IM::DIVU,
                rs1: Some(r_x),
                rs2: Some(r_y),
                rd: Some(rd),
                imm: None,
                virtual_sequence_index: None,
            },
            register_state: RegisterState {
                rs1_val: Some(x),
                rs2_val: Some(y),
                rd_post_val: Some(result as u64),
            },
            memory_state: None,
            advice_value: None,
        };

        let virtual_sequence = DIVUInstruction::<32>::virtual_sequence(divu_trace_row);
        let mut registers = vec![0u64; REGISTER_COUNT as usize];
        registers[r_x as usize] = x;
        registers[r_y as usize] = y;

        for row in virtual_sequence {
            if let Some(rs1_val) = row.register_state.rs1_val {
                assert_eq!(registers[row.instruction.rs1.unwrap() as usize], rs1_val);
            }
            if let Some(rs2_val) = row.register_state.rs2_val {
                assert_eq!(registers[row.instruction.rs2.unwrap() as usize], rs2_val);
            }

            let lookup = RV32I::try_from(&row).unwrap();
            let output = lookup.lookup_entry();
            if let Some(rd) = row.instruction.rd {
                registers[rd as usize] = output;
                assert_eq!(
                    registers[rd as usize],
                    row.register_state.rd_post_val.unwrap()
                );
            } else {
                assert!(output == 1)
            }
        }

        for (index, val) in registers.iter().enumerate() {
            if index as u64 == r_x {
                // Check that r_x hasn't been clobbered
                assert_eq!(*val, x);
            } else if index as u64 == r_y {
                // Check that r_y hasn't been clobbered
                assert_eq!(*val, y);
            } else if index as u64 == rd {
                // Check that result was written to rd
                assert_eq!(*val, result as u64);
            } else if index < 32 {
                // None of the other "real" registers were touched
                assert_eq!(*val, 0);
            }
        }
    }
}
