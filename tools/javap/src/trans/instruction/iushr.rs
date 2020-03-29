use super::{Instruction, InstructionInfo};
use classfile::OpCode;

pub struct Iushr;

impl Instruction for Iushr {
    fn run(&self, codes: &[u8], pc: usize) -> (InstructionInfo, usize) {
        let info = InstructionInfo {
            op_code: OpCode::iushr,
            icp: 0,
        };

        (info, pc + 1)
    }
}
