use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Fadd;

impl Instruction for Fadd {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::fadd,
            icp: 0,
        };

        (info, pc + 1)
    }
}
