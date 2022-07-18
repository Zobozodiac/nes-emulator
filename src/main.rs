extern crate core;

struct Memory {
    storage: [u8; 0xFFFF],
}

impl Memory {
    pub fn new(storage: [u8; 0xFFFF]) -> Self {
        Memory {
            storage
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.storage[addr as usize] = data;
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let [lo, hi] = data.to_le_bytes();
        self.mem_write(pos, lo);
        self.mem_write(pos.wrapping_add(1), hi);
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.storage[addr as usize]
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos);
        let hi = self.mem_read(pos.wrapping_add(1));

        u16::from_le_bytes([lo, hi])
    }

    fn load_program(&mut self, program: Vec<u8>) {
        self.storage[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
    }
}

enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    NoneAddressing,
}

// struct OpCode {
//     code: u8,
//     name: String,
//     bytes: i8,
//     cycles: i8,
//     address_mode: AddressingMode,
// }
//
// impl OpCode {
//     pub fn new(code: u8, name: String, bytes: i8, cycles: i8, address_mode: AddressingMode) -> Self {
//         OpCode {
//             code,
//             name,
//             bytes,
//             cycles,
//             address_mode
//         }
//     }
// }

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: Memory,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: Memory::new([0; 0xFFFF]),
        }
    }

    /// We get the address in the memory that the address mode refers to.
    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => {
                // The program counter is the address of the next instruction, so this is what we immediately want.
                // For example LDA #$a9 we want to use a9 as the actual value. In this case the program counter may be 0x0002 and we know the value at 0x0002 is 0xa9.
                self.program_counter
            }
            AddressingMode::ZeroPage => {
                // Here we have something like:
                // ```
                // LDA $a9
                // ```
                // In this case what we would like this function to return is 0xa9. We have the program counter which may be 0x0002 and we know that the value at 0x0002 is 0xa9, so we just need to read the value at the program counter.
                self.memory.mem_read(self.program_counter) as u16
            }
            AddressingMode::ZeroPageX => {
                // Here we have something like:
                // ```
                // LDX #$01
                // LDA $a1,X
                // ```
                // In this case we want to return 0xa2, because we take the 0xa1 and we add X to it (which is 0x01) to get 0xa2. Just like with zero page addressing we have the program counter like 0x0004, and if we read the value in memory at 0x0004 it is 0xa1, so we need to take the value at the program counter and add x to it.
                self.memory.mem_read(self.program_counter).wrapping_add(self.register_x) as u16
            }
            AddressingMode::ZeroPageY => {
                self.memory.mem_read(self.program_counter).wrapping_add(self.register_y) as u16
            }
            AddressingMode::Absolute => {
                self.memory.mem_read_u16(self.program_counter)
            }
            AddressingMode::AbsoluteX => {
                self.memory.mem_read_u16(self.program_counter).wrapping_add(self.register_x as u16)
            }
            AddressingMode::AbsoluteY => {
                self.memory.mem_read_u16(self.program_counter).wrapping_add(self.register_y as u16)
            }
            AddressingMode::Indirect => {
                let address = self.memory.mem_read_u16(self.program_counter);
                self.memory.mem_read_u16(address)
            }
            AddressingMode::IndirectX => {
                let address = self.memory.mem_read(self.program_counter).wrapping_add(self.register_x) as u16;
                self.memory.mem_read_u16(address)
            }
            AddressingMode::IndirectY => {
                let base = self.memory.mem_read(self.program_counter) as u16;
                let address = self.memory.mem_read_u16(base);
                address.wrapping_add(self.register_y as u16)
            }
            AddressingMode::NoneAddressing => {
                panic!("mode NoneAddressing is not supported");
            }
        }
    }


    fn lda(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);
        let value = self.memory.mem_read(address);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // Here we set the zero flag. The 0bxxxx_xxxx parts are literally to do bitwise AND or OR to set that particular bit of the status flag.
        if result == 0 {
            self.status = self.status | 0b000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        // Here we set the negative flag, the definition of negative in this case is that the first bit is 1.
        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory.load_program(program);
        self.memory.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {
        loop {
            let opscode = self.memory.mem_read(self.program_counter);
            self.program_counter += 1;

            match opscode {
                0xa9 => {
                    self.lda(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xa5 => {
                    self.lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0xad => {
                    self.lda(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x00 => {
                    return;
                }
                0xaa => {
                    self.tax();
                }
                _ => todo!()
            }
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;

        self.program_counter = self.memory.mem_read_u16(0xFFFC);
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);

        assert_eq!(cpu.register_a, 0x05);
        assert_eq!(cpu.status & 0b0000_0010, 0b00);
        assert_eq!(cpu.status & 0b1000_0000, 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);

        assert_eq!(cpu.status & 0b0000_0010, 0b10);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 10, 0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10);
    }

    #[test]
    fn test_mem_write() {
        let mut memory = Memory::new([0; 0xFFFF]);
        memory.mem_write(0x0000, 0x12);

        assert_eq!(memory.storage[0x0000], 0x12);
    }

    #[test]
    fn test_mem_write_u16() {
        let mut memory = Memory::new([0; 0xFFFF]);
        memory.mem_write_u16(0x0000, 0x1234);

        assert_eq!(memory.storage[0x0000], 0x34);
        assert_eq!(memory.storage[0x0001], 0x12);
    }

    #[test]
    fn test_mem_read() {
        let mut memory = Memory::new([0; 0xFFFF]);
        memory.mem_write(0x0000, 0x12);

        assert_eq!(memory.mem_read(0x0000), 0x12);
    }

    #[test]
    fn test_mem_read_u16() {
        let mut memory = Memory::new([0; 0xFFFF]);
        memory.mem_write_u16(0x0000, 0x1234);

        assert_eq!(memory.mem_read_u16(0x0000), 0x1234);
    }

    #[test]
    fn test_mem_load_program() {
        let mut memory = Memory::new([0; 0xFFFF]);
        let program = vec![0xa9];
        memory.load_program(program);

        assert_eq!(memory.mem_read(0x8000), 0xa9);
    }
}
