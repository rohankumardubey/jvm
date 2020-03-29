use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ldiv;

impl Instruction for Ldiv {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::ldiv,
            icp: 0,
        };

        (info, pc + 1)
    }
}
