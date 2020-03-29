use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Jsr;

impl Instruction for Jsr {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::jsr,
            icp: 0,
        };

        (info, pc + 3)
    }
}
