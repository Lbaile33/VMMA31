use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::process;

const RAM_SIZE: usize = 4096;
const MAGIC: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];

struct VM {
    memory: [u8; RAM_SIZE],
    pc: usize,     // Program counter
    sp: usize,     // Stack pointer
    exited: bool,  // Exit flag
    exit_code: i32, // Exit code
    code_size: usize, // Size of the loaded bytecode
}

impl VM {
    fn new() -> VM {
        VM {
            memory: [0u8; RAM_SIZE],
            pc: 0,
            sp: RAM_SIZE, // Stack starts at the bottom (4096)
            exited: false,
            exit_code: 0,
            code_size: 0, // Initialize to 0, will be set in load_file
        }
    }

    // Load bytecode file into memory, excluding magic bytes
    fn load_file(&mut self, filename: &str) -> Result<(), String> {
        let file = File::open(filename).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut reader = BufReader::new(file);

        // Check magic bytes
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic).map_err(|e| format!("Failed to read magic bytes: {}", e))?;
        if magic != MAGIC {
            return Err(format!("Invalid magic bytes: {:?}", magic));
        }

        // Read the rest of the file into memory and track code size
        let bytes_read = reader.read(&mut self.memory).map_err(|e| format!("Failed to read file: {}", e))?;
        if bytes_read > RAM_SIZE {
            return Err("File too large for memory".to_string());
        }
        self.code_size = bytes_read; // Store the size of the loaded bytecode

