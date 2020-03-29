use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Ifne;

impl Instruction for Ifne {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::ifne,
            icp: 0,
        };

        (info, pc + 3)
    }
}
