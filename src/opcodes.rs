use super::memory;

use std::collections::HashMap;

use lazy_static::lazy_static;

pub struct OpCode {
    pub name: &'static str,
    pub bytes: i8,
    pub cycles: i8,
    pub address_mode: AddressingMode,
}

impl OpCode {
    pub fn new(name: &'static str, bytes: i8, cycles: i8, address_mode: AddressingMode) -> Self {
        OpCode {
            name,
            bytes,
            cycles,
            address_mode,
        }
    }
}

pub enum AddressingMode {
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

lazy_static! {
    pub static ref OPSCODES_MAP: HashMap<u8, OpCode> = {
        let mut codes = HashMap::new();
        codes.insert(0x00, OpCode::new("BRK", 1, 7, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x01, OpCode::new("ORA", 2, 6, AddressingMode::IndirectX)); // extras 0
        codes.insert(0x05, OpCode::new("ORA", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x06, OpCode::new("ASL", 2, 5, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x08, OpCode::new("PHP", 1, 3, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x09, OpCode::new("ORA", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0x0a, OpCode::new("ASL", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x0d, OpCode::new("ORA", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0x0e, OpCode::new("ASL", 3, 6, AddressingMode::Absolute)); // extras 0
        codes.insert(0x10, OpCode::new("BPL", 2, 2, AddressingMode::Immediate)); // extras 2
        codes.insert(0x11, OpCode::new("ORA", 2, 5, AddressingMode::IndirectY)); // extras 1
        codes.insert(0x15, OpCode::new("ORA", 2, 4, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0x16, OpCode::new("ASL", 2, 6, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0x18, OpCode::new("CLC", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x19, OpCode::new("ORA", 3, 4, AddressingMode::AbsoluteY)); // extras 1
        codes.insert(0x1d, OpCode::new("ORA", 3, 4, AddressingMode::AbsoluteX)); // extras 1
        codes.insert(0x1e, OpCode::new("ASL", 3, 7, AddressingMode::AbsoluteX)); // extras 0
        codes.insert(0x20, OpCode::new("JSR", 3, 6, AddressingMode::Absolute)); // extras 0
        codes.insert(0x21, OpCode::new("AND", 2, 6, AddressingMode::IndirectX)); // extras 0
        codes.insert(0x24, OpCode::new("BIT", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x25, OpCode::new("AND", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x26, OpCode::new("ROL", 2, 5, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x28, OpCode::new("PLP", 1, 4, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x29, OpCode::new("AND", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0x2a, OpCode::new("ROL", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x2c, OpCode::new("BIT", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0x2d, OpCode::new("AND", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0x2e, OpCode::new("ROL", 3, 6, AddressingMode::Absolute)); // extras 0
        codes.insert(0x30, OpCode::new("BMI", 2, 2, AddressingMode::Immediate)); // extras 2
        codes.insert(0x31, OpCode::new("AND", 2, 5, AddressingMode::IndirectY)); // extras 1
        codes.insert(0x35, OpCode::new("AND", 2, 4, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0x36, OpCode::new("ROL", 2, 6, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0x38, OpCode::new("SEC", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x39, OpCode::new("AND", 3, 4, AddressingMode::AbsoluteY)); // extras 1
        codes.insert(0x3d, OpCode::new("AND", 3, 4, AddressingMode::AbsoluteX)); // extras 1
        codes.insert(0x3e, OpCode::new("ROL", 3, 7, AddressingMode::AbsoluteX)); // extras 0
        codes.insert(0x40, OpCode::new("RTI", 1, 6, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x41, OpCode::new("EOR", 2, 6, AddressingMode::IndirectX)); // extras 0
        codes.insert(0x45, OpCode::new("EOR", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x46, OpCode::new("LSR", 2, 5, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x48, OpCode::new("PHA", 1, 3, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x49, OpCode::new("EOR", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0x4a, OpCode::new("LSR", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x4c, OpCode::new("JMP", 3, 3, AddressingMode::Absolute)); // extras 0
        codes.insert(0x4d, OpCode::new("EOR", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0x4e, OpCode::new("LSR", 3, 6, AddressingMode::Absolute)); // extras 0
        codes.insert(0x50, OpCode::new("BVC", 2, 2, AddressingMode::Immediate)); // extras 2
        codes.insert(0x51, OpCode::new("EOR", 2, 5, AddressingMode::IndirectY)); // extras 1
        codes.insert(0x55, OpCode::new("EOR", 2, 4, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0x56, OpCode::new("LSR", 2, 6, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0x58, OpCode::new("CLI", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x59, OpCode::new("EOR", 3, 4, AddressingMode::AbsoluteY)); // extras 1
        codes.insert(0x5d, OpCode::new("EOR", 3, 4, AddressingMode::AbsoluteX)); // extras 1
        codes.insert(0x5e, OpCode::new("LSR", 3, 7, AddressingMode::AbsoluteX)); // extras 0
        codes.insert(0x60, OpCode::new("RTS", 1, 6, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x61, OpCode::new("ADC", 2, 6, AddressingMode::IndirectX)); // extras 0
        codes.insert(0x65, OpCode::new("ADC", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x66, OpCode::new("ROR", 2, 5, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x68, OpCode::new("PLA", 1, 4, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x69, OpCode::new("ADC", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0x6a, OpCode::new("ROR", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x6c, OpCode::new("JMP", 3, 5, AddressingMode::Indirect)); // extras 0
        codes.insert(0x6d, OpCode::new("ADC", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0x6e, OpCode::new("ROR", 3, 6, AddressingMode::Absolute)); // extras 0
        codes.insert(0x70, OpCode::new("BVS", 2, 2, AddressingMode::Immediate)); // extras 2
        codes.insert(0x71, OpCode::new("ADC", 2, 5, AddressingMode::IndirectY)); // extras 1
        codes.insert(0x75, OpCode::new("ADC", 2, 4, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0x76, OpCode::new("ROR", 2, 6, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0x78, OpCode::new("SEI", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x79, OpCode::new("ADC", 3, 4, AddressingMode::AbsoluteY)); // extras 1
        codes.insert(0x7d, OpCode::new("ADC", 3, 4, AddressingMode::AbsoluteX)); // extras 1
        codes.insert(0x7e, OpCode::new("ROR", 3, 7, AddressingMode::AbsoluteX)); // extras 0
        codes.insert(0x81, OpCode::new("STA", 2, 6, AddressingMode::IndirectX)); // extras 0
        codes.insert(0x84, OpCode::new("STY", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x85, OpCode::new("STA", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x86, OpCode::new("STX", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0x88, OpCode::new("DEY", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x8a, OpCode::new("TXA", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x8c, OpCode::new("STY", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0x8d, OpCode::new("STA", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0x8e, OpCode::new("STX", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0x90, OpCode::new("BCC", 2, 2, AddressingMode::Immediate)); // extras 2
        codes.insert(0x91, OpCode::new("STA", 2, 6, AddressingMode::IndirectY)); // extras 0
        codes.insert(0x94, OpCode::new("STY", 2, 4, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0x95, OpCode::new("STA", 2, 4, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0x96, OpCode::new("STX", 2, 4, AddressingMode::ZeroPageY)); // extras 0
        codes.insert(0x98, OpCode::new("TYA", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x99, OpCode::new("STA", 3, 5, AddressingMode::AbsoluteY)); // extras 0
        codes.insert(0x9a, OpCode::new("TXS", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0x9d, OpCode::new("STA", 3, 5, AddressingMode::AbsoluteX)); // extras 0
        codes.insert(0xa0, OpCode::new("LDY", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0xa1, OpCode::new("LDA", 2, 6, AddressingMode::IndirectX)); // extras 0
        codes.insert(0xa2, OpCode::new("LDX", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0xa4, OpCode::new("LDY", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0xa5, OpCode::new("LDA", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0xa6, OpCode::new("LDX", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0xa8, OpCode::new("TAY", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0xa9, OpCode::new("LDA", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0xaa, OpCode::new("TAX", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0xac, OpCode::new("LDY", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0xad, OpCode::new("LDA", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0xae, OpCode::new("LDX", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0xb0, OpCode::new("BCS", 2, 2, AddressingMode::Immediate)); // extras 2
        codes.insert(0xb1, OpCode::new("LDA", 2, 5, AddressingMode::IndirectY)); // extras 1
        codes.insert(0xb4, OpCode::new("LDY", 2, 4, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0xb5, OpCode::new("LDA", 2, 4, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0xb6, OpCode::new("LDX", 2, 4, AddressingMode::ZeroPageY)); // extras 0
        codes.insert(0xb8, OpCode::new("CLV", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0xb9, OpCode::new("LDA", 3, 4, AddressingMode::AbsoluteY)); // extras 1
        codes.insert(0xba, OpCode::new("TSX", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0xbc, OpCode::new("LDY", 3, 4, AddressingMode::AbsoluteX)); // extras 1
        codes.insert(0xbd, OpCode::new("LDA", 3, 4, AddressingMode::AbsoluteX)); // extras 1
        codes.insert(0xbe, OpCode::new("LDX", 3, 4, AddressingMode::AbsoluteY)); // extras 1
        codes.insert(0xc0, OpCode::new("CPY", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0xc1, OpCode::new("CMP", 2, 6, AddressingMode::IndirectX)); // extras 0
        codes.insert(0xc4, OpCode::new("CPY", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0xc5, OpCode::new("CMP", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0xc6, OpCode::new("DEC", 2, 5, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0xc8, OpCode::new("INY", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0xc9, OpCode::new("CMP", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0xca, OpCode::new("DEX", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0xcc, OpCode::new("CPY", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0xcd, OpCode::new("CMP", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0xce, OpCode::new("DEC", 3, 6, AddressingMode::Absolute)); // extras 0
        codes.insert(0xd0, OpCode::new("BNE", 2, 2, AddressingMode::Immediate)); // extras 2
        codes.insert(0xd1, OpCode::new("CMP", 2, 5, AddressingMode::IndirectY)); // extras 1
        codes.insert(0xd5, OpCode::new("CMP", 2, 4, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0xd6, OpCode::new("DEC", 2, 6, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0xd8, OpCode::new("CLD", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0xd9, OpCode::new("CMP", 3, 4, AddressingMode::AbsoluteY)); // extras 1
        codes.insert(0xdd, OpCode::new("CMP", 3, 4, AddressingMode::AbsoluteX)); // extras 1
        codes.insert(0xde, OpCode::new("DEC", 3, 7, AddressingMode::AbsoluteX)); // extras 0
        codes.insert(0xe0, OpCode::new("CPX", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0xe1, OpCode::new("SBC", 2, 6, AddressingMode::IndirectX)); // extras 0
        codes.insert(0xe4, OpCode::new("CPX", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0xe5, OpCode::new("SBC", 2, 3, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0xe6, OpCode::new("INC", 2, 5, AddressingMode::ZeroPage)); // extras 0
        codes.insert(0xe8, OpCode::new("INX", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0xe9, OpCode::new("SBC", 2, 2, AddressingMode::Immediate)); // extras 0
        codes.insert(0xea, OpCode::new("NOP", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0xec, OpCode::new("CPX", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0xed, OpCode::new("SBC", 3, 4, AddressingMode::Absolute)); // extras 0
        codes.insert(0xee, OpCode::new("INC", 3, 6, AddressingMode::Absolute)); // extras 0
        codes.insert(0xf0, OpCode::new("BEQ", 2, 2, AddressingMode::Immediate)); // extras 2
        codes.insert(0xf1, OpCode::new("SBC", 2, 5, AddressingMode::IndirectY)); // extras 1
        codes.insert(0xf5, OpCode::new("SBC", 2, 4, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0xf6, OpCode::new("INC", 2, 6, AddressingMode::ZeroPageX)); // extras 0
        codes.insert(0xf8, OpCode::new("SED", 1, 2, AddressingMode::NoneAddressing)); // extras 0
        codes.insert(0xf9, OpCode::new("SBC", 3, 4, AddressingMode::AbsoluteY)); // extras 1
        codes.insert(0xfd, OpCode::new("SBC", 3, 4, AddressingMode::AbsoluteX)); // extras 1
        codes.insert(0xfe, OpCode::new("INC", 3, 7, AddressingMode::AbsoluteX)); // extras 0

        codes
    };
}
