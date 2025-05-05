use std::io::{self, Write};
use crate::instruction::Instruction;

#[derive(Debug)]
pub enum Value {
    Int(i32),
    Str(String),
}

pub struct VM {
    pub memory: [u8; 4096],
    pub pc: i32,
    pub stack: Vec<Value>,
    pub sp: usize,
}

impl VM {
    pub fn new(memory: [u8; 4096]) -> VM {
        VM {
            memory,
            pc: 4095,
            stack: Vec::new(),
            sp: 0,
        }
    }

    pub fn run(&mut self) {
        loop {
            if self.pc < 0 || self.pc as usize >= self.memory.len() {
                break;
            }
            let instr_opt = Instruction::decode(&self.memory, self.pc);
            if instr_opt.is_none() {
                break;
            }
            let instr = instr_opt.unwrap();
            self.pc -= 4;
            match instr {
                Instruction::Exit => break,
                Instruction::Swap => {
                    if self.stack.len() < 2 { break; }
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 2);
                }
                Instruction::Nop => {}
                Instruction::Input => {
                    let mut buffer = String::new();
                    if io::stdin().read_line(&mut buffer).is_err() {
                        break;
                    }
                    if let Ok(val) = buffer.trim().parse::<i32>() {
                        self.stack.push(Value::Int(val));
                    } else {
                        break;
                    }
                }
                Instruction::StInput => {
                    let mut buffer = String::new();
                    if io::stdin().read_line(&mut buffer).is_err() {
                        break;
                    }
                    let s = buffer.trim_end().to_string();
                    self.stack.push(Value::Str(s));
                }
                Instruction::Debug => {
                    println!("DEBUG PC={} SP={} STACK={:?}", self.pc, self.sp, self.stack);
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::Dup => {
                    if let Some(v) = self.stack.last() {
                        match v {
                            Value::Int(i) => self.stack.push(Value::Int(*i)),
                            Value::Str(s) => self.stack.push(Value::Str(s.clone())),
                        }
                    } else { break; }
                }
                Instruction::Push(val) => {
                    self.stack.push(Value::Int(val));
                }
                Instruction::Add => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Int(b + a));
                    } else { break; }
                }
                Instruction::Sub => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Int(b - a));
                    } else { break; }
                }
                Instruction::Mul => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Int(b * a));
                    } else { break; }
                }
                Instruction::Div => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        if a == 0 { break; }
                        self.stack.push(Value::Int(b / a));
                    } else { break; }
                }
                Instruction::Rem => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        if a == 0 { break; }
                        self.stack.push(Value::Int(b % a));
                    } else { break; }
                }
                Instruction::And => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Int(b & a));
                    } else { break; }
                }
                Instruction::Or => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Int(b | a));
                    } else { break; }
                }
                Instruction::Xor => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Int(b ^ a));
                    } else { break; }
                }
                Instruction::Lsl => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Int((b as u32).wrapping_shl(a as u32) as i32));
                    } else { break; }
                }
                Instruction::Lsr => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        let bu = b as u32;
                        self.stack.push(Value::Int((bu.wrapping_shr(a as u32)) as i32));
                    } else { break; }
                }
                Instruction::Asr => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(Value::Int(b >> a));
                    } else { break; }
                }
                Instruction::Neg => {
                    if let Some(Value::Int(a)) = self.stack.pop() {
                        self.stack.push(Value::Int(-a));
                    } else { break; }
                }
                Instruction::Not => {
                    if let Some(Value::Int(a)) = self.stack.pop() {
                        self.stack.push(Value::Int(!a));
                    } else { break; }
                }
                Instruction::StPrint => {
                    if let Some(v) = self.stack.pop() {
                        match v {
                            Value::Str(s) => { print!("{}", s); io::stdout().flush().unwrap(); }
                            Value::Int(i) => { print!("{}", i); io::stdout().flush().unwrap(); }
                        }
                    } else { break; }
                }
                Instruction::Print => {
                    if let Some(v) = self.stack.pop() {
                        match v {
                            Value::Int(i) => println!("{}", i),
                            Value::Str(s) => println!("{}", s),
                        }
                    } else { break; }
                }
                Instruction::Call(addr) => {
                    self.stack.push(Value::Int(self.pc));
                    self.pc = addr;
                }
                Instruction::Ret => {
                    if let Some(Value::Int(a)) = self.stack.pop() {
                        self.pc = a;
                    } else { break; }
                }
                Instruction::Goto(addr) => {
                    self.pc = addr;
                }
                Instruction::IfEq(addr) => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        if b == a { self.pc = addr; }
                    } else { break; }
                }
                Instruction::IfNe(addr) => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        if b != a { self.pc = addr; }
                    } else { break; }
                }
                Instruction::IfLt(addr) => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        if b < a { self.pc = addr; }
                    } else { break; }
                }
                Instruction::IfLe(addr) => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        if b <= a { self.pc = addr; }
                    } else { break; }
                }
                Instruction::IfGt(addr) => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        if b > a { self.pc = addr; }
                    } else { break; }
                }
                Instruction::IfGe(addr) => {
                    if let (Some(Value::Int(a)), Some(Value::Int(b))) =
                        (self.stack.pop(), self.stack.pop()) {
                        if b >= a { self.pc = addr; }
                    } else { break; }
                }
                Instruction::If0(addr) => {
                    if let Some(Value::Int(a)) = self.stack.pop() {
                        if a == 0 { self.pc = addr; }
                    } else { break; }
                }
                Instruction::If1(addr) => {
                    if let Some(Value::Int(a)) = self.stack.pop() {
                        if a != 0 { self.pc = addr; }
                    } else { break; }
                }
                Instruction::Dump => {
                    println!("{:?}", self.stack);
                }
            }
            self.sp = self.stack.len();
        }
    }
}