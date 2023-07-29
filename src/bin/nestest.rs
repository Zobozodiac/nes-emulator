use std::fs;

use nes_emulator::bus::Bus;
use nes_emulator::cpu::CPU;
use nes_emulator::memory::Mem;
use nes_emulator::opcodes::{AddressingMode, OpCode, OpCodeDetail};
use nes_emulator::{cartridge, cpu};

fn main() {
    let file_name = "nestest/nestest.nes";

    let raw = fs::read(file_name).expect("nestest.nes not found");

    let cartridge = cartridge::Cartridge::new(&raw);
    let bus = Bus::new(cartridge);

    let mut cpu = cpu::CPU::new(bus);
    cpu.reset();

    cpu.run_with_callback(|cpu| {
        trace(cpu);
    });
}

fn trace(cpu: &CPU) {
    let mut full_trace = String::new();

    let program_counter = program_counter_string(cpu);
    let cpu_opcode = cpu_opcode_string(cpu);
    let cpu_assembly = cpu_opcode_assembly_string(cpu);
    let registers = registers_string(cpu);

    full_trace.push_str(&program_counter);
    full_trace.push_str(&cpu_opcode);
    full_trace.push_str(&cpu_assembly);
    full_trace.push_str(&registers);

    println!("{}", full_trace)
}

fn pad_string(string: String, length: usize) -> String {
    let mut extended_str = string;
    while extended_str.len() < length {
        extended_str.push_str(" ")
    }

    extended_str
}

fn program_counter_string(cpu: &CPU) -> String {
    pad_string(format!("{:04X}", cpu.program_counter), 6)
}

fn cpu_opcode_string(cpu: &CPU) -> String {
    let mut opcode_string = "".to_string();

    let opcode = cpu.bus.mem_read(cpu.program_counter);
    opcode_string.push_str(&format!("{:02X}", opcode));

    let opcode = OpCode::from_code(&opcode);
    let opcode_detail = OpCodeDetail::from_opcode(&opcode);

    match opcode_detail.address_mode {
        AddressingMode::Absolute
        | AddressingMode::AbsoluteX
        | AddressingMode::AbsoluteY
        | AddressingMode::Indirect => opcode_string.push_str(&format!(
            " {:02X} {:02X}",
            cpu.bus.mem_read(cpu.program_counter + 1),
            cpu.bus.mem_read(cpu.program_counter + 2)
        )),
        AddressingMode::ZeroPage
        | AddressingMode::ZeroPageX
        | AddressingMode::ZeroPageY
        | AddressingMode::Relative
        | AddressingMode::IndirectX
        | AddressingMode::IndirectY
        | AddressingMode::Immediate => opcode_string.push_str(&format!(
            " {:02X}",
            cpu.bus.mem_read(cpu.program_counter + 1)
        )),
        AddressingMode::Implied | AddressingMode::Accumulator => {}
    };

    pad_string(opcode_string, 10)
}

fn cpu_opcode_assembly_string(cpu: &CPU) -> String {
    let mut opcode_string = "".to_string();

    let opcode = cpu.bus.mem_read(cpu.program_counter);
    let opcode = OpCode::from_code(&opcode);
    let opcode_detail = OpCodeDetail::from_opcode(&opcode);

    opcode_string.push_str(opcode_detail.instruction.to_string());

    match opcode_detail.address_mode {
        AddressingMode::Accumulator => opcode_string.push_str(" A"),
        AddressingMode::Absolute => {
            let value = cpu.get_operand_address_value(&opcode_detail.address_mode);

            opcode_string.push_str(&format!(
                " ${:04X} = {:02X}",
                cpu.bus.mem_read_u16(cpu.program_counter + 1),
                value
            ))
        }
        AddressingMode::AbsoluteX => {
            let address = cpu.get_operand_address(&opcode_detail.address_mode);
            let value = cpu.get_operand_address_value(&opcode_detail.address_mode);

            opcode_string.push_str(&format!(
                " ${:04X},X @ {:04X} = {:02X}",
                cpu.bus.mem_read_u16(cpu.program_counter + 1),
                address,
                value
            ))
        }
        AddressingMode::AbsoluteY => {
            let address = cpu.get_operand_address(&opcode_detail.address_mode);
            let value = cpu.get_operand_address_value(&opcode_detail.address_mode);

            opcode_string.push_str(&format!(
                " ${:04X},Y @ {:04X} = {:02X}",
                cpu.bus.mem_read_u16(cpu.program_counter + 1),
                address,
                value
            ))
        }
        AddressingMode::Immediate => opcode_string.push_str(&format!(
            " #${:02X}",
            cpu.bus.mem_read(cpu.program_counter + 1)
        )),
        AddressingMode::Implied => {}
        AddressingMode::Indirect => {
            let address = cpu.get_operand_address(&opcode_detail.address_mode);
            opcode_string.push_str(&format!(
                " (${:02X}) = {:04X}",
                cpu.bus.mem_read_u16(cpu.program_counter + 1),
                address
            ))
        }
        AddressingMode::IndirectX => {
            let address = cpu.get_operand_address(&opcode_detail.address_mode);
            let value = cpu.get_operand_address_value(&opcode_detail.address_mode);

            opcode_string.push_str(&format!(
                " (${:02X},X) @ {:02X} = {:04X} = {:02X}",
                cpu.bus.mem_read(cpu.program_counter + 1),
                cpu.bus.mem_read(cpu.program_counter + 1),
                address,
                value
            ))
        }
        AddressingMode::IndirectY => {
            let address = cpu.get_operand_address(&opcode_detail.address_mode);
            let value = cpu.get_operand_address_value(&opcode_detail.address_mode);

            opcode_string.push_str(&format!(
                " (${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                cpu.bus.mem_read(cpu.program_counter + 1),
                cpu.bus
                    .mem_read_u16(cpu.bus.mem_read(cpu.program_counter + 1) as u16),
                address,
                value
            ))
        }
        AddressingMode::Relative => {
            let address = cpu.get_operand_address(&opcode_detail.address_mode);
            opcode_string.push_str(&format!(" ${:02X}", address))
        }
        AddressingMode::ZeroPage => {
            let value = cpu.get_operand_address_value(&opcode_detail.address_mode);

            opcode_string.push_str(&format!(
                " ${:02X} = {:02X}",
                cpu.bus.mem_read(cpu.program_counter + 1),
                value
            ))
        }
        AddressingMode::ZeroPageX => {
            let value = cpu.get_operand_address_value(&opcode_detail.address_mode);

            opcode_string.push_str(&format!(
                " ${:02X},X @ {:02X} = {:02X}",
                cpu.bus.mem_read(cpu.program_counter + 1),
                cpu.bus
                    .mem_read(cpu.program_counter + 1)
                    .wrapping_add(cpu.register_x),
                value
            ))
        }
        AddressingMode::ZeroPageY => {
            let value = cpu.get_operand_address_value(&opcode_detail.address_mode);

            opcode_string.push_str(&format!(
                " ${:02X},Y @ {:02X} = {:02X}",
                cpu.bus.mem_read(cpu.program_counter + 1),
                cpu.bus
                    .mem_read(cpu.program_counter + 1)
                    .wrapping_add(cpu.register_y),
                value
            ))
        }
    };

    pad_string(opcode_string, 32)
}

fn registers_string(cpu: &CPU) -> String {
    format!(
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} ",
        cpu.register_a,
        cpu.register_x,
        cpu.register_y,
        cpu.status.get_status_byte(),
        cpu.stack_pointer,
    )
}
