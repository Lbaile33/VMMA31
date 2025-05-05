// Updated main.rs
mod instruction;
mod vm;

use std::env;
use std::fs::File;
use std::io::Read;
use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <bytecode file>", args[0]);
        return;
    }
    let filename = &args[1];
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file {}: {}", filename, e);
            return;
        }
    };
    let mut data = Vec::new();
    if let Err(e) = file.read_to_end(&mut data) {
        eprintln!("Error reading file: {}", e);
        return;
    }
    if data.len() < 4 {
        eprintln!("File too short");
        return;
    }
    // Check magic number 0xDEADBEEF
    if data[0] != 0xDE || data[1] != 0xAD || data[2] != 0xBE || data[3] != 0xEF {
        eprintln!("Invalid magic number");
        return;
    }
    let mut memory = [0u8; 4096];
    let code = &data[4..];
    if code.len() > 4096 {
        eprintln!("Input file too large");
        return;
    }
    
    // Load code into memory in the correct order for backward execution
    // Start from the end of memory and work backward
    let start_pos = 4095;
    for i in 0..code.len() {
        memory[start_pos - i] = code[code.len() - 1 - i];
    }
    
    
    let mut vm = VM::new(memory);
    // Set the PC to the correct starting point for the first instruction
    vm.pc = start_pos as i32;
    vm.run();
}