use std::ops::Add;

use crate::bus::Bus;
use crate::cartridge::{Cartridge, CHR_ROM_PAGE_SIZE, PRG_ROM_PAGE_SIZE};
use crate::memory::Mem;
use crate::opcodes;
use crate::opcodes::{AddressingMode, Instruction, OpCode, OpCodeDetail};
use crate::status;
use crate::status::Flag;

// TODO the program counter will be implemented incorrectly when using brk and the jmp commands because it always will increase by 1 afterwards but it should ignore it. Need to find best place to define.

pub struct CPU {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: status::Status,
    program_counter: u16,
    stack_pointer: u8,
    memory: Bus,
}

impl CPU {
    pub fn new(rom: Cartridge) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: status::Status::new(),
            program_counter: 0,
            stack_pointer: 0xff,
            memory: Bus::new(rom),
        }
    }

    /// Reset the CPU to its default
    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = 0xff;
        self.status = status::Status::new();

        self.program_counter = self.memory.mem_read_u16(0xfffc);
    }

    pub fn mem_write(&mut self, address: u8, value: u8) {
        self.memory.mem_write(address as u16, value);
    }

    pub fn mem_read(&self, address: u16) -> u8 {
        self.memory.mem_read(address)
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
            AddressingMode::Relative => self.program_counter,
            _ => {
                panic!("mode does not support getting an address");
            }
        }
    }

    fn get_stack_address(&self) -> u16 {
        u16::from_le_bytes([self.stack_pointer, 0x01])
    }

    fn push_to_stack(&mut self, data: u8) {
        let stack_address = self.get_stack_address();

        self.memory.mem_write(stack_address, data);
        self.stack_pointer = self.stack_pointer - 1;
    }

    fn push_to_stack_u16(&mut self, data: u16) {
        let [lo, hi] = u16::to_le_bytes(data);

        self.push_to_stack(hi);
        self.push_to_stack(lo);
    }

    fn pull_from_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer + 1;
        let stack_address = self.get_stack_address();

        return self.memory.mem_read(stack_address);
    }

    fn pull_from_stack_u16(&mut self) -> u16 {
        let lo = self.pull_from_stack();
        let hi = self.pull_from_stack();

        return u16::from_le_bytes([lo, hi]);
    }

    fn get_operand_address_value(&mut self, mode: &AddressingMode) -> u8 {
        match mode {
            AddressingMode::Accumulator => {
                return self.register_a;
            }
            AddressingMode::Implied => {
                panic!("mode Implied does not have a value");
            }
            _ => (),
        };

        let address = self.get_operand_address(mode);
        let value = self.memory.mem_read(address);

        value
    }

    fn move_pointer_on_branch(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

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

    fn compare_to_memory(&mut self, value: u8, mode: &AddressingMode) {
        let memory_value = self.get_operand_address_value(mode);

        let inverse_memory_value = (!memory_value as u16).wrapping_add(1);

        let result = inverse_memory_value.wrapping_add(value as u16);

        let [lo, hi] = u16::to_le_bytes(result);

        self.status.set_zero_flag(lo);
        self.status.set_negative_flag(lo);
        self.status.set_flag(Flag::Carry, hi > 0);
    }

    fn set_decrement_flags(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(0b1111_1111);

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);

        return result;
    }

    fn set_increment_flags(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);

        return result;
    }

    fn check_boundary_crossed(&mut self, address: u16, value: u8) -> bool {
        let updated_address = address.wrapping_add(value as u16);

        let [_start_address_lo, start_address_hi] = u16::to_le_bytes(address);
        let [_updated_address_lo, updated_address_hi] = u16::to_le_bytes(updated_address);

        let crossed_page = updated_address_hi != start_address_hi;
        crossed_page
    }

    fn major_cycles(&mut self, mode: &AddressingMode) -> u8 {
        match mode {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => {
                let address = self.memory.mem_read_u16(self.program_counter);

                let crossed_page = self.check_boundary_crossed(address, self.register_x);

                if crossed_page {
                    5
                } else {
                    4
                }
            }
            AddressingMode::AbsoluteY => {
                let address = self.memory.mem_read_u16(self.program_counter);

                let crossed_page = self.check_boundary_crossed(address, self.register_y);

                if crossed_page {
                    5
                } else {
                    4
                }
            }
            AddressingMode::IndirectX => 6,
            AddressingMode::IndirectY => {
                let address = self.memory.mem_read(self.program_counter);

                let address = self.memory.mem_read_u16(address as u16);

                let crossed_page = self.check_boundary_crossed(address, self.register_y);

                if crossed_page {
                    6
                } else {
                    5
                }
            }
            _ => {
                panic!("Trying to calculate cycles of an unsupported mode.")
            }
        }
    }

    /// Add a value and the Accumulator together with a Carry.
    ///
    /// Setting overflow is the hardest thing here, but essentially the 8th bit encodes if a number
    /// is positive or negative. So we are comparing the result and saying if both values are
    /// positive but the result is negative then we have overflow, and vice versa if both are
    /// negative but the result is positive then we have overflow.
    fn adc(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        self.addition_with_register_a(value as u16);

        self.apply_bytes_to_program_counter(bytes);
    }

    /// Bitwise AND with a value and the Accumulator
    fn and(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let result = self.register_a & value;

        self.register_a = result;

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);

        self.apply_bytes_to_program_counter(bytes);
    }

    /// Shift left one bit, either the accumulator or a value
    fn asl(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let result = (value as u16) << 1;

        println!("result: {}", result);

        let [lo, hi] = u16::to_le_bytes(result);

        self.register_a = lo;

        self.status.set_zero_flag(lo); // TODO need to check if this is correct and not based on result (rather than lo)
        self.status.set_negative_flag(lo);
        self.status.set_flag(Flag::Carry, hi > 0);

        self.apply_bytes_to_program_counter(bytes);
    }

    /// Branch when the Carry flag = 0 (Carry clear)
    fn bcc(&mut self, mode: &AddressingMode, bytes: u8) {
        let carry = self.status.read_flag(Flag::Carry);

        if carry {
            self.apply_bytes_to_program_counter(bytes);
            return;
        }

        self.move_pointer_on_branch(mode, bytes);
    }

    /// Branch when the Carry flag = 1 (Carry set)
    fn bcs(&mut self, mode: &AddressingMode, bytes: u8) {
        let carry = self.status.read_flag(Flag::Carry);

        if !carry {
            self.apply_bytes_to_program_counter(bytes);
            return;
        }

        self.move_pointer_on_branch(mode, bytes);
    }

    /// Branch when the Zero flag = 1 (Result Zero)
    fn beq(&mut self, mode: &AddressingMode, bytes: u8) {
        let zero = self.status.read_flag(Flag::Zero);

        if !zero {
            self.apply_bytes_to_program_counter(bytes);
            return;
        }

        self.move_pointer_on_branch(mode, bytes);
    }

    /// Test Bits in Memory with Accumulator
    ///
    /// A confusing one, it sets the Negative and Overflow flag to bits 7 and 6 of the given value.
    /// It also sets the zero flag if the result of the value AND accumulator is 0 or not.
    fn bit(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let and_result = self.register_a & value;

        self.status
            .set_flag(Flag::Negative, (value & 0b1000_0000) > 0);
        self.status
            .set_flag(Flag::Overflow, (value & 0b0100_0000) > 0);
        self.status.set_flag(Flag::Zero, and_result == 0);

        self.apply_bytes_to_program_counter(bytes);
    }

    /// Branch when the Negative flag = 1 (Result Minus)
    fn bmi(&mut self, mode: &AddressingMode, bytes: u8) {
        let negative = self.status.read_flag(Flag::Negative);

        if !negative {
            self.apply_bytes_to_program_counter(bytes);
            return;
        }

        self.move_pointer_on_branch(mode, bytes);
    }

    /// Branch when the Zero flag = 0 (Result not Zero)
    fn bne(&mut self, mode: &AddressingMode, bytes: u8) {
        let zero = self.status.read_flag(Flag::Zero);

        if zero {
            self.apply_bytes_to_program_counter(bytes);
            return;
        }

        self.move_pointer_on_branch(mode, bytes);
    }

    /// Branch when the Negative flag = 0 (Result Plus)
    fn bpl(&mut self, mode: &AddressingMode, bytes: u8) {
        let negative = self.status.read_flag(Flag::Negative);

        if negative {
            self.apply_bytes_to_program_counter(bytes);
            return;
        }

        self.move_pointer_on_branch(mode, bytes);
    }

    /// Force break
    fn brk(&mut self) {
        self.push_to_stack_u16(self.program_counter + 2);

        let break_flag = self.status.read_flag(Flag::Break);

        self.status.set_flag(Flag::Break, true);
        self.push_to_stack(self.status.get_status_byte());

        self.status.set_flag(Flag::Break, break_flag);

        self.program_counter = self.memory.mem_read_u16(0xfffe)
    }

    /// Branch when the Overflow flag = 0 (Overflow Clear)
    fn bvc(&mut self, mode: &AddressingMode, bytes: u8) {
        let overflow = self.status.read_flag(Flag::Overflow);

        if overflow {
            self.apply_bytes_to_program_counter(bytes);
            return;
        }

        self.move_pointer_on_branch(mode, bytes);
    }

    /// Branch when the Overflow flag = 1 (Overflow Set)
    fn bvs(&mut self, mode: &AddressingMode, bytes: u8) {
        let overflow = self.status.read_flag(Flag::Overflow);

        if !overflow {
            self.apply_bytes_to_program_counter(bytes);
            return;
        }

        self.move_pointer_on_branch(mode, bytes);
    }

    fn clc(&mut self) {
        self.status.set_flag(Flag::Carry, false);
    }

    fn cld(&mut self) {
        self.status.set_flag(Flag::Decimal, false);
    }

    fn cli(&mut self) {
        self.status.set_flag(Flag::Interrupt, false);
    }

    fn clv(&mut self) {
        self.status.set_flag(Flag::Overflow, false);
    }

    fn cmp(&mut self, mode: &AddressingMode, bytes: u8) {
        let accumulator = self.register_a;
        self.compare_to_memory(accumulator, mode);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn cpx(&mut self, mode: &AddressingMode, bytes: u8) {
        let accumulator = self.register_x;
        self.compare_to_memory(accumulator, mode);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn cpy(&mut self, mode: &AddressingMode, bytes: u8) {
        let accumulator = self.register_y;
        self.compare_to_memory(accumulator, mode);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn dec(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let result = self.set_decrement_flags(value);

        let address = self.get_operand_address(mode);

        self.memory.mem_write(address, result);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn dex(&mut self) {
        let value = self.register_x;

        let result = self.set_decrement_flags(value);

        self.register_x = result;
    }

    fn dey(&mut self) {
        let value = self.register_y;

        let result = self.set_decrement_flags(value);

        self.register_y = result;
    }

    fn eor(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let accumulator = self.register_a;

        let result = accumulator ^ value;

        self.register_a = result;

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn inc(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let result = self.set_increment_flags(value);

        let address = self.get_operand_address(mode);

        self.memory.mem_write(address, result);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn inx(&mut self) {
        let value = self.register_x;

        let result = self.set_increment_flags(value);

        self.register_x = result;
    }

    fn iny(&mut self) {
        let value = self.register_y;

        let result = self.set_increment_flags(value);

        self.register_y = result;
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        let address = self.get_operand_address(mode);

        self.program_counter = address;
    }

    fn jsr(&mut self, mode: &AddressingMode) {
        self.push_to_stack_u16(self.program_counter.wrapping_add(1));

        self.jmp(mode);
    }

    /// Load Accumulator
    fn lda(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        self.register_a = value;
        let result = self.register_a;
        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);

        self.apply_bytes_to_program_counter(bytes);
    }

    /// Load X register
    fn ldx(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        self.register_x = value;
        let result = self.register_x;
        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);

        self.apply_bytes_to_program_counter(bytes);
    }

    /// Load Y register
    fn ldy(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        self.register_y = value;
        let result = self.register_y;
        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn lsr(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let carry_flag = value & 0b0000_0001;
        let result = value >> 1;

        match mode {
            AddressingMode::Accumulator => {
                self.register_a = result;
            }
            _ => {
                let address = self.get_operand_address(mode);

                self.memory.mem_write(address, result);
            }
        }

        self.status.set_zero_flag(result);
        self.status.set_flag(Flag::Carry, carry_flag > 0);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn nop(&mut self) {}

    /// Bitwise OR with a value and the Accumulator
    fn ora(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let result = self.register_a | value;

        self.register_a = result;
        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn pha(&mut self) {
        self.push_to_stack(self.register_a)
    }

    fn php(&mut self) {
        let break_flag = self.status.read_flag(Flag::Break);
        let ignored_flag = self.status.read_flag(Flag::Ignored);

        self.status.set_flag(Flag::Break, true);
        self.status.set_flag(Flag::Ignored, true);
        let status = self.status.get_status_byte();

        self.push_to_stack(status);

        self.status.set_flag(Flag::Break, break_flag);
        self.status.set_flag(Flag::Ignored, ignored_flag);
    }

    fn pla(&mut self) {
        let result = self.pull_from_stack();

        self.register_a = result;

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
    }

    fn plp(&mut self) {
        let break_flag = self.status.read_flag(Flag::Break);
        let ignored_flag = self.status.read_flag(Flag::Ignored);

        let result = self.pull_from_stack();

        self.status.set_from_byte(result);

        self.status.set_flag(Flag::Break, break_flag);
        self.status.set_flag(Flag::Ignored, ignored_flag);
    }

    fn rol(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let carry_flag = value & 0b1000_0000;
        let result = (value << 1) | (self.status.read_flag(Flag::Carry) as u8);

        match mode {
            AddressingMode::Accumulator => {
                self.register_a = result;
            }
            _ => {
                let address = self.get_operand_address(mode);

                self.memory.mem_write(address, result);
            }
        }

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
        self.status.set_flag(Flag::Carry, carry_flag > 0);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn ror(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let carry_flag = value & 0b0000_0001;
        let result = (value >> 1) | ((self.status.read_flag(Flag::Carry) as u8) << 7);

        match mode {
            AddressingMode::Accumulator => {
                self.register_a = result;
            }
            _ => {
                let address = self.get_operand_address(mode);

                self.memory.mem_write(address, result);
            }
        }

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
        self.status.set_flag(Flag::Carry, carry_flag > 0);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn rti(&mut self) {
        self.plp();

        let program_counter = self.pull_from_stack_u16();

        self.program_counter = program_counter;
    }

    fn rts(&mut self) {
        let program_counter = self.pull_from_stack_u16().wrapping_add(1);

        self.program_counter = program_counter
    }

    fn sbc(&mut self, mode: &AddressingMode, bytes: u8) {
        let value = self.get_operand_address_value(mode);

        let value = !value;

        self.addition_with_register_a(value as u16);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn sec(&mut self) {
        self.status.set_flag(Flag::Carry, true);
    }

    fn sed(&mut self) {
        self.status.set_flag(Flag::Decimal, true);
    }

    fn sei(&mut self) {
        self.status.set_flag(Flag::Interrupt, true);
    }

    fn sta(&mut self, mode: &AddressingMode, bytes: u8) {
        let address = self.get_operand_address(mode);

        self.memory.mem_write(address, self.register_a);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn stx(&mut self, mode: &AddressingMode, bytes: u8) {
        let address = self.get_operand_address(mode);

        self.memory.mem_write(address, self.register_x);

        self.apply_bytes_to_program_counter(bytes);
    }

    fn sty(&mut self, mode: &AddressingMode, bytes: u8) {
        let address = self.get_operand_address(mode);

        self.memory.mem_write(address, self.register_y);

        self.apply_bytes_to_program_counter(bytes);
    }

    /// Transfer Accumulator to Index X
    fn tax(&mut self) {
        let result = self.register_a;

        self.register_x = result;

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
    }

    /// Transfer Accumulator to Index Y
    fn tay(&mut self) {
        let result = self.register_a;

        self.register_y = result;

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
    }

    /// Transfer Stack Pointer to Index X
    fn tsx(&mut self) {
        let result = self.stack_pointer;

        self.register_x = result;

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
    }

    /// Transfer Index X to Accumulator
    fn txa(&mut self) {
        let result = self.register_x;

        self.register_a = result;

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
    }

    /// Transfer Index X to Stack Pointer
    fn txs(&mut self) {
        let result = self.register_x;

        self.stack_pointer = result;

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
    }

    /// Transfer Index Y to Accumulator
    fn tya(&mut self) {
        let result = self.register_y;

        self.register_a = result;

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);
    }

    pub fn run_opcode(&mut self, opcode: &OpCode) {
        let OpCodeDetail {
            instruction,
            bytes,
            address_mode: mode,
            ..
        } = opcodes::get_opcode_detail(&opcode);

        let bytes = bytes - 1;

        match instruction {
            Instruction::ADC => {
                self.adc(&mode, bytes);
            }
            Instruction::AND => {
                self.and(&mode, bytes);
            }
            Instruction::ASL => {
                self.asl(&mode, bytes);
            }
            Instruction::BCC => {
                self.bcc(&mode, bytes);
            }
            Instruction::BCS => {
                self.bcs(&mode, bytes);
            }
            Instruction::BEQ => {
                self.beq(&mode, bytes);
            }
            Instruction::BIT => {
                self.bit(&mode, bytes);
            }
            Instruction::BMI => {
                self.bmi(&mode, bytes);
            }
            Instruction::BNE => {
                self.bne(&mode, bytes);
            }
            Instruction::BPL => {
                self.bpl(&mode, bytes);
            }
            Instruction::BRK => {
                self.brk();
            }
            Instruction::BVC => {
                self.bvc(&mode, bytes);
            }
            Instruction::BVS => {
                self.bvs(&mode, bytes);
            }
            Instruction::CLC => {
                self.clc();
            }
            Instruction::CLD => {
                self.cld();
            }
            Instruction::CLI => {
                self.cli();
            }
            Instruction::CLV => {
                self.clv();
            }
            Instruction::CMP => {
                self.cmp(&mode, bytes);
            }
            Instruction::CPX => {
                self.cpx(&mode, bytes);
            }
            Instruction::CPY => {
                self.cpy(&mode, bytes);
            }
            Instruction::DEC => {
                self.dec(&mode, bytes);
            }
            Instruction::DEX => {
                self.dex();
            }
            Instruction::DEY => {
                self.dey();
            }
            Instruction::EOR => {
                self.eor(&mode, bytes);
            }
            Instruction::INC => {
                self.inc(&mode, bytes);
            }
            Instruction::INX => {
                self.inx();
            }
            Instruction::INY => {
                self.iny();
            }
            Instruction::JMP => {
                self.jmp(&mode);
            }
            Instruction::JSR => {
                self.jsr(&mode);
            }
            Instruction::LDA => {
                self.lda(&mode, bytes);
            }
            Instruction::LDX => {
                self.ldx(&mode, bytes);
            }
            Instruction::LDY => {
                self.ldy(&mode, bytes);
            }
            Instruction::LSR => {
                self.lsr(&mode, bytes);
            }
            Instruction::NOP => {
                self.nop();
            }
            Instruction::ORA => {
                self.ora(&mode, bytes);
            }
            Instruction::PHA => {
                self.pha();
            }
            Instruction::PHP => {
                self.php();
            }
            Instruction::PLA => {
                self.pla();
            }
            Instruction::PLP => {
                self.plp();
            }
            Instruction::ROL => {
                self.rol(&mode, bytes);
            }
            Instruction::ROR => {
                self.ror(&mode, bytes);
            }
            Instruction::RTI => {
                self.rti();
            }
            Instruction::RTS => {
                self.rts();
            }
            Instruction::SBC => {
                self.sbc(&mode, bytes);
            }
            Instruction::SEC => {
                self.sec();
            }
            Instruction::SED => {
                self.sed();
            }
            Instruction::SEI => {
                self.sei();
            }
            Instruction::STA => {
                self.sta(&mode, bytes);
            }
            Instruction::STX => {
                self.stx(&mode, bytes);
            }
            Instruction::STY => {
                self.sty(&mode, bytes);
            }
            Instruction::TAX => {
                self.tax();
            }
            Instruction::TAY => {
                self.tay();
            }
            Instruction::TSX => {
                self.tsx();
            }
            Instruction::TXA => {
                self.txa();
            }
            Instruction::TXS => {
                self.txs();
            }
            Instruction::TYA => {
                self.tya();
            }
        };
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            // println!("Iteration:");
            // println!("register_a: {:#04x}", self.register_a);
            // println!("register_x: {:#04x}", self.register_x);
            // println!("register_y: {:#04x}", self.register_y);
            // println!("stack_pointer: {:#04x}", self.stack_pointer);
            // println!("program_counter: {:#06x}", self.program_counter);
            // println!("status_byte: {:b}", self.status.get_status_byte());
            // println!("\nZero page:");
            // self.memory.print_page(0x00);
            // println!("\nSnake page:");
            // self.memory.print_page(0x02);
            // println!();

            callback(self);

            let code = self.memory.mem_read(self.program_counter);
            self.program_counter += 1;

            let opcode = opcodes::get_opcode(&code);

            self.run_opcode(&opcode)
        }
    }
}

fn make_test_cartridge() -> Cartridge {
    let mut contents: Vec<u8> = vec![
        0x4e,
        0x45,
        0x53,
        0x1a,
        0x02,
        0x02,
        0b0001_0001,
        0b0000_0000,
        0x00,
        0x00,
    ];

    contents.extend([0; 6]);
    contents.extend([0x01; PRG_ROM_PAGE_SIZE * 2]);
    contents.extend([0x02; CHR_ROM_PAGE_SIZE * 2]);

    let cartridge = Cartridge::new(&contents);

    return cartridge;
}

#[cfg(test)]
mod test_stack {
    use super::*;

    #[test]
    fn test_stack_address() {
        let cpu = CPU::new(make_test_cartridge());

        let address = cpu.get_stack_address();

        assert_eq!(address, 0x01ff);
    }

    #[test]
    fn test_push_to_stack() {
        let mut cpu = CPU::new(make_test_cartridge());

        cpu.push_to_stack(0x12);

        let stack_pointer = cpu.stack_pointer;

        assert_eq!(stack_pointer, 0xfe);
        assert_eq!(cpu.memory.mem_read(0x01ff), 0x12);
    }

    #[test]
    fn test_push_to_stack_u16() {
        let mut cpu = CPU::new(make_test_cartridge());

        cpu.push_to_stack_u16(0x1234);

        let stack_pointer = cpu.stack_pointer;

        assert_eq!(stack_pointer, 0xfd);
        assert_eq!(cpu.memory.mem_read(0x01ff), 0x12);
        assert_eq!(cpu.memory.mem_read(0x01fe), 0x34);
    }

    #[test]
    fn test_pull_from_stack() {
        let mut cpu = CPU::new(make_test_cartridge());

        cpu.push_to_stack(0x12);
        let data = cpu.pull_from_stack();

        let stack_pointer = cpu.stack_pointer;

        assert_eq!(stack_pointer, 0xff);
        assert_eq!(data, 0x12);
    }

    #[test]
    fn test_pull_from_stack_u16() {
        let mut cpu = CPU::new(make_test_cartridge());

        cpu.push_to_stack_u16(0x1234);
        let data = cpu.pull_from_stack_u16();

        let stack_pointer = cpu.stack_pointer;

        assert_eq!(stack_pointer, 0xff);
        assert_eq!(data, 0x1234);
    }
}

#[cfg(test)]
mod test_address_modes {
    use super::*;

    #[test]
    fn test_immediate() {
        let mut cpu = CPU::new(make_test_cartridge());

        let address = cpu.get_operand_address(&AddressingMode::Immediate);

        assert_eq!(address, 0x0000)
    }

    #[test]
    fn test_zero_page() {
        // The program counter is pointing to 0x00 so we set this to 0x12 and check it works
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x00, 0x12);

        let address = cpu.get_operand_address(&AddressingMode::ZeroPage);

        assert_eq!(address, 0x0012)
    }

    #[test]
    fn test_zero_page_x() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_x = 0x01;
        cpu.memory.mem_write(0x00, 0x12);

        let address = cpu.get_operand_address(&AddressingMode::ZeroPageX);

        assert_eq!(address, 0x0013)
    }

    #[test]
    fn test_zero_page_y() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_y = 0x01;
        cpu.memory.mem_write(0x00, 0x12);

        let address = cpu.get_operand_address(&AddressingMode::ZeroPageY);

        assert_eq!(address, 0x0013)
    }

    #[test]
    fn test_absolute() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write_u16(0x00, 0x1234);

        let address = cpu.get_operand_address(&AddressingMode::Absolute);

        assert_eq!(address, 0x1234)
    }

    #[test]
    fn test_absolute_x() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_x = 0x01;
        cpu.memory.mem_write_u16(0x00, 0x1234);

        let address = cpu.get_operand_address(&AddressingMode::AbsoluteX);

        assert_eq!(address, 0x1235)
    }

    #[test]
    fn test_absolute_y() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_y = 0x01;
        cpu.memory.mem_write_u16(0x00, 0x1234);

        let address = cpu.get_operand_address(&AddressingMode::AbsoluteY);

        assert_eq!(address, 0x1235)
    }

    #[test]
    fn test_indirect() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write_u16(0x00, 0x1234);
        cpu.memory.mem_write_u16(0x1234, 0x5678);

        let address = cpu.get_operand_address(&AddressingMode::Indirect);

        assert_eq!(address, 0x5678)
    }

    #[test]
    fn test_indirect_x() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_x = 0x01;
        cpu.memory.mem_write(0x00, 0x12);
        cpu.memory.mem_write_u16(0x13, 0x3456);

        let address = cpu.get_operand_address(&AddressingMode::IndirectX);

        assert_eq!(address, 0x3456)
    }

    #[test]
    fn test_indirect_y() {
        let mut cpu = CPU::new(make_test_cartridge());
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

    #[test]
    fn test_adc() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0x12;
        cpu.memory.mem_write(0x0000, 0x34);

        cpu.adc(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0x46);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), false);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), false);
    }

    #[test]
    fn test_adc_zero() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0x00;
        cpu.memory.mem_write(0x0000, 0x00);

        cpu.adc(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.status.read_flag(Flag::Zero), true);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), false);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), false);
    }

    #[test]
    fn test_adc_negative() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0b1000_1010;
        cpu.memory.mem_write(0x0000, 0b0000_0001);

        cpu.adc(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0b1000_1011);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), true);
        assert_eq!(cpu.status.read_flag(Flag::Carry), false);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), false);
    }

    #[test]
    fn test_adc_carry() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0b1100_1010;
        cpu.memory.mem_write(0x0000, 0b0100_0001);

        cpu.adc(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0b0000_1011);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), false);
    }

    #[test]
    fn test_adc_overflow() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0b1000_1010;
        cpu.memory.mem_write(0x0000, 0b1000_0001);

        cpu.adc(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0b0000_1011);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), true);
    }

    #[test]
    fn test_and() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0b1000_1010;
        cpu.memory.mem_write(0x0000, 0b0000_0010);

        cpu.and(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0b0000_0010);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x0000, 0b0000_0001);

        cpu.asl(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0b0000_0010);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), false);
    }

    #[test]
    fn test_asl_accumulator() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0b0000_0001;

        cpu.asl(&AddressingMode::Accumulator, 0);

        assert_eq!(cpu.register_a, 0b0000_0010);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), false);
    }

    #[test]
    fn test_bcc() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Carry, false);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bcc(&AddressingMode::Relative, 0);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_bcs() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Carry, true);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bcs(&AddressingMode::Relative, 0);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_beq() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Zero, true);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.beq(&AddressingMode::Relative, 0);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_bit() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.program_counter = 0x0000;
        cpu.register_a = 0b0100_0000;
        cpu.memory.mem_write(0x0000, 0x01);
        cpu.memory.mem_write(0x0001, 0b1100_0000);

        cpu.bit(&AddressingMode::ZeroPage, 0);

        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), true);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), true);
    }

    #[test]
    fn test_bmi() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Negative, true);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bmi(&AddressingMode::Relative, 0);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_bne() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Zero, false);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bne(&AddressingMode::Relative, 0);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_bpl() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Negative, false);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bpl(&AddressingMode::Relative, 0);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_brk() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write_u16(0xfffe, 0x0012);

        cpu.brk();

        let status = cpu.pull_from_stack();
        let program_counter = cpu.pull_from_stack_u16();

        assert_eq!(status, 0b0001_0000);
        assert_eq!(program_counter, 0x0002);
        assert_eq!(cpu.program_counter, 0x0012);
    }

    #[test]
    fn test_bvc() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Overflow, false);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bvc(&AddressingMode::Relative, 0);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_bvs() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Overflow, true);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bvs(&AddressingMode::Relative, 0);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_clc() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.status.set_flag(Flag::Carry, true);
        cpu.clc();

        let carry_flag = cpu.status.read_flag(Flag::Carry);

        assert_eq!(carry_flag, false);
    }

    #[test]
    fn test_cld() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.status.set_flag(Flag::Decimal, true);
        cpu.cld();

        let decimal_flag = cpu.status.read_flag(Flag::Decimal);

        assert_eq!(decimal_flag, false);
    }

    #[test]
    fn test_cli() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.status.set_flag(Flag::Interrupt, true);
        cpu.cli();

        let interrupt_flag = cpu.status.read_flag(Flag::Interrupt);

        assert_eq!(interrupt_flag, false);
    }

    #[test]
    fn test_clv() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.status.set_flag(Flag::Overflow, true);
        cpu.clv();

        let overflow_flag = cpu.status.read_flag(Flag::Overflow);

        assert_eq!(overflow_flag, false);
    }

    #[test]
    fn test_cmp_negative() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x0000, 0b0100_0000);
        cpu.register_a = 0b1100_0000;

        cpu.cmp(&AddressingMode::Immediate, 0);

        let negative_flag = cpu.status.read_flag(Flag::Negative);

        assert_eq!(negative_flag, true);
    }

    #[test]
    fn test_cmp_zero() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x0000, 0b0100_0000);
        cpu.register_a = 0b0100_0000;

        cpu.cmp(&AddressingMode::Immediate, 0);

        let zero_flag = cpu.status.read_flag(Flag::Zero);

        assert_eq!(zero_flag, true);
    }

    #[test]
    fn test_cmp_carry() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x0000, 0b1000_0000);
        cpu.register_a = 0b1000_0000;

        cpu.cmp(&AddressingMode::Immediate, 0);

        let zero_flag = cpu.status.read_flag(Flag::Zero);
        let carry_flag = cpu.status.read_flag(Flag::Carry);

        assert_eq!(zero_flag, true);
        assert_eq!(carry_flag, true);
    }

    #[test]
    fn test_dec() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x0000, 0x12);

        cpu.dec(&AddressingMode::Immediate, 0);

        let result = cpu.memory.mem_read(0x0000);

        assert_eq!(result, 0x11);
    }

    #[test]
    fn test_dex() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_x = 0x12;

        cpu.dex();

        let result = cpu.register_x;

        assert_eq!(result, 0x11);
    }

    #[test]
    fn test_dey() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_y = 0x12;

        cpu.dey();

        let result = cpu.register_y;

        assert_eq!(result, 0x11);
    }

    #[test]
    fn test_inc() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x0000, 0x11);

        cpu.inc(&AddressingMode::Immediate, 0);

        let result = cpu.memory.mem_read(0x0000);

        assert_eq!(result, 0x12);
    }

    #[test]
    fn test_inx() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_x = 0x11;

        cpu.inx();

        let result = cpu.register_x;

        assert_eq!(result, 0x12);
    }

    #[test]
    fn test_iny() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_y = 0x11;

        cpu.iny();

        let result = cpu.register_y;

        assert_eq!(result, 0x12);
    }

    #[test]
    fn test_eor() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x0000, 0b1010_1010);
        cpu.register_a = 0b1111_0000;

        cpu.eor(&AddressingMode::Immediate, 0);

        let result = cpu.register_a;

        assert_eq!(result, 0b0101_1010);
    }

    #[test]
    fn test_jmp() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write_u16(0x0000, 0x0200);

        cpu.jmp(&AddressingMode::Absolute);

        assert_eq!(cpu.program_counter, 0x0200);
    }

    #[test]
    fn test_jsr() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write_u16(0x0000, 0x0200);

        cpu.jsr(&AddressingMode::Absolute);

        let jump_program_counter = cpu.pull_from_stack_u16();

        assert_eq!(cpu.program_counter, 0x0200);
        assert_eq!(jump_program_counter, 0x0002);
    }

    #[test]
    fn test_lda() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x0000, 0x12);

        cpu.lda(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0x12);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_ldx() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x0000, 0x12);

        cpu.ldx(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_x, 0x12);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_ldy() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write(0x0000, 0x12);

        cpu.ldy(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_y, 0x12);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_lsr() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0b0000_1111;

        cpu.lsr(&AddressingMode::Accumulator, 0);

        assert_eq!(cpu.register_a, 0b0000_0111);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
    }

    #[test]
    fn test_ora() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0b0000_0001;
        cpu.memory.mem_write(0x0000, 0b0000_0010);

        cpu.ora(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0b0000_0011);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_pha() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0x12;

        cpu.pha();

        let stack_value = cpu.pull_from_stack();

        assert_eq!(stack_value, 0x12);
    }

    #[test]
    fn test_php() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.status.set_flag(Flag::Zero, true);

        let status = cpu.status.get_status_byte();

        cpu.php();

        let stack_value = cpu.pull_from_stack();

        assert_eq!(cpu.status.get_status_byte(), status);
        assert_eq!(stack_value, 0b0011_0010)
    }

    #[test]
    fn test_pla() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.push_to_stack(0x12);

        cpu.pla();

        assert_eq!(cpu.register_a, 0x12);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
    }

    #[test]
    fn test_plp() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.push_to_stack(0b0011_0010);

        cpu.plp();

        assert_eq!(cpu.status.get_status_byte(), 0b0000_0010);
    }

    #[test]
    fn test_rol() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0b1000_1110;
        cpu.status.set_flag(Flag::Carry, true);

        cpu.rol(&AddressingMode::Accumulator, 0);

        assert_eq!(cpu.register_a, 0b0001_1101);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
    }

    #[test]
    fn test_ror() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0b0111_0001;
        cpu.status.set_flag(Flag::Carry, true);

        cpu.ror(&AddressingMode::Accumulator, 0);

        assert_eq!(cpu.register_a, 0b1011_1000);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
    }

    #[test]
    fn test_rti() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.push_to_stack_u16(0x1234);
        cpu.push_to_stack(0b0000_0011);

        cpu.rti();

        assert_eq!(cpu.status.get_status_byte(), 0b0000_0011);
        assert_eq!(cpu.program_counter, 0x1234);
    }

    #[test]
    fn test_rts() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.push_to_stack_u16(0x1234);

        cpu.rts();

        assert_eq!(cpu.program_counter, 0x1235);
    }

    #[test]
    fn test_sbc() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0x12;
        cpu.memory.mem_write(0x0000, 0x08);
        cpu.status.set_flag(Flag::Carry, true);

        cpu.sbc(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0x0a);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
        assert_eq!(cpu.status.read_flag(Flag::Overflow), false);
    }

    #[test]
    fn test_sec() {
        let mut cpu = CPU::new(make_test_cartridge());

        cpu.sec();

        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
    }

    #[test]
    fn test_sed() {
        let mut cpu = CPU::new(make_test_cartridge());

        cpu.sed();

        assert_eq!(cpu.status.read_flag(Flag::Decimal), true);
    }

    #[test]
    fn test_sei() {
        let mut cpu = CPU::new(make_test_cartridge());

        cpu.sei();

        assert_eq!(cpu.status.read_flag(Flag::Interrupt), true);
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write_u16(0x0000, 0x1234);
        cpu.register_a = 0x01;

        cpu.sta(&AddressingMode::Absolute, 0);

        assert_eq!(cpu.memory.mem_read(0x1234), 0x01);
    }

    #[test]
    fn test_stx() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write_u16(0x0000, 0x1234);
        cpu.register_x = 0x01;

        cpu.stx(&AddressingMode::Absolute, 0);

        assert_eq!(cpu.memory.mem_read(0x1234), 0x01);
    }

    #[test]
    fn test_sty() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.memory.mem_write_u16(0x0000, 0x1234);
        cpu.register_y = 0x01;

        cpu.sty(&AddressingMode::Absolute, 0);

        assert_eq!(cpu.memory.mem_read(0x1234), 0x01);
    }

    #[test]
    fn test_tax() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0x01;

        cpu.tax();

        assert_eq!(cpu.register_x, 0x01);
    }

    #[test]
    fn test_tay() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_a = 0x01;

        cpu.tay();

        assert_eq!(cpu.register_y, 0x01);
    }

    #[test]
    fn test_tsx() {
        let mut cpu = CPU::new(make_test_cartridge());

        cpu.tsx();

        assert_eq!(cpu.register_x, 0xff);
    }

    #[test]
    fn test_txa() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_x = 0x12;

        cpu.txa();

        assert_eq!(cpu.register_a, 0x12);
    }

    #[test]
    fn test_txs() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_x = 0x12;

        cpu.txs();

        assert_eq!(cpu.stack_pointer, 0x12);
    }

    #[test]
    fn test_tya() {
        let mut cpu = CPU::new(make_test_cartridge());
        cpu.register_y = 0x12;

        cpu.tya();

        assert_eq!(cpu.register_a, 0x12);
    }
}

#[cfg(test)]
mod test_run {
    #[test]
    fn test_snake() {
        // let mut cpu = CPU::new();
        //
        // let game_code = vec![
        //     0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9, 0x02, 0x85,
        //     0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9, 0x0f, 0x85,
        //     0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60, 0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe,
        //     0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3,
        //     0x06, 0x20, 0x19, 0x07, 0x20, 0x20, 0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9,
        //     0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60,
        //     0xa9, 0x04, 0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
        //     0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02,
        //     0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60, 0x60, 0x20, 0x94, 0x06,
        //     0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07,
        //     0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60, 0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06,
        //     0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c,
        //     0x35, 0x07, 0x60, 0xa6, 0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02,
        //     0x4a, 0xb0, 0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
        //     0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
        //     0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69, 0x20, 0x85, 0x10, 0xb0,
        //     0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29,
        //     0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35, 0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60,
        //     0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa2, 0x00, 0xea,
        //     0xea, 0xca, 0xd0, 0xfb, 0x60,
        // ];
        //
        // cpu.load(game_code);
        //
        // cpu.reset();
        //
        // cpu.run_with_callback(move |cpu| {});
    }
}
