use opcodes::{AddressingMode, OpCode};

mod memory;

mod opcodes;

pub struct CPU {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: u8,
    program_counter: u16,
    memory: memory::Memory,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: memory::Memory::new([0; 0xFFFF]),
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
            let code = self.memory.mem_read(self.program_counter);
            self.program_counter += 1;

            let opcode = opcodes::OPSCODES_MAP.get(&code);

            match opcode {
                Some(OpCode {name: "BRK", ..}) => {
                    return
                },
                Some(OpCode {name: "LDA", cycles, address_mode: mode, ..}) => {
                    self.lda(mode);
                    self.program_counter += *cycles as u16;
                },
                _ => {
                    todo!()
                }
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

    // #[test]
    // fn test_0xaa_tax_move_a_to_x() {
    //     let mut cpu = CPU::new();
    //     cpu.load_and_run(vec![0xa9, 10, 0xaa, 0x00]);
    //
    //     assert_eq!(cpu.register_x, 10);
    // }
}
