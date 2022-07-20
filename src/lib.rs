use crate::status::Flag;
use opcodes::{AddressingMode, OpCode};
use std::ops::Add;

pub mod memory;

pub mod opcodes;

pub mod status;

pub struct CPU {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: status::Status,
    program_counter: u16,
    memory: memory::Memory,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: status::Status::new(),
            program_counter: 0,
            memory: memory::Memory::new([0; 0xFFFF]),
        }
    }

    /// Reset the CPU to its default
    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = status::Status::new();

        self.program_counter = self.memory.mem_read_u16(0xFFFC);
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
                self.memory
                    .mem_read(self.program_counter)
                    .wrapping_add(self.register_x) as u16
            }
            AddressingMode::ZeroPageY => self
                .memory
                .mem_read(self.program_counter)
                .wrapping_add(self.register_y) as u16,
            AddressingMode::Absolute => self.memory.mem_read_u16(self.program_counter),
            AddressingMode::AbsoluteX => self
                .memory
                .mem_read_u16(self.program_counter)
                .wrapping_add(self.register_x as u16),
            AddressingMode::AbsoluteY => self
                .memory
                .mem_read_u16(self.program_counter)
                .wrapping_add(self.register_y as u16),
            AddressingMode::Indirect => {
                let address = self.memory.mem_read_u16(self.program_counter);
                self.memory.mem_read_u16(address)
            }
            AddressingMode::IndirectX => {
                let address = self
                    .memory
                    .mem_read(self.program_counter)
                    .wrapping_add(self.register_x) as u16;
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

    fn get_operand_address_value(&mut self, mode: &AddressingMode) -> u8 {
        let address = self.get_operand_address(mode);
        let value = self.memory.mem_read(address);

        value
    }

    /// Add a value and the Accumulator together with a Carry.
    ///
    /// Setting overflow is the hardest thing here, but essentially the 8th bit encodes if a number
    /// is positive or negative. So we are comparing the result and saying if both values are
    /// positive but the result is negative then we have overflow, and vice versa if both are
    /// negative but the result is positive then we have overflow.
    fn adc(&mut self, mode: &AddressingMode) {
        let value = self.get_operand_address_value(mode);

        let initial_carry = self.status.read_flag(Flag::Carry) as u8;
        let result = (self.register_a as u16)
            .add(value as u16)
            .add(initial_carry as u16);

        let [lo, hi] = u16::to_le_bytes(result);

        let overflow = ((self.register_a ^ lo) & (value ^ lo) & 0b1000_0000) > 0;

        // Set the result in the accumulator
        self.register_a = lo;

        self.status.set_zero_flag(lo);
        self.status.set_negative_flag(lo);
        self.status.set_flag(Flag::Carry, hi > 0);
        self.status.set_flag(Flag::Overflow, overflow);
    }

    /// Load Accumulator
    fn lda(&mut self, mode: &AddressingMode) {
        let value = self.get_operand_address_value(mode);

        self.register_a = value;
        let result = self.register_a;
        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
    }

    /// Transfer Accumulator to Index X
    fn tax(&mut self) {
        self.register_x = self.register_a;
        let result = self.register_x;
        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
    }

    /// Bitwise OR with a value and the Accumulator
    fn ora(&mut self, mode: &AddressingMode) {
        let value = self.get_operand_address_value(mode);

        self.register_a = self.register_a | value;
        let result = self.register_a;
        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
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

            let OpCode {
                name,
                cycles,
                address_mode: mode,
                ..
            } = match opcode {
                Some(valid_opcode) => valid_opcode,
                None => panic!("OpCode not found in HashMap."),
            };

            match *name {
                "BRK" => {
                    return;
                }
                "LDA" => {
                    self.lda(mode);
                }
                _ => {
                    todo!()
                }
            }

            self.program_counter += *cycles as u16;
        }
    }
}

#[cfg(test)]
mod test_address_modes {
    use super::*;

