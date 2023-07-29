use crate::cpu::CPU;
use crate::memory::Mem;
use crate::opcodes::{AddressingMode, OpCode, OpCodeDetail};

pub fn trace(cpu: &CPU) -> String {
    let mut full_trace = String::new();

    let program_counter = program_counter_string(cpu);
    let cpu_opcode = cpu_opcode_string(cpu);
    let cpu_assembly = cpu_opcode_assembly_string(cpu);
    let registers = registers_string(cpu);

    full_trace.push_str(&program_counter);
    full_trace.push_str(&cpu_opcode);
    full_trace.push_str(&cpu_assembly);
    full_trace.push_str(&registers);

    println!("{}", full_trace);

    full_trace
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
        AddressingMode::Absolute => opcode_string.push_str(&format!(
            " ${:04X}",
            cpu.bus.mem_read_u16(cpu.program_counter + 1)
        )),
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
            let offset = cpu.bus.mem_read(cpu.program_counter + 1) as u16;
            opcode_string.push_str(&format!(" ${:02X}", cpu.program_counter + 2 + offset))
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
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
        cpu.register_a,
        cpu.register_x,
        cpu.register_y,
        cpu.status.get_status_byte(),
        cpu.stack_pointer,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cartridge::Cartridge;

    #[test]
    fn test_format_trace() {
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
        contents.extend([0x01; 16384 * 2]);
        contents.extend([0x02; 8192 * 2]);

        let cartridge = Cartridge::new(&contents);

        let mut bus = Bus::new(cartridge);
        bus.mem_write(100, 0xa2);
        bus.mem_write(101, 0x01);
        bus.mem_write(102, 0xca);
        bus.mem_write(103, 0x88);
        bus.mem_write(104, 0x00);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_a = 1;
        cpu.register_x = 2;
        cpu.register_y = 3;

        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });

        assert_eq!(
            "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
            result[0]
        );
        assert_eq!(
            "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
            result[1]
        );
        assert_eq!(
            "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
            result[2]
        );
    }

    #[test]
    fn test_format_mem_access() {
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
        contents.extend([0x01; 16384 * 2]);
        contents.extend([0x02; 8192 * 2]);

        let cartridge = Cartridge::new(&contents);

        let mut bus = Bus::new(cartridge);
        // ORA ($33), Y
        bus.mem_write(0x64, 0x11);
        bus.mem_write(0x65, 0x33);

        //data
        bus.mem_write(0x33, 00);
        bus.mem_write(0x34, 04);

        //target cell
        bus.mem_write(0x400, 0xAA);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_y = 0;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
            "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
            result[0]
        );
    }
}
