pub enum Instruction {
    Exit,
    Swap,
    Nop,
    Input,
    StInput,
    Debug,
    Pop,
    Dup,
    Push(i32),
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Lsl,
    Lsr,
    Asr,
    Neg,
    Not,
    StPrint,
    Print,
    Call(i32),
    Ret,
    Goto(i32),
    IfEq(i32),
    IfNe(i32),
    IfLt(i32),
    IfLe(i32),
    IfGt(i32),
    IfGe(i32),
    If0(i32),
    If1(i32),
    Dump,
}

impl Instruction {
    pub fn decode(mem: &[u8; 4096], pc: i32) -> Option<Instruction> {
        if pc < 0 || pc as usize >= mem.len() {
            return None;
        }
        if pc - 3 < 0 {
            return None;
        }
        let opcode = mem[pc as usize];
        let b1 = mem[(pc - 1) as usize] as u32;
        let b2 = mem[(pc - 2) as usize] as u32;
        let b3 = mem[(pc - 3) as usize] as u32;
        let raw = (b1 << 16) | (b2 << 8) | b3;
        let imm = if raw & 0x0080_0000 != 0 {
            (raw | 0xFF00_0000) as i32
        } else {
            raw as i32
        };
        match opcode {
            0x00 => Some(Instruction::Exit),
            0x01 => Some(Instruction::Swap),
            0x02 => Some(Instruction::Nop),
            0x03 => Some(Instruction::Input),
            0x04 => Some(Instruction::StInput),
            0x05 => Some(Instruction::Debug),
            0x10 => Some(Instruction::Pop),
            0x11 => Some(Instruction::Dup),
            0x12 => Some(Instruction::Push(imm)),
            0x20 => Some(Instruction::Add),
            0x21 => Some(Instruction::Sub),
            0x22 => Some(Instruction::Mul),
            0x23 => Some(Instruction::Div),
            0x24 => Some(Instruction::Rem),
            0x25 => Some(Instruction::And),
            0x26 => Some(Instruction::Or),
            0x27 => Some(Instruction::Xor),
            0x28 => Some(Instruction::Lsl),
            0x29 => Some(Instruction::Lsr),
            0x2A => Some(Instruction::Asr),
            0x2B => Some(Instruction::Neg),
            0x2C => Some(Instruction::Not),
            0x30 => Some(Instruction::StPrint),
            0x31 => Some(Instruction::Print),
            0x40 => Some(Instruction::Call(imm)),
            0x41 => Some(Instruction::Ret),
            0x42 => Some(Instruction::Goto(imm)),
            0x50 => Some(Instruction::IfEq(imm)),
            0x51 => Some(Instruction::IfNe(imm)),
            0x52 => Some(Instruction::IfLt(imm)),
            0x53 => Some(Instruction::IfLe(imm)),
            0x54 => Some(Instruction::IfGt(imm)),
            0x55 => Some(Instruction::IfGe(imm)),
            0x56 => Some(Instruction::If0(imm)),
            0x57 => Some(Instruction::If1(imm)),
            0x60 => Some(Instruction::Dump),
            _ => None,
        }
    }
}
