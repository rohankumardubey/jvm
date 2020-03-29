use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Baload;

impl Instruction for Baload {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::baload,
            icp: 0,
        };

        (info, pc + 1)
    }
}
