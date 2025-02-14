use sdl2::sys::wchar_t;
use std::ops::Add;

use crate::bus::CpuBus;
use crate::errors::NesError;
use crate::memory::Mem;
use crate::opcodes::{AddressingMode, Instruction, OpCode, OpCodeDetail};
use crate::status;
use crate::status::Flag;

// TODO the program counter will be implemented incorrectly when using brk and the jmp commands because it always will increase by 1 afterwards but it should ignore it. Need to find best place to define.

pub mod stack;
pub mod trace;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: status::Status,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub bus: CpuBus,
}

impl CPU {
    pub fn new(bus: CpuBus) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: status::Status::new(),
            program_counter: 0,
            stack_pointer: 0xfd,
            bus,
        }
    }

    /// Reset the CPU to its default
    pub fn reset(&mut self) -> Result<(), NesError> {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = 0xfd;
        self.status.reset();

        self.program_counter = self.bus.mem_read_u16(0xfffc)?;

        Ok(())
    }

    /// We get the address in the memory that the address mode refers to.
    pub fn get_operand_address(&self, mode: &AddressingMode) -> Result<u16, NesError> {
        let program_counter = self.program_counter + 1;

        match mode {
            AddressingMode::Immediate => {
                // The program counter is the address of the next instruction, so this is what we immediately want.
                // For example LDA #$a9 we want to use a9 as the actual value. In this case the program counter may be 0x0002 and we know the value at 0x0002 is 0xa9.
                Ok(program_counter)
            }
            AddressingMode::ZeroPage => {
                // Here we have something like:
                // ```
                // LDA $a9
                // ```
                // In this case what we would like this function to return is 0xa9. We have the program counter which may be 0x0002 and we know that the value at 0x0002 is 0xa9, so we just need to read the value at the program counter.
                Ok(self.bus.mem_read(program_counter)? as u16)
            }
            AddressingMode::ZeroPageX => {
                // Here we have something like:
                // ```
                // LDX #$01
                // LDA $a1,X
                // ```
                // In this case we want to return 0xa2, because we take the 0xa1 and we add X to it (which is 0x01) to get 0xa2. Just like with zero page addressing we have the program counter like 0x0004, and if we read the value in memory at 0x0004 it is 0xa1, so we need to take the value at the program counter and add x to it.
                Ok(self
                    .bus
                    .mem_read(program_counter)?
                    .wrapping_add(self.register_x) as u16)
            }
            AddressingMode::ZeroPageY => Ok(self
                .bus
                .mem_read(program_counter)?
                .wrapping_add(self.register_y) as u16),
            AddressingMode::Absolute => Ok(self.bus.mem_read_u16(program_counter)?),
            AddressingMode::AbsoluteX => Ok(self
                .bus
                .mem_read_u16(program_counter)?
                .wrapping_add(self.register_x as u16)),
            AddressingMode::AbsoluteY => Ok(self
                .bus
                .mem_read_u16(program_counter)?
                .wrapping_add(self.register_y as u16)),
            AddressingMode::Indirect => {
                let address = self.bus.mem_read_u16(program_counter)?;
                Ok(self.bus.mem_read_u16_wrapping_boundary(address)?)
            }
            AddressingMode::IndirectX => {
                let address = self
                    .bus
                    .mem_read(program_counter)?
                    .wrapping_add(self.register_x) as u16;
                Ok(self.bus.mem_read_u16_wrapping_boundary(address)?)
            }
            AddressingMode::IndirectY => {
                let base = self.bus.mem_read(program_counter)? as u16;
                let address = self.bus.mem_read_u16_wrapping_boundary(base)?;
                Ok(address.wrapping_add(self.register_y as u16))
            }
            AddressingMode::Relative => Ok(program_counter),
            _ => Err(NesError::new("mode does not support getting an address")),
        }
    }

    pub fn get_operand_address_value(&self, mode: &AddressingMode) -> Result<u8, NesError> {
        match mode {
            AddressingMode::Accumulator => {
                return Ok(self.register_a);
            }
            AddressingMode::Implied => {
                return Err(NesError::new("mode Implied does not have a value"))
            }
            _ => (),
        };

        let address = self.get_operand_address(mode)?;

        Ok(self.bus.mem_read(address)?)
    }

    fn move_pointer_on_branch(&mut self, mode: &AddressingMode, bytes: u8) -> Result<(), NesError> {
        let value = self.get_operand_address_value(mode)?;

        // Signed value to know which direction the relative change is
        let signed_value = value as i8;

        // We now convert the signed value into an unsigned u16. We needed to do this rather than
        // converting straight to u16 because for instance -1 would be 1111_1111 in u8, but
        // 1111_1111_1111_1111 in u16.
        let unsigned_u16: u16 = signed_value as u16;

        self.apply_bytes_to_program_counter(bytes);

        let current_pointer = self.program_counter;

        // This I find slightly unintuitive, but because the negative numbers are larger in binary
        // (as the first digit is 1) then the wrapping means it does actually work correctly.
        let result = current_pointer.wrapping_add(unsigned_u16);

        self.program_counter = result;

        Ok(())
    }

    fn apply_bytes_to_program_counter(&mut self, bytes: u8) {
        self.program_counter = self.program_counter.wrapping_add(bytes as u16);
    }

    fn addition_with_register_a(&mut self, value: u16) {
        let initial_carry = self.status.read_flag(Flag::Carry) as u8;
        let result = (self.register_a as u16)
            .add(value)
            .add(initial_carry as u16);

        let [lo, hi] = u16::to_le_bytes(result);

        let overflow = ((self.register_a ^ lo) & ((value as u8) ^ lo) & 0b1000_0000) > 0;

        // Set the result in the accumulator
        self.register_a = lo;

        self.status.set_zero_flag(lo);
        self.status.set_negative_flag(lo);
        self.status.set_flag(Flag::Carry, hi > 0);
        self.status.set_flag(Flag::Overflow, overflow);
    }

    fn compare_to_memory(&mut self, value: u8, mode: &AddressingMode) -> Result<(), NesError> {
        let memory_value = self.get_operand_address_value(mode)?;

        let inverse_memory_value = (!memory_value as u16).wrapping_add(1);

        let result = inverse_memory_value.wrapping_add(value as u16);

        let [lo, hi] = u16::to_le_bytes(result);

        self.status.set_zero_flag(lo);
        self.status.set_negative_flag(lo);
        self.status.set_flag(Flag::Carry, hi > 0);

        Ok(())
    }

    fn check_boundary_crossed(&mut self, address: u16, value: u8) -> bool {
        let updated_address = address.wrapping_add(value as u16);

        let [_start_address_lo, start_address_hi] = u16::to_le_bytes(address);
        let [_updated_address_lo, updated_address_hi] = u16::to_le_bytes(updated_address);

        let crossed_page = updated_address_hi != start_address_hi;
        crossed_page
    }

    fn major_cycles(&mut self, mode: &AddressingMode) -> Result<u8, NesError> {
        match mode {
            AddressingMode::Immediate => Ok(2),
            AddressingMode::ZeroPage => Ok(3),
            AddressingMode::ZeroPageX => Ok(4),
            AddressingMode::Absolute => Ok(4),
            AddressingMode::AbsoluteX => {
                let address = self.bus.mem_read_u16(self.program_counter)?;

                let crossed_page = self.check_boundary_crossed(address, self.register_x);

                if crossed_page {
                    Ok(5)
                } else {
                    Ok(4)
                }
            }
            AddressingMode::AbsoluteY => {
                let address = self.bus.mem_read_u16(self.program_counter)?;

                let crossed_page = self.check_boundary_crossed(address, self.register_y);

                if crossed_page {
                    Ok(5)
                } else {
                    Ok(4)
                }
            }
            AddressingMode::IndirectX => Ok(6),
            AddressingMode::IndirectY => {
                let address = self.bus.mem_read(self.program_counter)?;

                let address = self.bus.mem_read_u16(address as u16)?;

                let crossed_page = self.check_boundary_crossed(address, self.register_y);

                if crossed_page {
                    Ok(6)
                } else {
                    Ok(5)
                }
            }
            _ => Err(NesError::new(
                "Trying to calculate cycles of an unsupported mode.",
            )),
        }
    }

    pub fn run(&mut self) -> Result<(), NesError> {
        self.run_with_callback(|_| {})?;
        Ok(())
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F) -> Result<(), NesError>
    where
        F: FnMut(&mut CPU),
    {
        let mut not_break = true;

        while not_break {
            let code = self.bus.mem_read(self.program_counter)?;
            let opcode = OpCodeDetail::from_opcode(&OpCode::from_code(&code)?);

            match opcode.instruction {
                Instruction::BRK => break,
                _ => {}
            };

            callback(self);

            self.run_opcode(&opcode)?;
        }

        Ok(())
    }

    pub fn run_opcode(&mut self, opcode: &OpCodeDetail) -> Result<(), NesError> {
        let OpCodeDetail {
            instruction,
            bytes,
            address_mode: mode,
            ..
        } = opcode;

        let bytes = *bytes;

        match instruction {
            Instruction::ADC => {
                let value = self.get_operand_address_value(&mode)?;

                self.addition_with_register_a(value as u16);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::AND => {
                let value = self.get_operand_address_value(&mode)?;

                let result = self.register_a & value;

                self.register_a = result;

                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::ASL => {
                let value = self.get_operand_address_value(&mode)?;

                let result = (value as u16) << 1;

                let [lo, hi] = u16::to_le_bytes(result);

                match mode {
                    AddressingMode::Accumulator => {
                        self.register_a = lo;
                    }
                    _ => {
                        let address = self.get_operand_address(&mode)?;

                        self.bus.mem_write(address, lo)?;
                    }
                }

                self.status.set_zero_flag(lo);
                self.status.set_negative_flag(lo);
                self.status.set_flag(Flag::Carry, hi > 0);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::BCC => {
                let carry = self.status.read_flag(Flag::Carry);

                if carry {
                    self.apply_bytes_to_program_counter(bytes);
                } else {
                    self.move_pointer_on_branch(&mode, bytes)?;
                }
            }
            Instruction::BCS => {
                let carry = self.status.read_flag(Flag::Carry);

                if !carry {
                    self.apply_bytes_to_program_counter(bytes);
                } else {
                    self.move_pointer_on_branch(&mode, bytes)?;
                }
            }
            Instruction::BEQ => {
                let zero = self.status.read_flag(Flag::Zero);

                if !zero {
                    self.apply_bytes_to_program_counter(bytes);
                } else {
                    self.move_pointer_on_branch(&mode, bytes)?;
                }
            }
            Instruction::BIT => {
                let value = self.get_operand_address_value(&mode)?;

                let and_result = self.register_a & value;

                self.status
                    .set_flag(Flag::Negative, (value & 0b1000_0000) > 0);
                self.status
                    .set_flag(Flag::Overflow, (value & 0b0100_0000) > 0);
                self.status.set_flag(Flag::Zero, and_result == 0);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::BMI => {
                let negative = self.status.read_flag(Flag::Negative);

                if !negative {
                    self.apply_bytes_to_program_counter(bytes);
                } else {
                    self.move_pointer_on_branch(&mode, bytes)?;
                }
            }
            Instruction::BNE => {
                let zero = self.status.read_flag(Flag::Zero);

                if zero {
                    self.apply_bytes_to_program_counter(bytes);
                } else {
                    self.move_pointer_on_branch(&mode, bytes)?;
                }
            }
            Instruction::BPL => {
                let negative = self.status.read_flag(Flag::Negative);

                if negative {
                    self.apply_bytes_to_program_counter(bytes);
                } else {
                    self.move_pointer_on_branch(&mode, bytes)?;
                }
            }
            Instruction::BRK => {
                self.push_to_stack_u16(self.program_counter + 2)?;

                let break_flag = self.status.read_flag(Flag::Break);

                self.status.set_flag(Flag::Break, true);
                self.push_to_stack(self.status.get_status_byte())?;

                self.status.set_flag(Flag::Break, break_flag);

                self.program_counter = self.bus.mem_read_u16(0xfffe)?;
            }
            Instruction::BVC => {
                let overflow = self.status.read_flag(Flag::Overflow);

                if overflow {
                    self.apply_bytes_to_program_counter(bytes);
                } else {
                    self.move_pointer_on_branch(&mode, bytes)?;
                }
            }
            Instruction::BVS => {
                let overflow = self.status.read_flag(Flag::Overflow);

                if !overflow {
                    self.apply_bytes_to_program_counter(bytes);
                } else {
                    self.move_pointer_on_branch(&mode, bytes)?;
                }
            }
            Instruction::CLC => {
                self.status.set_flag(Flag::Carry, false);
                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::CLD => {
                self.status.set_flag(Flag::Decimal, false);
                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::CLI => {
                self.status.set_flag(Flag::Interrupt, false);
                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::CLV => {
                self.status.set_flag(Flag::Overflow, false);
                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::CMP => {
                let accumulator = self.register_a;
                self.compare_to_memory(accumulator, &mode)?;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::CPX => {
                let accumulator = self.register_x;
                self.compare_to_memory(accumulator, &mode)?;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::CPY => {
                let accumulator = self.register_y;
                self.compare_to_memory(accumulator, &mode)?;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::DEC => {
                let value = self.get_operand_address_value(&mode)?;

                let result = self.status.set_decrement_flags(value);

                let address = self.get_operand_address(&mode)?;

                self.bus.mem_write(address, result)?;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::DEX => {
                let value = self.register_x;

                let result = self.status.set_decrement_flags(value);

                self.register_x = result;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::DEY => {
                let value = self.register_y;

                let result = self.status.set_decrement_flags(value);

                self.register_y = result;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::EOR => {
                let value = self.get_operand_address_value(&mode)?;

                let accumulator = self.register_a;

                let result = accumulator ^ value;

                self.register_a = result;

                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::INC => {
                let value = self.get_operand_address_value(&mode)?;

                let result = self.status.set_increment_flags(value);

                let address = self.get_operand_address(&mode)?;

                self.bus.mem_write(address, result)?;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::INX => {
                let value = self.register_x;

                let result = self.status.set_increment_flags(value);

                self.register_x = result;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::INY => {
                let value = self.register_y;

                let result = self.status.set_increment_flags(value);

                self.register_y = result;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::JMP => {
                self.jmp(&mode)?;
            }
            Instruction::JSR => {
                self.push_to_stack_u16(self.program_counter.wrapping_add(2))?;

                self.jmp(&mode)?;
            }
            Instruction::LDA => {
                let value = self.get_operand_address_value(&mode)?;

                self.register_a = value;
                self.status.set_zero_flag(value);
                self.status.set_negative_flag(value);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::LDX => {
                let value = self.get_operand_address_value(&mode)?;

                self.register_x = value;
                let result = self.register_x;
                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::LDY => {
                let value = self.get_operand_address_value(&mode)?;

                self.register_y = value;
                let result = self.register_y;
                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::LSR => {
                let value = self.get_operand_address_value(&mode)?;

                let carry_flag = value & 0b0000_0001;
                let result = value >> 1;

                match mode {
                    AddressingMode::Accumulator => {
                        self.register_a = result;
                    }
                    _ => {
                        let address = self.get_operand_address(&mode)?;

                        self.bus.mem_write(address, result)?;
                    }
                }

                self.status.set_flag(Flag::Negative, false);
                self.status.set_zero_flag(result);
                self.status.set_flag(Flag::Carry, carry_flag > 0);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::NOP => {
                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::ORA => {
                let value = self.get_operand_address_value(&mode)?;

                let result = self.register_a | value;

                self.register_a = result;
                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::PHA => {
                self.push_to_stack(self.register_a)?;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::PHP => {
                let break_flag = self.status.read_flag(Flag::Break);
                let ignored_flag = self.status.read_flag(Flag::Ignored);

                self.status.set_flag(Flag::Break, true);
                self.status.set_flag(Flag::Ignored, true);
                let status = self.status.get_status_byte();

                self.push_to_stack(status)?;

                self.status.set_flag(Flag::Break, break_flag);
                self.status.set_flag(Flag::Ignored, ignored_flag);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::PLA => {
                let result = self.pull_from_stack()?;

                self.register_a = result;

                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::PLP => {
                self.plp()?;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::ROL => {
                let value = self.get_operand_address_value(&mode)?;

                let carry_flag = value & 0b1000_0000;
                let result = (value << 1) | (self.status.read_flag(Flag::Carry) as u8);

                match &mode {
                    AddressingMode::Accumulator => {
                        self.register_a = result;
                    }
                    _ => {
                        let address = self.get_operand_address(&mode)?;

                        self.bus.mem_write(address, result)?;
                    }
                }

                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);
                self.status.set_flag(Flag::Carry, carry_flag > 0);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::ROR => {
                let value = self.get_operand_address_value(&mode)?;

                let carry_flag = value & 0b0000_0001;
                let result = (value >> 1) | ((self.status.read_flag(Flag::Carry) as u8) << 7);

                match &mode {
                    AddressingMode::Accumulator => {
                        self.register_a = result;
                    }
                    _ => {
                        let address = self.get_operand_address(&mode)?;

                        self.bus.mem_write(address, result)?;
                    }
                }

                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);
                self.status.set_flag(Flag::Carry, carry_flag > 0);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::RTI => {
                self.plp()?;

                let program_counter = self.pull_from_stack_u16()?;

                self.program_counter = program_counter;
            }
            Instruction::RTS => {
                let program_counter = self.pull_from_stack_u16()?;

                self.program_counter = program_counter + 1
            }
            Instruction::SBC => {
                let value = self.get_operand_address_value(&mode)?;

                let value = !value;

                self.addition_with_register_a(value as u16);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::SEC => {
                self.status.set_flag(Flag::Carry, true);
                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::SED => {
                self.status.set_flag(Flag::Decimal, true);
                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::SEI => {
                self.status.set_flag(Flag::Interrupt, true);
                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::STA => {
                let address = self.get_operand_address(&mode)?;

                self.bus.mem_write(address, self.register_a)?;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::STX => {
                let address = self.get_operand_address(&mode)?;

                self.bus.mem_write(address, self.register_x)?;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::STY => {
                let address = self.get_operand_address(&mode)?;

                self.bus.mem_write(address, self.register_y)?;

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::TAX => {
                let result = self.register_a;

                self.register_x = result;

                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::TAY => {
                let result = self.register_a;

                self.register_y = result;

                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::TSX => {
                let result = self.stack_pointer;

                self.register_x = result;

                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::TXA => {
                let result = self.register_x;

                self.register_a = result;

                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::TXS => {
                let result = self.register_x;

                self.stack_pointer = result;

                // self.status.set_zero_flag(result);
                // self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
            Instruction::TYA => {
                let result = self.register_y;

                self.register_a = result;

                self.status.set_zero_flag(result);
                self.status.set_negative_flag(result);

                self.apply_bytes_to_program_counter(bytes);
            }
        };

        Ok(())
    }

    fn plp(&mut self) -> Result<(), NesError> {
        let break_flag = self.status.read_flag(Flag::Break);
        let ignored_flag = self.status.read_flag(Flag::Ignored);

        let result = self.pull_from_stack()?;

        self.status.set_from_byte(result);

        self.status.set_flag(Flag::Break, break_flag);
        self.status.set_flag(Flag::Ignored, ignored_flag);

        Ok(())
    }

    fn jmp(&mut self, mode: &AddressingMode) -> Result<(), NesError> {
        let address = self.get_operand_address(&mode)?;

        self.program_counter = address;

        Ok(())
    }
}