    #[test]
    fn test_immediate() {
        let mut cpu = CPU::new();

        let address = cpu.get_operand_address(&AddressingMode::Immediate);

        assert_eq!(address, 0x0000)
    }

    #[test]
    fn test_zero_page() {
        // The program counter is pointing to 0x00 so we set this to 0x12 and check it works
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x00, 0x12);

        let address = cpu.get_operand_address(&AddressingMode::ZeroPage);

        assert_eq!(address, 0x0012)
    }

    #[test]
    fn test_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x01;
        cpu.memory.mem_write(0x00, 0x12);

        let address = cpu.get_operand_address(&AddressingMode::ZeroPageX);

        assert_eq!(address, 0x0013)
    }

    #[test]
    fn test_zero_page_y() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x01;
        cpu.memory.mem_write(0x00, 0x12);

        let address = cpu.get_operand_address(&AddressingMode::ZeroPageY);

        assert_eq!(address, 0x0013)
    }

    #[test]
    fn test_absolute() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write_u16(0x00, 0x1234);

        let address = cpu.get_operand_address(&AddressingMode::Absolute);

        assert_eq!(address, 0x1234)
    }

    #[test]
    fn test_absolute_x() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x01;
        cpu.memory.mem_write_u16(0x00, 0x1234);

        let address = cpu.get_operand_address(&AddressingMode::AbsoluteX);

        assert_eq!(address, 0x1235)
    }

    #[test]
    fn test_absolute_y() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x01;
        cpu.memory.mem_write_u16(0x00, 0x1234);

        let address = cpu.get_operand_address(&AddressingMode::AbsoluteY);

        assert_eq!(address, 0x1235)
    }

    #[test]
    fn test_indirect() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write_u16(0x00, 0x1234);
        cpu.memory.mem_write_u16(0x1234, 0x5678);

        let address = cpu.get_operand_address(&AddressingMode::Indirect);

        assert_eq!(address, 0x5678)
    }

    #[test]
    fn test_indirect_x() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x01;
        cpu.memory.mem_write(0x00, 0x12);
        cpu.memory.mem_write_u16(0x13, 0x3456);

        let address = cpu.get_operand_address(&AddressingMode::IndirectX);

        assert_eq!(address, 0x3456)
    }

    #[test]
    fn test_indirect_y() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x01;
        cpu.memory.mem_write_u16(0x00, 0x12);
        cpu.memory.mem_write_u16(0x12, 0x3456);

        let address = cpu.get_operand_address(&AddressingMode::IndirectY);

        assert_eq!(address, 0x3457)
    }
}

#[cfg(test)]
mod test_opcodes {
    use super::*;
    use crate::status::Flag;

    #[test]
    fn test_adc() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x12;
        cpu.memory.mem_write(0x0000, 0x34);

        cpu.adc(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x46);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), false);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), false);
    }

    #[test]
    fn test_adc_zero() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x00;
        cpu.memory.mem_write(0x0000, 0x00);

        cpu.adc(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.status.read_flag(Flag::Zero), true);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), false);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), false);
    }

    #[test]
    fn test_adc_negative() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1000_1010;
        cpu.memory.mem_write(0x0000, 0b0000_0001);

        cpu.adc(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0b1000_1011);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), true);
        assert_eq!(cpu.status.read_flag(Flag::Carry), false);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), false);
    }

    #[test]
    fn test_adc_carry() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1100_1010;
        cpu.memory.mem_write(0x0000, 0b0100_0001);

        cpu.adc(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0b0000_1011);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), false);
    }

    #[test]
    fn test_adc_overflow() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1000_1010;
        cpu.memory.mem_write(0x0000, 0b1000_0001);

        cpu.adc(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0b0000_1011);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), true);
    }

    #[test]
    fn test_lda() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x0000, 0x12);

        cpu.lda(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0x12);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_ora() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0000_0001;
        cpu.memory.mem_write(0x0000, 0b0000_0010);

        cpu.ora(&AddressingMode::Immediate);

        assert_eq!(cpu.register_a, 0b0000_0011);
    }
}
