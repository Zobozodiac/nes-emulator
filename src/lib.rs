use crate::status::Flag;
use opcodes::{AddressingMode, OpCode};
use std::ops::Add;

// TODO the program counter will be implemented incorrectly when using brk and the jmp commands because it always will increase by 1 afterwards but it should ignore it. Need to find best place to define.

pub mod memory;

pub mod opcodes;

pub mod status;

pub struct CPU {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: status::Status,
    program_counter: u16,
    stack_pointer: u8,
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
            stack_pointer: 0xff,
            memory: memory::Memory::new(),
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
        let stack_address  = self.get_stack_address();

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
        let stack_address  = self.get_stack_address();

        return self.memory.mem_read(stack_address)
    }

    fn pull_from_stack_u16(&mut self) -> u16 {
        let lo = self.pull_from_stack();
        let hi = self.pull_from_stack();

        return u16::from_le_bytes([lo, hi])
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

    fn move_pointer_on_branch(&mut self, mode: &AddressingMode) {
        let value = self.get_operand_address_value(mode);

        // Signed value to know which direction the relative change is
        let signed_value = value as i8;

        // We now convert the signed value into an unsigned u16. We needed to do this rather than
        // converting straight to u16 because for instance -1 would be 1111_1111 in u8, but
        // 1111_1111_1111_1111 in u16.
        let unsigned_u16: u16 = signed_value as u16;

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

        return result
    }

    fn set_increment_flags(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        self.status.set_zero_flag(result);
        self.status.set_negative_flag(result);

        return result
    }

    fn check_boundary_crossed(&mut self, address: u16, value: u8) -> bool {
        let updated_address = address.wrapping_add(value as u16);

        let [start_address_lo, start_address_hi] = u16::to_le_bytes(address);
        let [updated_address_lo, updated_address_hi] = u16::to_le_bytes(updated_address);

        let crossed_page = updated_address_hi != start_address_hi;
        crossed_page
    }

    fn major_cycles(&mut self, mode: &AddressingMode) -> u8 {
        match mode {
            AddressingMode::Immediate => {
                2
            }
            AddressingMode::ZeroPage => {
                3
            }
            AddressingMode::ZeroPageX => {
                4
            }
            AddressingMode::Absolute => {
                4
            }
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
            AddressingMode::IndirectX => {
                6
            }
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
    fn bcc(&mut self, mode: &AddressingMode) {
        let carry = self.status.read_flag(Flag::Carry);

        if carry {
            return;
        }

        self.move_pointer_on_branch(mode);
    }

    /// Branch when the Carry flag = 1 (Carry set)
    fn bcs(&mut self, mode: &AddressingMode) {
        let carry = self.status.read_flag(Flag::Carry);

        if !carry {
            return;
        }

        self.move_pointer_on_branch(mode);
    }

    /// Branch when the Zero flag = 1 (Result Zero)
    fn beq(&mut self, mode: &AddressingMode) {
        let zero = self.status.read_flag(Flag::Zero);

        if !zero {
            return;
        }

        self.move_pointer_on_branch(mode);
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
    fn bmi(&mut self, mode: &AddressingMode) {
        let negative = self.status.read_flag(Flag::Negative);

        if !negative {
            return;
        }

        self.move_pointer_on_branch(mode);
    }

    /// Branch when the Zero flag = 0 (Result not Zero)
    fn bne(&mut self, mode: &AddressingMode) {
        let zero = self.status.read_flag(Flag::Zero);

        if zero {
            return;
        }

        self.move_pointer_on_branch(mode);
    }

    /// Branch when the Negative flag = 0 (Result Plus)
    fn bpl(&mut self, mode: &AddressingMode) {
        let negative = self.status.read_flag(Flag::Negative);

        if negative {
            return;
        }

        self.move_pointer_on_branch(mode);
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
    fn bvc(&mut self, mode: &AddressingMode) {
        let overflow = self.status.read_flag(Flag::Overflow);

        if overflow {
            return;
        }

        self.move_pointer_on_branch(mode);
    }

    /// Branch when the Overflow flag = 1 (Overflow Set)
    fn bvs(&mut self, mode: &AddressingMode) {
        let overflow = self.status.read_flag(Flag::Overflow);

        if !overflow {
            return;
        }

        self.move_pointer_on_branch(mode);
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
        self.push_to_stack_u16(self.program_counter.wrapping_add(2));

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

    fn nop(&mut self) {

    }

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
                bytes,
                address_mode: mode,
                ..
            } = match opcode {
                Some(valid_opcode) => valid_opcode,
                None => panic!("OpCode not found in HashMap."),
            };

            let bytes = *bytes - 1;

            match *name {
                "ADC" => {
                    self.adc(mode, bytes);
                }
                "AND" => {
                    self.and(mode, bytes);
                }
                "ASL" => {
                    self.asl(mode, bytes);
                }
                "BCC" => {
                    self.bcc(mode);
                }
                "BCS" => {
                    self.bcs(mode);
                }
                "BEQ" => {
                    self.beq(mode);
                }
                "BIT" => {
                    self.bit(mode, bytes);
                }
                "BMI" => {
                    self.bmi(mode);
                }
                "BNE" => {
                    self.bne(mode);
                }
                "BPL" => {
                    self.bpl(mode);
                }
                "BRK" => {
                    return;
                }
                "BVC" => {
                    self.bvc(mode);
                }
                "BVS" => {
                    self.bvs(mode);
                }
                "CLC" => {
                    self.clc();
                }
                "CLD" => {
                    self.cld();
                }
                "CLI" => {
                    self.cli();
                }
                "CLV" => {
                    self.clv();
                }
                "CMP" => {
                    self.cmp(mode, bytes);
                }
                "CPX" => {
                    self.cpx(mode, bytes);
                }
                "CPY" => {
                    self.cpy(mode, bytes);
                }
                "DEC" => {
                    self.dec(mode, bytes);
                }
                "DEX" => {
                    self.dex();
                }
                "DEY" => {
                    self.dey();
                }
                "EOR" => {
                    self.eor(mode, bytes);
                }
                "INC" => {
                    self.inc(mode, bytes);
                }
                "INX" => {
                    self.inx();
                }
                "INY" => {
                    self.iny();
                }
                "JMP" => {
                    self.jmp(mode);
                }
                "JSR" => {
                    self.jsr(mode);
                }
                "LDA" => {
                    self.lda(mode, bytes);
                }
                "LDX" => {
                    self.ldx(mode, bytes);
                }
                "LDY" => {
                    self.ldy(mode, bytes);
                }
                "LSR" => {
                    self.lsr(mode, bytes);
                }
                "NOP" => {
                    self.nop();
                }
                "ORA" => {
                    self.ora(mode, bytes);
                }
                "PHA" => {
                    self.pha();
                }
                "PHP" => {
                    self.php();
                }
                "PLA" => {
                    self.pla();
                }
                "PLP" => {
                    self.plp();
                }
                "ROL" => {
                    self.rol(mode, bytes);
                }
                "ROR" => {
                    self.ror(mode, bytes);
                }
                "RTI" => {
                    self.rti();
                }
                "RTS" => {
                    self.rts();
                }
                "SBC" => {
                    self.sbc(mode, bytes);
                }
                "SEC" => {
                    self.sec();
                }
                "SED" => {
                    self.sed();
                }
                "SEI" => {
                    self.sei();
                }
                "STA" => {
                    self.sta(mode, bytes);
                }
                "STX" => {
                    self.stx(mode, bytes);
                }
                "STY" => {
                    self.sty(mode, bytes);
                }
                "TAX" => {
                    self.tax();
                }
                "TAY" => {
                    self.tay();
                }
                "TSX" => {
                    self.tsx();
                }
                "TXA" => {
                    self.txa();
                }
                "TXS" => {
                    self.txs();
                }
                "TYA" => {
                    self.tya();
                }
                _ => {
                    panic!("Unknown opcode.")
                }
            }
        }
    }
}

#[cfg(test)]
mod test_stack {
    use super::*;

    #[test]
    fn test_stack_address() {
        let cpu = CPU::new();

        let address = cpu.get_stack_address();

        assert_eq!(address, 0x01ff);
    }

    #[test]
    fn test_push_to_stack() {
        let mut cpu = CPU::new();

        cpu.push_to_stack(0x12);

        let stack_pointer = cpu.stack_pointer;

        assert_eq!(stack_pointer, 0xfe);
        assert_eq!(cpu.memory.mem_read(0x01ff), 0x12);
    }

    #[test]
    fn test_push_to_stack_u16() {
        let mut cpu = CPU::new();

        cpu.push_to_stack_u16(0x1234);

        let stack_pointer = cpu.stack_pointer;

        assert_eq!(stack_pointer, 0xfd);
        assert_eq!(cpu.memory.mem_read(0x01ff), 0x12);
        assert_eq!(cpu.memory.mem_read(0x01fe), 0x34);
    }

    #[test]
    fn test_pull_from_stack() {
        let mut cpu = CPU::new();

        cpu.push_to_stack(0x12);
        let data = cpu.pull_from_stack();

        let stack_pointer = cpu.stack_pointer;

        assert_eq!(stack_pointer, 0xff);
        assert_eq!(data, 0x12);
    }

    #[test]
    fn test_pull_from_stack_u16() {
        let mut cpu = CPU::new();

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

    #[test]
    fn test_adc() {
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
        cpu.register_a = 0b1000_1010;
        cpu.memory.mem_write(0x0000, 0b0000_0010);

        cpu.and(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0b0000_0010);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x0000, 0b0000_0001);

        cpu.asl(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0b0000_0010);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), false);
    }

    #[test]
    fn test_asl_accumulator() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0000_0001;

        cpu.asl(&AddressingMode::Accumulator, 0);

        assert_eq!(cpu.register_a, 0b0000_0010);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Carry), false);
    }

    #[test]
    fn test_bcc() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Carry, false);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bcc(&AddressingMode::Relative);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_bcs() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Carry, true);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bcs(&AddressingMode::Relative);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_beq() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Zero, true);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.beq(&AddressingMode::Relative);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_bit() {
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Negative, true);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bmi(&AddressingMode::Relative);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_bne() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Zero, false);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bne(&AddressingMode::Relative);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_bpl() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Negative, false);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bpl(&AddressingMode::Relative);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_brk() {
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Overflow, false);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bvc(&AddressingMode::Relative);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_bvs() {
        let mut cpu = CPU::new();
        cpu.program_counter = 0x0002;
        cpu.status.set_flag(Flag::Overflow, true);
        cpu.memory.mem_write(0x0002, 0b1111_1110);

        cpu.bvs(&AddressingMode::Relative);

        assert_eq!(cpu.program_counter, 0b0000_0000);
    }

    #[test]
    fn test_clc() {
        let mut cpu = CPU::new();
        cpu.status.set_flag(Flag::Carry, true);
        cpu.clc();

        let carry_flag = cpu.status.read_flag(Flag::Carry);

        assert_eq!(carry_flag, false);
    }

    #[test]
    fn test_cld() {
        let mut cpu = CPU::new();
        cpu.status.set_flag(Flag::Decimal, true);
        cpu.cld();

        let decimal_flag = cpu.status.read_flag(Flag::Decimal);

        assert_eq!(decimal_flag, false);
    }

    #[test]
    fn test_cli() {
        let mut cpu = CPU::new();
        cpu.status.set_flag(Flag::Interrupt, true);
        cpu.cli();

        let interrupt_flag = cpu.status.read_flag(Flag::Interrupt);

        assert_eq!(interrupt_flag, false);
    }

    #[test]
    fn test_clv() {
        let mut cpu = CPU::new();
        cpu.status.set_flag(Flag::Overflow, true);
        cpu.clv();

        let overflow_flag = cpu.status.read_flag(Flag::Overflow);

        assert_eq!(overflow_flag, false);
    }

    #[test]
    fn test_cmp_negative() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x0000, 0b0100_0000);
        cpu.register_a = 0b1100_0000;

        cpu.cmp(&AddressingMode::Immediate, 0);

        let negative_flag = cpu.status.read_flag(Flag::Negative);

        assert_eq!(negative_flag, true);
    }

    #[test]
    fn test_cmp_zero() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x0000, 0b0100_0000);
        cpu.register_a = 0b0100_0000;

        cpu.cmp(&AddressingMode::Immediate, 0);

        let zero_flag = cpu.status.read_flag(Flag::Zero);

        assert_eq!(zero_flag, true);
    }

    #[test]
    fn test_cmp_carry() {
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x0000, 0x12);

        cpu.dec(&AddressingMode::Immediate, 0);

        let result = cpu.memory.mem_read(0x0000);

        assert_eq!(result, 0x11);
    }

    #[test]
    fn test_dex() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x12;

        cpu.dex();

        let result = cpu.register_x;

        assert_eq!(result, 0x11);
    }

    #[test]
    fn test_dey() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x12;

        cpu.dey();

        let result = cpu.register_y;

        assert_eq!(result, 0x11);
    }

    #[test]
    fn test_inc() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x0000, 0x11);

        cpu.inc(&AddressingMode::Immediate, 0);

        let result = cpu.memory.mem_read(0x0000);

        assert_eq!(result, 0x12);
    }

    #[test]
    fn test_inx() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x11;

        cpu.inx();

        let result = cpu.register_x;

        assert_eq!(result, 0x12);
    }

    #[test]
    fn test_iny() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x11;

        cpu.iny();

        let result = cpu.register_y;

        assert_eq!(result, 0x12);
    }

    #[test]
    fn test_eor() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x0000, 0b1010_1010);
        cpu.register_a = 0b1111_0000;

        cpu.eor(&AddressingMode::Immediate, 0);

        let result = cpu.register_a;

        assert_eq!(result, 0b0101_1010);
    }

    #[test]
    fn test_jmp() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write_u16(0x0000, 0x0200);

        cpu.jmp(&AddressingMode::Absolute);

        assert_eq!(cpu.program_counter, 0x0200);
    }

    #[test]
    fn test_jsr() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write_u16(0x0000, 0x0200);

        cpu.jsr(&AddressingMode::Absolute);

        let jump_program_counter = cpu.pull_from_stack_u16();

        assert_eq!(cpu.program_counter, 0x0200);
        assert_eq!(jump_program_counter, 0x0002);
    }

    #[test]
    fn test_lda() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x0000, 0x12);

        cpu.lda(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0x12);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_ldx() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x0000, 0x12);

        cpu.ldx(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_x, 0x12);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_ldy() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write(0x0000, 0x12);

        cpu.ldy(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_y, 0x12);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_lsr() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0000_1111;

        cpu.lsr(&AddressingMode::Accumulator, 0);

        assert_eq!(cpu.register_a, 0b0000_0111);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
    }

    #[test]
    fn test_ora() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0000_0001;
        cpu.memory.mem_write(0x0000, 0b0000_0010);

        cpu.ora(&AddressingMode::Immediate, 0);

        assert_eq!(cpu.register_a, 0b0000_0011);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
    }

    #[test]
    fn test_pha() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x12;

        cpu.pha();

        let stack_value = cpu.pull_from_stack();

        assert_eq!(stack_value, 0x12);
    }

    #[test]
    fn test_php() {
        let mut cpu = CPU::new();
        cpu.status.set_flag(Flag::Zero, true);

        let status = cpu.status.get_status_byte();

        cpu.php();

        let stack_value = cpu.pull_from_stack();

        assert_eq!(cpu.status.get_status_byte(), status);
        assert_eq!(stack_value, 0b0011_0010)
    }

    #[test]
    fn test_pla() {
        let mut cpu = CPU::new();
        cpu.push_to_stack(0x12);

        cpu.pla();

        assert_eq!(cpu.register_a, 0x12);
        assert_eq!(cpu.status.read_flag(Flag::Negative), false);
        assert_eq!(cpu.status.read_flag(Flag::Zero), false);
    }

    #[test]
    fn test_plp() {
        let mut cpu = CPU::new();
        cpu.push_to_stack(0b0011_0010);

        cpu.plp();

        assert_eq!(cpu.status.get_status_byte(), 0b0000_0010);
    }

    #[test]
    fn test_rol() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b1000_1110;
        cpu.status.set_flag(Flag::Carry, true);

        cpu.rol(&AddressingMode::Accumulator, 0);

        assert_eq!(cpu.register_a, 0b0001_1101);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
    }

    #[test]
    fn test_ror() {
        let mut cpu = CPU::new();
        cpu.register_a = 0b0111_0001;
        cpu.status.set_flag(Flag::Carry, true);

        cpu.ror(&AddressingMode::Accumulator, 0);

        assert_eq!(cpu.register_a, 0b1011_1000);
        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
    }

    #[test]
    fn test_rti() {
        let mut cpu = CPU::new();
        cpu.push_to_stack_u16(0x1234);
        cpu.push_to_stack(0b0000_0011);

        cpu.rti();

        assert_eq!(cpu.status.get_status_byte(), 0b0000_0011);
        assert_eq!(cpu.program_counter, 0x1234);
    }

    #[test]
    fn test_rts() {
        let mut cpu = CPU::new();
        cpu.push_to_stack_u16(0x1234);

        cpu.rts();

        assert_eq!(cpu.program_counter, 0x1235);
    }

    #[test]
    fn test_sbc() {
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();

        cpu.sec();

        assert_eq!(cpu.status.read_flag(Flag::Carry), true);
    }

    #[test]
    fn test_sed() {
        let mut cpu = CPU::new();

        cpu.sed();

        assert_eq!(cpu.status.read_flag(Flag::Decimal), true);
    }

    #[test]
    fn test_sei() {
        let mut cpu = CPU::new();

        cpu.sei();

        assert_eq!(cpu.status.read_flag(Flag::Interrupt), true);
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write_u16(0x0000, 0x1234);
        cpu.register_a = 0x01;

        cpu.sta(&AddressingMode::Absolute, 0);

        assert_eq!(cpu.memory.mem_read(0x1234), 0x01);
    }

    #[test]
    fn test_stx() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write_u16(0x0000, 0x1234);
        cpu.register_x = 0x01;

        cpu.stx(&AddressingMode::Absolute, 0);

        assert_eq!(cpu.memory.mem_read(0x1234), 0x01);
    }

    #[test]
    fn test_sty() {
        let mut cpu = CPU::new();
        cpu.memory.mem_write_u16(0x0000, 0x1234);
        cpu.register_y = 0x01;

        cpu.sty(&AddressingMode::Absolute, 0);

        assert_eq!(cpu.memory.mem_read(0x1234), 0x01);
    }


    #[test]
    fn test_tax() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;

        cpu.tax();

        assert_eq!(cpu.register_x, 0x01);
    }


    #[test]
    fn test_tay() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x01;

        cpu.tay();

        assert_eq!(cpu.register_y, 0x01);
    }


    #[test]
    fn test_tsx() {
        let mut cpu = CPU::new();

        cpu.tsx();

        assert_eq!(cpu.register_x, 0xff);
    }


    #[test]
    fn test_txa() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x12;

        cpu.txa();

        assert_eq!(cpu.register_a, 0x12);
    }


    #[test]
    fn test_txs() {
        let mut cpu = CPU::new();
        cpu.register_x = 0x12;

        cpu.txs();

        assert_eq!(cpu.stack_pointer, 0x12);
    }


    #[test]
    fn test_tya() {
        let mut cpu = CPU::new();
        cpu.register_y = 0x12;

        cpu.tya();

        assert_eq!(cpu.register_a, 0x12);
    }
}

#[cfg(test)]
mod test_run {
    use super::*;

    #[test]
    fn test_snake() {
        let mut cpu = CPU::new();

        let game_code = vec![
            0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9, 0x02, 0x85,
            0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9, 0x0f, 0x85,
            0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60, 0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe,
            0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3,
            0x06, 0x20, 0x19, 0x07, 0x20, 0x20, 0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9,
            0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60,
            0xa9, 0x04, 0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
            0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02,
            0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60, 0x60, 0x20, 0x94, 0x06,
            0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07,
            0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60, 0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06,
            0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c,
            0x35, 0x07, 0x60, 0xa6, 0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02,
            0x4a, 0xb0, 0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
            0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
            0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69, 0x20, 0x85, 0x10, 0xb0,
            0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29,
            0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35, 0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60,
            0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa2, 0x00, 0xea,
            0xea, 0xca, 0xd0, 0xfb, 0x60
        ];

        let start_address = 0x0600;
        cpu.memory.mem_write_u16(0xfffc, start_address);

        let mut address = start_address;

        for code in game_code {
            cpu.memory.mem_write(address, code);
            address += 1;
        }

        cpu.reset();
        cpu.run();
    }
}