        Ok(())
    }

    // Run the virtual machine
    fn run(&mut self) -> i32 {
        while self.pc < self.code_size && !self.exited {
            let instruction = self.read_u32(self.pc);
            let pc_before = self.pc;
            self.execute_instruction(instruction);
            
            // Only increment PC if it wasn't modified by the instruction
            if self.pc == pc_before && !self.exited {
                self.pc += 4; // Instructions are 4 bytes
            }
        }
        self.exit_code
    }

    // Read a 4-byte little-endian u32 from memory at the given address
    fn read_u32(&self, addr: usize) -> u32 {
        if addr + 3 >= RAM_SIZE {
            return 0; // Return 0 if out of bounds
        }
        u32::from_le_bytes(self.memory[addr..addr + 4].try_into().unwrap())
    }

    // Write a 4-byte u32 to memory at the given address in little-endian
    fn write_u32(&mut self, addr: usize, value: u32) {
        if addr + 3 < RAM_SIZE {
            self.memory[addr..addr + 4].copy_from_slice(&value.to_le_bytes());
        }
    }

    // Push a value onto the stack
    fn push(&mut self, value: u32) {
        if self.sp >= 4 { // Prevent underflow
            self.sp -=
             4;
            self.write_u32(self.sp, value);
        }
    }

    // Pop a value from the stack
    fn pop(&mut self) -> u32 {
        if self.sp + 4 <= RAM_SIZE { // Prevent overflow
            let value = self.read_u32(self.sp);
            self.sp += 4;
            value
        } else {
            0 // Return 0 if stack is empty
        }
    }

    // Peek a value from the stack at sp + offset
    fn peek(&self, offset: i32) -> u32 {
        let addr = (self.sp as i32 + offset) as usize;
        if addr + 3 < RAM_SIZE {
            self.read_u32(addr)
        } else {
            0
        }
    }

    // Execute a single instruction
    fn execute_instruction(&mut self, instruction: u32) {
        let opcode = (instruction >> 28) & 0xF;

        match opcode {
            0 => self.exec_miscellaneous(instruction),
            1 => self.exec_pop(instruction),
            2 => self.exec_binary_arithmetic(instruction),
            3 => self.exec_unary_arithmetic(instruction),
            4 => self.exec_stprint(instruction),
            5 => self.exec_call(instruction),
            6 => self.exec_return(instruction),
            7 => self.exec_goto(instruction),
            8 => self.exec_binary_if(instruction),
            9 => self.exec_unary_if(instruction),
            12 => self.exec_dup(instruction),
            13 => self.exec_print(instruction),
            14 => self.exec_dump(),
            15 => self.exec_push(instruction),
            _ => {} // Unknown opcode, ignore
        }
    }

    fn exec_miscellaneous(&mut self, instruction: u32) {
        let subopcode = (instruction >> 24) & 0xF;
        match subopcode {
            0 => { // exit [code]
                let code = instruction & 0xFFF; // Only 12 bits for exit code
                self.exit_code = code as i32;
                self.exited = true;
            }
            1 => { // swap [from] [to]
                let from_raw = (instruction >> 12) & 0xFFF;
                let to_raw = instruction & 0xFFF;
                // Sign-extend 12-bit values and multiply by 4 (word offsets)
                let from = ((from_raw as i32) << 20 >> 20) * 4;
                let to = ((to_raw as i32) << 20 >> 20) * 4;
                let addr1 = (self.sp as i32 + from) as usize;
                let addr2 = (self.sp as i32 + to) as usize;
                if addr1 + 3 < RAM_SIZE && addr2 + 3 < RAM_SIZE {
                    let val1 = self.read_u32(addr1);
                    let val2 = self.read_u32(addr2);
                    self.write_u32(addr1, val2);
                    self.write_u32(addr2, val1);
                }
            }
            2 => {} // nop
            4 => { // input
                print!(""); // Flush any pending output
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read input");
                let input = input.trim();
                
                let value = if input.starts_with("0x") || input.starts_with("0X") {
                    // Parse hex
                    i32::from_str_radix(&input[2..], 16)
                } else if input.starts_with("0b") || input.starts_with("0B") {
                    // Parse binary
                    i32::from_str_radix(&input[2..], 2)
                } else {
                    // Parse decimal
                    input.parse::<i32>()
                }.unwrap_or(0);
                
                self.push(value as u32);
            }
            5 => { // stinput [max_chars]
                print!(""); // Flush any pending output
                io::stdout().flush().unwrap();
                
                let max_chars = instruction & 0xFFFFFF;
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read input");
                let trimmed = input.trim();
                
                if trimmed.is_empty() {
                    self.push(0);
                } else {
                    let bytes = trimmed.as_bytes();
                    let len = if max_chars == 0xFFFFFF { bytes.len() } else { bytes.len().min(max_chars as usize) };
                    
                    let mut chunks = Vec::new();
                    let mut start = 0;
                    while start < len {
                        let end = (start + 3).min(len);
                        let mut value = [0u8; 4];
                        for i in 0..(end - start) {
                            value[i] = bytes[start + i];
                        }
                        value[3] = if end < len { 0x01 } else { 0x00 };
                        chunks.push(u32::from_le_bytes(value));
                        start += 3;
                    }
                    
                    // Push chunks in reverse order
                    for &chunk in chunks.iter().rev() {
                        self.push(chunk);
                    }
                }
            }
            _ => {} // debug or unknown, ignore
        }
    }

    fn exec_pop(&mut self, instruction: u32) {
        let offset = (instruction >> 2) & 0x3FFFFFF;
        let offset = offset as usize * 4; // Multiply by 4 for byte addressing
        
        if self.sp + offset <= RAM_SIZE {
            self.sp += offset;
        } else {
            self.sp = RAM_SIZE;
        }
    }

    fn exec_binary_arithmetic(&mut self, instruction: u32) {
        let subopcode = (instruction >> 24) & 0xF;
        let right = self.pop() as i32;
        let left = self.pop() as i32;
        let result = match subopcode {
            0 => left + right,    // add
            1 => left - right,    // sub
            2 => left * right,    // mul
            3 => if right != 0 { left / right } else { 0 },    // div
            4 => if right != 0 { left % right } else { 0 },    // rem
            5 => left & right,    // and
            6 => left | right,    // or
            7 => left ^ right,    // xor
            8 => left << right,   // lsl
            9 => (left as u32 >> right as u32) as i32, // lsr
            11 => left >> right,  // asr
            _ => 0
        };
        self.push(result as u32);
    }

    fn exec_unary_arithmetic(&mut self, instruction: u32) {
        let subopcode = (instruction >> 24) & 0xF;
        let value = self.pop() as i32;
        let result = match subopcode {
            0 => -value, // neg
            1 => !value, // not
            _ => 0
        };
        self.push(result as u32);
    }

    fn exec_stprint(&mut self, instruction: u32) {
        let offset_raw = (instruction >> 2) & 0x3FFFFFF; // Bits 27:2
        let offset = if (offset_raw & (1 << 25)) != 0 {
            ((offset_raw | 0xFC000000) as i32) * 4 // Sign-extend from bit 25
        } else {
            (offset_raw as i32) * 4
        };
        
        let mut addr = (self.sp as i32 + offset) as usize;
        while addr < RAM_SIZE {
            let byte = self.memory[addr];
            if byte == 0 {
                break;
            } else if byte != 1 { // Skip continuation byte
                print!("{}", byte as char);
            }
            addr += 1;
        }
        io::stdout().flush().unwrap();
    }

    fn exec_call(&mut self, instruction: u32) {
        let offset_raw = (instruction >> 2) & 0x3FFFFFF;
        let offset = if (offset_raw & (1 << 25)) != 0 {
            ((offset_raw | 0xFC000000) as i32) * 4
        } else {
            (offset_raw as i32) * 4
        };
        
        self.push((self.pc + 4) as u32); // Push next instruction address
        self.pc = ((self.pc as i32) + offset) as usize;
    }

    fn exec_return(&mut self, instruction: u32) {
        let offset_raw = (instruction >> 2) & 0x3FFFFFF;
        let offset = if (offset_raw & (1 << 25)) != 0 {
            ((offset_raw | 0xFC000000) as i32) * 4
        } else {
            (offset_raw as i32) * 4
        };
        
        if offset as usize > 0 && self.sp + (offset as usize) <= RAM_SIZE {
            self.sp += offset as usize;
        }
        
        if self.sp < RAM_SIZE {
            self.pc = self.pop() as usize;
        }
    }

    fn exec_goto(&mut self, instruction: u32) {
        let offset_raw = (instruction >> 2) & 0x3FFFFFF;
        let offset = if (offset_raw & (1 << 25)) != 0 {
            ((offset_raw | 0xFC000000) as i32) * 4
        } else {
            (offset_raw as i32) * 4
        };
        
        self.pc = ((self.pc as i32) + offset) as usize;
    }

    fn exec_binary_if(&mut self, instruction: u32) {
        let condition = (instruction >> 25) & 0x7;
        let offset_raw = (instruction >> 2) & 0x7FFFFF; // 23 bits for PC relative offset
        let offset = if (offset_raw & (1 << 22)) != 0 {
            ((offset_raw | 0xFF800000) as i32) * 4
        } else {
            (offset_raw as i32) * 4
        };
        
        let right = self.peek(0) as i32;
        let left = self.peek(4) as i32;
        let condition_met = match condition {
            0 => left == right,  // eq
            1 => left != right,  // ne
            2 => left < right,   // lt
            3 => left > right,   // gt
            4 => left <= right,  // le
            5 => left >= right,  // ge
            _ => false
        };
        
        if condition_met {
            self.pc = ((self.pc as i32) + offset) as usize;
        }
    }

    fn exec_unary_if(&mut self, instruction: u32) {
        let condition = (instruction >> 24) & 0x3; // Bits 25:24 contain the condition
        let offset_raw = (instruction >> 2) & 0x3FFFFF; // 22 bits for PC relative offset
        let offset = if (offset_raw & (1 << 21)) != 0 {
            ((offset_raw | 0xFFC00000) as i32) * 4
        } else {
            (offset_raw as i32) * 4
        };
        
        let value = self.peek(0) as i32;
        let condition_met = match condition {
            0 => value == 0,  // ez (equals zero)
            1 => value != 0,  // nz (not zero)
            2 => value < 0,   // mi (minus/negative)
            3 => value >= 0,  // pl (plus/positive or zero)
            _ => false
        };
        
        if condition_met {
            self.pc = ((self.pc as i32) + offset) as usize;
        }
    }

    fn exec_dup(&mut self, instruction: u32) {
        let offset_raw = (instruction >> 2) & 0x3FFFFFF;
        let offset = if (offset_raw & (1 << 25)) != 0 {
            ((offset_raw | 0xFC000000) as i32) * 4
        } else {
            (offset_raw as i32) * 4
        };
        
        let value = self.peek(offset);
        self.push(value);
    }

    fn exec_print(&mut self, instruction: u32) {
        let offset_raw = (instruction >> 2) & 0x3FFFFFF;
        let offset = if (offset_raw & (1 << 25)) != 0 {
            ((offset_raw | 0xFC000000) as i32) * 4
        } else {
            (offset_raw as i32) * 4
        };
        
        let fmt = instruction & 0x3;
        let value = self.peek(offset) as i32;
        match fmt {
            0 => println!("{}", value),              // decimal
            1 => println!("0x{:x}", value),         // hex
            2 => println!("0b{:b}", value),         // binary
            3 => println!("0o{:o}", value),         // octal
            _ => {}
        }
    }

    fn exec_dump(&mut self) {
        if self.sp >= RAM_SIZE {
            return; // Stack empty
        }
        let mut addr = self.sp;
        while addr < RAM_SIZE {
            let value = self.read_u32(addr);
            println!("{:04x}: {:08x}", addr - self.sp, value);
            addr += 4;
        }
    }

    fn exec_push(&mut self, instruction: u32) {
        let value_raw = instruction & 0x0FFFFFFF; // Extract bits 27:0
        let value = if (value_raw & (1 << 27)) != 0 {
            (value_raw | 0xF0000000) as i32 // Sign-extend if bit 27 is set
        } else {
            value_raw as i32
        };
        self.push(value as u32);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <bytecode_file>", args[0]);
        process::exit(1);
    }

    let mut vm = VM::new();
    if let Err(e) = vm.load_file(&args[1]) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    let exit_code = vm.run();
    process::exit(exit_code);
}