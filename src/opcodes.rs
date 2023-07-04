use std::collections::HashMap;

pub enum OpCode {
    X00,
    X01,
    X05,
    X06,
    X08,
    X09,
    X0a,
    X0d,
    X0e,
    X10,
    X11,
    X15,
    X16,
    X18,
    X19,
    X1d,
    X1e,
    X20,
    X21,
    X24,
    X25,
    X26,
    X28,
    X29,
    X2a,
    X2c,
    X2d,
    X2e,
    X30,
    X31,
    X35,
    X36,
    X38,
    X39,
    X3d,
    X3e,
    X40,
    X41,
    X45,
    X46,
    X48,
    X49,
    X4a,
    X4c,
    X4d,
    X4e,
    X50,
    X51,
    X55,
    X56,
    X58,
    X59,
    X5d,
    X5e,
    X60,
    X61,
    X65,
    X66,
    X68,
    X69,
    X6a,
    X6c,
    X6d,
    X6e,
    X70,
    X71,
    X75,
    X76,
    X78,
    X79,
    X7d,
    X7e,
    X81,
    X84,
    X85,
    X86,
    X88,
    X8a,
    X8c,
    X8d,
    X8e,
    X90,
    X91,
    X94,
    X95,
    X96,
    X98,
    X99,
    X9a,
    X9d,
    Xa0,
    Xa1,
    Xa2,
    Xa4,
    Xa5,
    Xa6,
    Xa8,
    Xa9,
    Xaa,
    Xac,
    Xad,
    Xae,
    Xb0,
    Xb1,
    Xb4,
    Xb5,
    Xb6,
    Xb8,
    Xb9,
    Xba,
    Xbc,
    Xbd,
    Xbe,
    Xc0,
    Xc1,
    Xc4,
    Xc5,
    Xc6,
    Xc8,
    Xc9,
    Xca,
    Xcc,
    Xcd,
    Xce,
    Xd0,
    Xd1,
    Xd5,
    Xd6,
    Xd8,
    Xd9,
    Xdd,
    Xde,
    Xe0,
    Xe1,
    Xe4,
    Xe5,
    Xe6,
    Xe8,
    Xe9,
    Xea,
    Xec,
    Xed,
    Xee,
    Xf0,
    Xf1,
    Xf5,
    Xf6,
    Xf8,
    Xf9,
    Xfd,
    Xfe,
}

pub struct OpCodeDetail {
    pub name: &'static str,
    pub bytes: u8,
    pub cycles: i8,
    pub address_mode: AddressingMode,
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
    Implied,
    Relative,
    Accumulator,
}

pub fn get_opcode_detail(op_code: OpCode) -> OpCodeDetail {
    match op_code {
        OpCode::X00 => OpCodeDetail::new("BRK", 1, 7, AddressingMode::Implied),
        OpCode::X01 => OpCodeDetail::new("ORA", 2, 6, AddressingMode::IndirectX),
        OpCode::X05 => OpCodeDetail::new("ORA", 2, 3, AddressingMode::ZeroPage),
        OpCode::X06 => OpCodeDetail::new("ASL", 2, 5, AddressingMode::ZeroPage),
        OpCode::X08 => OpCodeDetail::new("PHP", 1, 3, AddressingMode::Implied),
        OpCode::X09 => OpCodeDetail::new("ORA", 2, 2, AddressingMode::Immediate),
        OpCode::X0a => OpCodeDetail::new("ASL", 1, 2, AddressingMode::Accumulator),
        OpCode::X0d => OpCodeDetail::new("ORA", 3, 4, AddressingMode::Absolute),
        OpCode::X0e => OpCodeDetail::new("ASL", 3, 6, AddressingMode::Absolute),
        OpCode::X10 => OpCodeDetail::new("BPL", 2, 2, AddressingMode::Relative),
        OpCode::X11 => OpCodeDetail::new("ORA", 2, 5, AddressingMode::IndirectY),
        OpCode::X15 => OpCodeDetail::new("ORA", 2, 4, AddressingMode::ZeroPageX),
        OpCode::X16 => OpCodeDetail::new("ASL", 2, 6, AddressingMode::ZeroPageX),
        OpCode::X18 => OpCodeDetail::new("CLC", 1, 2, AddressingMode::Implied),
        OpCode::X19 => OpCodeDetail::new("ORA", 3, 4, AddressingMode::AbsoluteY),
        OpCode::X1d => OpCodeDetail::new("ORA", 3, 4, AddressingMode::AbsoluteX),
        OpCode::X1e => OpCodeDetail::new("ASL", 3, 7, AddressingMode::AbsoluteX),
        OpCode::X20 => OpCodeDetail::new("JSR", 3, 6, AddressingMode::Absolute),
        OpCode::X21 => OpCodeDetail::new("AND", 2, 6, AddressingMode::IndirectX),
        OpCode::X24 => OpCodeDetail::new("BIT", 2, 3, AddressingMode::ZeroPage),
        OpCode::X25 => OpCodeDetail::new("AND", 2, 3, AddressingMode::ZeroPage),
        OpCode::X26 => OpCodeDetail::new("ROL", 2, 5, AddressingMode::ZeroPage),
        OpCode::X28 => OpCodeDetail::new("PLP", 1, 4, AddressingMode::Implied),
        OpCode::X29 => OpCodeDetail::new("AND", 2, 2, AddressingMode::Immediate),
        OpCode::X2a => OpCodeDetail::new("ROL", 1, 2, AddressingMode::Accumulator),
        OpCode::X2c => OpCodeDetail::new("BIT", 3, 4, AddressingMode::Absolute),
        OpCode::X2d => OpCodeDetail::new("AND", 3, 4, AddressingMode::Absolute),
        OpCode::X2e => OpCodeDetail::new("ROL", 3, 6, AddressingMode::Absolute),
        OpCode::X30 => OpCodeDetail::new("BMI", 2, 2, AddressingMode::Relative),
        OpCode::X31 => OpCodeDetail::new("AND", 2, 5, AddressingMode::IndirectY),
        OpCode::X35 => OpCodeDetail::new("AND", 2, 4, AddressingMode::ZeroPageX),
        OpCode::X36 => OpCodeDetail::new("ROL", 2, 6, AddressingMode::ZeroPageX),
        OpCode::X38 => OpCodeDetail::new("SEC", 1, 2, AddressingMode::Implied),
        OpCode::X39 => OpCodeDetail::new("AND", 3, 4, AddressingMode::AbsoluteY),
        OpCode::X3d => OpCodeDetail::new("AND", 3, 4, AddressingMode::AbsoluteX),
        OpCode::X3e => OpCodeDetail::new("ROL", 3, 7, AddressingMode::AbsoluteX),
        OpCode::X40 => OpCodeDetail::new("RTI", 1, 6, AddressingMode::Implied),
        OpCode::X41 => OpCodeDetail::new("EOR", 2, 6, AddressingMode::IndirectX),
        OpCode::X45 => OpCodeDetail::new("EOR", 2, 3, AddressingMode::ZeroPage),
        OpCode::X46 => OpCodeDetail::new("LSR", 2, 5, AddressingMode::ZeroPage),
        OpCode::X48 => OpCodeDetail::new("PHA", 1, 3, AddressingMode::Implied),
        OpCode::X49 => OpCodeDetail::new("EOR", 2, 2, AddressingMode::Immediate),
        OpCode::X4a => OpCodeDetail::new("LSR", 1, 2, AddressingMode::Accumulator),
        OpCode::X4c => OpCodeDetail::new("JMP", 3, 3, AddressingMode::Absolute),
        OpCode::X4d => OpCodeDetail::new("EOR", 3, 4, AddressingMode::Absolute),
        OpCode::X4e => OpCodeDetail::new("LSR", 3, 6, AddressingMode::Absolute),
        OpCode::X50 => OpCodeDetail::new("BVC", 2, 2, AddressingMode::Relative),
        OpCode::X51 => OpCodeDetail::new("EOR", 2, 5, AddressingMode::IndirectY),
        OpCode::X55 => OpCodeDetail::new("EOR", 2, 4, AddressingMode::ZeroPageX),
        OpCode::X56 => OpCodeDetail::new("LSR", 2, 6, AddressingMode::ZeroPageX),
        OpCode::X58 => OpCodeDetail::new("CLI", 1, 2, AddressingMode::Implied),
        OpCode::X59 => OpCodeDetail::new("EOR", 3, 4, AddressingMode::AbsoluteY),
        OpCode::X5d => OpCodeDetail::new("EOR", 3, 4, AddressingMode::AbsoluteX),
        OpCode::X5e => OpCodeDetail::new("LSR", 3, 7, AddressingMode::AbsoluteX),
        OpCode::X60 => OpCodeDetail::new("RTS", 1, 6, AddressingMode::Implied),
        OpCode::X61 => OpCodeDetail::new("ADC", 2, 6, AddressingMode::IndirectX),
        OpCode::X65 => OpCodeDetail::new("ADC", 2, 3, AddressingMode::ZeroPage),
        OpCode::X66 => OpCodeDetail::new("ROR", 2, 5, AddressingMode::ZeroPage),
        OpCode::X68 => OpCodeDetail::new("PLA", 1, 4, AddressingMode::Implied),
        OpCode::X69 => OpCodeDetail::new("ADC", 2, 2, AddressingMode::Immediate),
        OpCode::X6a => OpCodeDetail::new("ROR", 1, 2, AddressingMode::Accumulator),
        OpCode::X6c => OpCodeDetail::new("JMP", 3, 5, AddressingMode::Indirect),
        OpCode::X6d => OpCodeDetail::new("ADC", 3, 4, AddressingMode::Absolute),
        OpCode::X6e => OpCodeDetail::new("ROR", 3, 6, AddressingMode::Absolute),
        OpCode::X70 => OpCodeDetail::new("BVS", 2, 2, AddressingMode::Relative),
        OpCode::X71 => OpCodeDetail::new("ADC", 2, 5, AddressingMode::IndirectY),
        OpCode::X75 => OpCodeDetail::new("ADC", 2, 4, AddressingMode::ZeroPageX),
        OpCode::X76 => OpCodeDetail::new("ROR", 2, 6, AddressingMode::ZeroPageX),
        OpCode::X78 => OpCodeDetail::new("SEI", 1, 2, AddressingMode::Implied),
        OpCode::X79 => OpCodeDetail::new("ADC", 3, 4, AddressingMode::AbsoluteY),
        OpCode::X7d => OpCodeDetail::new("ADC", 3, 4, AddressingMode::AbsoluteX),
        OpCode::X7e => OpCodeDetail::new("ROR", 3, 7, AddressingMode::AbsoluteX),
        OpCode::X81 => OpCodeDetail::new("STA", 2, 6, AddressingMode::IndirectX),
        OpCode::X84 => OpCodeDetail::new("STY", 2, 3, AddressingMode::ZeroPage),
        OpCode::X85 => OpCodeDetail::new("STA", 2, 3, AddressingMode::ZeroPage),
        OpCode::X86 => OpCodeDetail::new("STX", 2, 3, AddressingMode::ZeroPage),
        OpCode::X88 => OpCodeDetail::new("DEY", 1, 2, AddressingMode::Implied),
        OpCode::X8a => OpCodeDetail::new("TXA", 1, 2, AddressingMode::Implied),
        OpCode::X8c => OpCodeDetail::new("STY", 3, 4, AddressingMode::Absolute),
        OpCode::X8d => OpCodeDetail::new("STA", 3, 4, AddressingMode::Absolute),
        OpCode::X8e => OpCodeDetail::new("STX", 3, 4, AddressingMode::Absolute),
        OpCode::X90 => OpCodeDetail::new("BCC", 2, 2, AddressingMode::Relative),
        OpCode::X91 => OpCodeDetail::new("STA", 2, 6, AddressingMode::IndirectY),
        OpCode::X94 => OpCodeDetail::new("STY", 2, 4, AddressingMode::ZeroPageX),
        OpCode::X95 => OpCodeDetail::new("STA", 2, 4, AddressingMode::ZeroPageX),
        OpCode::X96 => OpCodeDetail::new("STX", 2, 4, AddressingMode::ZeroPageY),
        OpCode::X98 => OpCodeDetail::new("TYA", 1, 2, AddressingMode::Implied),
        OpCode::X99 => OpCodeDetail::new("STA", 3, 5, AddressingMode::AbsoluteY),
        OpCode::X9a => OpCodeDetail::new("TXS", 1, 2, AddressingMode::Implied),
        OpCode::X9d => OpCodeDetail::new("STA", 3, 5, AddressingMode::AbsoluteX),
        OpCode::Xa0 => OpCodeDetail::new("LDY", 2, 2, AddressingMode::Immediate),
        OpCode::Xa1 => OpCodeDetail::new("LDA", 2, 6, AddressingMode::IndirectX),
        OpCode::Xa2 => OpCodeDetail::new("LDX", 2, 2, AddressingMode::Immediate),
        OpCode::Xa4 => OpCodeDetail::new("LDY", 2, 3, AddressingMode::ZeroPage),
        OpCode::Xa5 => OpCodeDetail::new("LDA", 2, 3, AddressingMode::ZeroPage),
        OpCode::Xa6 => OpCodeDetail::new("LDX", 2, 3, AddressingMode::ZeroPage),
        OpCode::Xa8 => OpCodeDetail::new("TAY", 1, 2, AddressingMode::Implied),
        OpCode::Xa9 => OpCodeDetail::new("LDA", 2, 2, AddressingMode::Immediate),
        OpCode::Xaa => OpCodeDetail::new("TAX", 1, 2, AddressingMode::Implied),
        OpCode::Xac => OpCodeDetail::new("LDY", 3, 4, AddressingMode::Absolute),
        OpCode::Xad => OpCodeDetail::new("LDA", 3, 4, AddressingMode::Absolute),
        OpCode::Xae => OpCodeDetail::new("LDX", 3, 4, AddressingMode::Absolute),
        OpCode::Xb0 => OpCodeDetail::new("BCS", 2, 2, AddressingMode::Relative),
        OpCode::Xb1 => OpCodeDetail::new("LDA", 2, 5, AddressingMode::IndirectY),
        OpCode::Xb4 => OpCodeDetail::new("LDY", 2, 4, AddressingMode::ZeroPageX),
        OpCode::Xb5 => OpCodeDetail::new("LDA", 2, 4, AddressingMode::ZeroPageX),
        OpCode::Xb6 => OpCodeDetail::new("LDX", 2, 4, AddressingMode::ZeroPageY),
        OpCode::Xb8 => OpCodeDetail::new("CLV", 1, 2, AddressingMode::Implied),
        OpCode::Xb9 => OpCodeDetail::new("LDA", 3, 4, AddressingMode::AbsoluteY),
        OpCode::Xba => OpCodeDetail::new("TSX", 1, 2, AddressingMode::Implied),
        OpCode::Xbc => OpCodeDetail::new("LDY", 3, 4, AddressingMode::AbsoluteX),
        OpCode::Xbd => OpCodeDetail::new("LDA", 3, 4, AddressingMode::AbsoluteX),
        OpCode::Xbe => OpCodeDetail::new("LDX", 3, 4, AddressingMode::AbsoluteY),
        OpCode::Xc0 => OpCodeDetail::new("CPY", 2, 2, AddressingMode::Immediate),
        OpCode::Xc1 => OpCodeDetail::new("CMP", 2, 6, AddressingMode::IndirectX),
        OpCode::Xc4 => OpCodeDetail::new("CPY", 2, 3, AddressingMode::ZeroPage),
        OpCode::Xc5 => OpCodeDetail::new("CMP", 2, 3, AddressingMode::ZeroPage),
        OpCode::Xc6 => OpCodeDetail::new("DEC", 2, 5, AddressingMode::ZeroPage),
        OpCode::Xc8 => OpCodeDetail::new("INY", 1, 2, AddressingMode::Implied),
        OpCode::Xc9 => OpCodeDetail::new("CMP", 2, 2, AddressingMode::Immediate),
        OpCode::Xca => OpCodeDetail::new("DEX", 1, 2, AddressingMode::Implied),
        OpCode::Xcc => OpCodeDetail::new("CPY", 3, 4, AddressingMode::Absolute),
        OpCode::Xcd => OpCodeDetail::new("CMP", 3, 4, AddressingMode::Absolute),
        OpCode::Xce => OpCodeDetail::new("DEC", 3, 6, AddressingMode::Absolute),
        OpCode::Xd0 => OpCodeDetail::new("BNE", 2, 2, AddressingMode::Relative),
        OpCode::Xd1 => OpCodeDetail::new("CMP", 2, 5, AddressingMode::IndirectY),
        OpCode::Xd5 => OpCodeDetail::new("CMP", 2, 4, AddressingMode::ZeroPageX),
        OpCode::Xd6 => OpCodeDetail::new("DEC", 2, 6, AddressingMode::ZeroPageX),
        OpCode::Xd8 => OpCodeDetail::new("CLD", 1, 2, AddressingMode::Implied),
        OpCode::Xd9 => OpCodeDetail::new("CMP", 3, 4, AddressingMode::AbsoluteY),
        OpCode::Xdd => OpCodeDetail::new("CMP", 3, 4, AddressingMode::AbsoluteX),
        OpCode::Xde => OpCodeDetail::new("DEC", 3, 7, AddressingMode::AbsoluteX),
        OpCode::Xe0 => OpCodeDetail::new("CPX", 2, 2, AddressingMode::Immediate),
        OpCode::Xe1 => OpCodeDetail::new("SBC", 2, 6, AddressingMode::IndirectX),
        OpCode::Xe4 => OpCodeDetail::new("CPX", 2, 3, AddressingMode::ZeroPage),
        OpCode::Xe5 => OpCodeDetail::new("SBC", 2, 3, AddressingMode::ZeroPage),
        OpCode::Xe6 => OpCodeDetail::new("INC", 2, 5, AddressingMode::ZeroPage),
        OpCode::Xe8 => OpCodeDetail::new("INX", 1, 2, AddressingMode::Implied),
        OpCode::Xe9 => OpCodeDetail::new("SBC", 2, 2, AddressingMode::Immediate),
        OpCode::Xea => OpCodeDetail::new("NOP", 1, 2, AddressingMode::Implied),
        OpCode::Xec => OpCodeDetail::new("CPX", 3, 4, AddressingMode::Absolute),
        OpCode::Xed => OpCodeDetail::new("SBC", 3, 4, AddressingMode::Absolute),
        OpCode::Xee => OpCodeDetail::new("INC", 3, 6, AddressingMode::Absolute),
        OpCode::Xf0 => OpCodeDetail::new("BEQ", 2, 2, AddressingMode::Relative),
        OpCode::Xf1 => OpCodeDetail::new("SBC", 2, 5, AddressingMode::IndirectY),
        OpCode::Xf5 => OpCodeDetail::new("SBC", 2, 4, AddressingMode::ZeroPageX),
        OpCode::Xf6 => OpCodeDetail::new("INC", 2, 6, AddressingMode::ZeroPageX),
        OpCode::Xf8 => OpCodeDetail::new("SED", 1, 2, AddressingMode::Implied),
        OpCode::Xf9 => OpCodeDetail::new("SBC", 3, 4, AddressingMode::AbsoluteY),
        OpCode::Xfd => OpCodeDetail::new("SBC", 3, 4, AddressingMode::AbsoluteX),
        OpCode::Xfe => OpCodeDetail::new("INC", 3, 7, AddressingMode::AbsoluteX),
    }
}

// lazy_static! {
//     pub static ref OPSCODES_MAP: HashMap<u8, OpCode> = {
//         let mut codes = HashMap::new();
//         codes.insert(0x00, OpCode::new("BRK", 1, 7, AddressingMode::Implied));
//         codes.insert(0x01, OpCode::new("ORA", 2, 6, AddressingMode::IndirectX));
//         codes.insert(0x05, OpCode::new("ORA", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0x06, OpCode::new("ASL", 2, 5, AddressingMode::ZeroPage));
//         codes.insert(0x08, OpCode::new("PHP", 1, 3, AddressingMode::Implied));
//         codes.insert(0x09, OpCode::new("ORA", 2, 2, AddressingMode::Immediate));
//         codes.insert(0x0a, OpCode::new("ASL", 1, 2, AddressingMode::Accumulator));
//         codes.insert(0x0d, OpCode::new("ORA", 3, 4, AddressingMode::Absolute));
//         codes.insert(0x0e, OpCode::new("ASL", 3, 6, AddressingMode::Absolute));
//         codes.insert(0x10, OpCode::new("BPL", 2, 2, AddressingMode::Relative)); // extras 2
//         codes.insert(0x11, OpCode::new("ORA", 2, 5, AddressingMode::IndirectY)); // extras 1
//         codes.insert(0x15, OpCode::new("ORA", 2, 4, AddressingMode::ZeroPageX));
//         codes.insert(0x16, OpCode::new("ASL", 2, 6, AddressingMode::ZeroPageX));
//         codes.insert(0x18, OpCode::new("CLC", 1, 2, AddressingMode::Implied));
//         codes.insert(0x19, OpCode::new("ORA", 3, 4, AddressingMode::AbsoluteY)); // extras 1
//         codes.insert(0x1d, OpCode::new("ORA", 3, 4, AddressingMode::AbsoluteX)); // extras 1
//         codes.insert(0x1e, OpCode::new("ASL", 3, 7, AddressingMode::AbsoluteX));
//         codes.insert(0x20, OpCode::new("JSR", 3, 6, AddressingMode::Absolute));
//         codes.insert(0x21, OpCode::new("AND", 2, 6, AddressingMode::IndirectX));
//         codes.insert(0x24, OpCode::new("BIT", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0x25, OpCode::new("AND", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0x26, OpCode::new("ROL", 2, 5, AddressingMode::ZeroPage));
//         codes.insert(0x28, OpCode::new("PLP", 1, 4, AddressingMode::Implied));
//         codes.insert(0x29, OpCode::new("AND", 2, 2, AddressingMode::Immediate));
//         codes.insert(0x2a, OpCode::new("ROL", 1, 2, AddressingMode::Accumulator));
//         codes.insert(0x2c, OpCode::new("BIT", 3, 4, AddressingMode::Absolute));
//         codes.insert(0x2d, OpCode::new("AND", 3, 4, AddressingMode::Absolute));
//         codes.insert(0x2e, OpCode::new("ROL", 3, 6, AddressingMode::Absolute));
//         codes.insert(0x30, OpCode::new("BMI", 2, 2, AddressingMode::Relative)); // extras 2
//         codes.insert(0x31, OpCode::new("AND", 2, 5, AddressingMode::IndirectY)); // extras 1
//         codes.insert(0x35, OpCode::new("AND", 2, 4, AddressingMode::ZeroPageX));
//         codes.insert(0x36, OpCode::new("ROL", 2, 6, AddressingMode::ZeroPageX));
//         codes.insert(0x38, OpCode::new("SEC", 1, 2, AddressingMode::Implied));
//         codes.insert(0x39, OpCode::new("AND", 3, 4, AddressingMode::AbsoluteY)); // extras 1
//         codes.insert(0x3d, OpCode::new("AND", 3, 4, AddressingMode::AbsoluteX)); // extras 1
//         codes.insert(0x3e, OpCode::new("ROL", 3, 7, AddressingMode::AbsoluteX));
//         codes.insert(0x40, OpCode::new("RTI", 1, 6, AddressingMode::Implied));
//         codes.insert(0x41, OpCode::new("EOR", 2, 6, AddressingMode::IndirectX));
//         codes.insert(0x45, OpCode::new("EOR", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0x46, OpCode::new("LSR", 2, 5, AddressingMode::ZeroPage));
//         codes.insert(0x48, OpCode::new("PHA", 1, 3, AddressingMode::Implied));
//         codes.insert(0x49, OpCode::new("EOR", 2, 2, AddressingMode::Immediate));
//         codes.insert(0x4a, OpCode::new("LSR", 1, 2, AddressingMode::Accumulator));
//         codes.insert(0x4c, OpCode::new("JMP", 3, 3, AddressingMode::Absolute));
//         codes.insert(0x4d, OpCode::new("EOR", 3, 4, AddressingMode::Absolute));
//         codes.insert(0x4e, OpCode::new("LSR", 3, 6, AddressingMode::Absolute));
//         codes.insert(0x50, OpCode::new("BVC", 2, 2, AddressingMode::Relative)); // extras 2
//         codes.insert(0x51, OpCode::new("EOR", 2, 5, AddressingMode::IndirectY)); // extras 1
//         codes.insert(0x55, OpCode::new("EOR", 2, 4, AddressingMode::ZeroPageX));
//         codes.insert(0x56, OpCode::new("LSR", 2, 6, AddressingMode::ZeroPageX));
//         codes.insert(0x58, OpCode::new("CLI", 1, 2, AddressingMode::Implied));
//         codes.insert(0x59, OpCode::new("EOR", 3, 4, AddressingMode::AbsoluteY)); // extras 1
//         codes.insert(0x5d, OpCode::new("EOR", 3, 4, AddressingMode::AbsoluteX)); // extras 1
//         codes.insert(0x5e, OpCode::new("LSR", 3, 7, AddressingMode::AbsoluteX));
//         codes.insert(0x60, OpCode::new("RTS", 1, 6, AddressingMode::Implied));
//         codes.insert(0x61, OpCode::new("ADC", 2, 6, AddressingMode::IndirectX));
//         codes.insert(0x65, OpCode::new("ADC", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0x66, OpCode::new("ROR", 2, 5, AddressingMode::ZeroPage));
//         codes.insert(0x68, OpCode::new("PLA", 1, 4, AddressingMode::Implied));
//         codes.insert(0x69, OpCode::new("ADC", 2, 2, AddressingMode::Immediate));
//         codes.insert(0x6a, OpCode::new("ROR", 1, 2, AddressingMode::Accumulator));
//         codes.insert(0x6c, OpCode::new("JMP", 3, 5, AddressingMode::Indirect));
//         codes.insert(0x6d, OpCode::new("ADC", 3, 4, AddressingMode::Absolute));
//         codes.insert(0x6e, OpCode::new("ROR", 3, 6, AddressingMode::Absolute));
//         codes.insert(0x70, OpCode::new("BVS", 2, 2, AddressingMode::Relative)); // extras 2
//         codes.insert(0x71, OpCode::new("ADC", 2, 5, AddressingMode::IndirectY)); // extras 1
//         codes.insert(0x75, OpCode::new("ADC", 2, 4, AddressingMode::ZeroPageX));
//         codes.insert(0x76, OpCode::new("ROR", 2, 6, AddressingMode::ZeroPageX));
//         codes.insert(0x78, OpCode::new("SEI", 1, 2, AddressingMode::Implied));
//         codes.insert(0x79, OpCode::new("ADC", 3, 4, AddressingMode::AbsoluteY)); // extras 1
//         codes.insert(0x7d, OpCode::new("ADC", 3, 4, AddressingMode::AbsoluteX)); // extras 1
//         codes.insert(0x7e, OpCode::new("ROR", 3, 7, AddressingMode::AbsoluteX));
//         codes.insert(0x81, OpCode::new("STA", 2, 6, AddressingMode::IndirectX));
//         codes.insert(0x84, OpCode::new("STY", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0x85, OpCode::new("STA", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0x86, OpCode::new("STX", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0x88, OpCode::new("DEY", 1, 2, AddressingMode::Implied));
//         codes.insert(0x8a, OpCode::new("TXA", 1, 2, AddressingMode::Implied));
//         codes.insert(0x8c, OpCode::new("STY", 3, 4, AddressingMode::Absolute));
//         codes.insert(0x8d, OpCode::new("STA", 3, 4, AddressingMode::Absolute));
//         codes.insert(0x8e, OpCode::new("STX", 3, 4, AddressingMode::Absolute));
//         codes.insert(0x90, OpCode::new("BCC", 2, 2, AddressingMode::Relative)); // extras 2
//         codes.insert(0x91, OpCode::new("STA", 2, 6, AddressingMode::IndirectY));
//         codes.insert(0x94, OpCode::new("STY", 2, 4, AddressingMode::ZeroPageX));
//         codes.insert(0x95, OpCode::new("STA", 2, 4, AddressingMode::ZeroPageX));
//         codes.insert(0x96, OpCode::new("STX", 2, 4, AddressingMode::ZeroPageY));
//         codes.insert(0x98, OpCode::new("TYA", 1, 2, AddressingMode::Implied));
//         codes.insert(0x99, OpCode::new("STA", 3, 5, AddressingMode::AbsoluteY));
//         codes.insert(0x9a, OpCode::new("TXS", 1, 2, AddressingMode::Implied));
//         codes.insert(0x9d, OpCode::new("STA", 3, 5, AddressingMode::AbsoluteX));
//         codes.insert(0xa0, OpCode::new("LDY", 2, 2, AddressingMode::Immediate));
//         codes.insert(0xa1, OpCode::new("LDA", 2, 6, AddressingMode::IndirectX));
//         codes.insert(0xa2, OpCode::new("LDX", 2, 2, AddressingMode::Immediate));
//         codes.insert(0xa4, OpCode::new("LDY", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0xa5, OpCode::new("LDA", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0xa6, OpCode::new("LDX", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0xa8, OpCode::new("TAY", 1, 2, AddressingMode::Implied));
//         codes.insert(0xa9, OpCode::new("LDA", 2, 2, AddressingMode::Immediate));
//         codes.insert(0xaa, OpCode::new("TAX", 1, 2, AddressingMode::Implied));
//         codes.insert(0xac, OpCode::new("LDY", 3, 4, AddressingMode::Absolute));
//         codes.insert(0xad, OpCode::new("LDA", 3, 4, AddressingMode::Absolute));
//         codes.insert(0xae, OpCode::new("LDX", 3, 4, AddressingMode::Absolute));
//         codes.insert(0xb0, OpCode::new("BCS", 2, 2, AddressingMode::Relative)); // extras 2
//         codes.insert(0xb1, OpCode::new("LDA", 2, 5, AddressingMode::IndirectY)); // extras 1
//         codes.insert(0xb4, OpCode::new("LDY", 2, 4, AddressingMode::ZeroPageX));
//         codes.insert(0xb5, OpCode::new("LDA", 2, 4, AddressingMode::ZeroPageX));
//         codes.insert(0xb6, OpCode::new("LDX", 2, 4, AddressingMode::ZeroPageY));
//         codes.insert(0xb8, OpCode::new("CLV", 1, 2, AddressingMode::Implied));
//         codes.insert(0xb9, OpCode::new("LDA",3, 4, AddressingMode::AbsoluteY)); // extras 1
//         codes.insert(0xba, OpCode::new("TSX", 1, 2, AddressingMode::Implied));
//         codes.insert(0xbc, OpCode::new("LDY", 3, 4, AddressingMode::AbsoluteX)); // extras 1
//         codes.insert(0xbd, OpCode::new("LDA", 3, 4, AddressingMode::AbsoluteX)); // extras 1
//         codes.insert(0xbe, OpCode::new("LDX", 3, 4, AddressingMode::AbsoluteY)); // extras 1
//         codes.insert(0xc0, OpCode::new("CPY", 2, 2, AddressingMode::Immediate));
//         codes.insert(0xc1, OpCode::new("CMP", 2, 6, AddressingMode::IndirectX));
//         codes.insert(0xc4, OpCode::new("CPY", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0xc5, OpCode::new("CMP", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0xc6, OpCode::new("DEC", 2, 5, AddressingMode::ZeroPage));
//         codes.insert(0xc8, OpCode::new("INY", 1, 2, AddressingMode::Implied));
//         codes.insert(0xc9, OpCode::new("CMP", 2, 2, AddressingMode::Immediate));
//         codes.insert(0xca, OpCode::new("DEX", 1, 2, AddressingMode::Implied));
//         codes.insert(0xcc, OpCode::new("CPY", 3, 4, AddressingMode::Absolute));
//         codes.insert(0xcd, OpCode::new("CMP", 3, 4, AddressingMode::Absolute));
//         codes.insert(0xce, OpCode::new("DEC", 3, 6, AddressingMode::Absolute));
//         codes.insert(0xd0, OpCode::new("BNE", 2, 2, AddressingMode::Relative)); // extras 2
//         codes.insert(0xd1, OpCode::new("CMP", 2, 5, AddressingMode::IndirectY)); // extras 1
//         codes.insert(0xd5, OpCode::new("CMP", 2, 4, AddressingMode::ZeroPageX));
//         codes.insert(0xd6, OpCode::new("DEC", 2, 6, AddressingMode::ZeroPageX));
//         codes.insert(0xd8, OpCode::new("CLD", 1, 2, AddressingMode::Implied));
//         codes.insert(0xd9, OpCode::new("CMP", 3, 4, AddressingMode::AbsoluteY)); // extras 1
//         codes.insert(0xdd, OpCode::new("CMP", 3, 4, AddressingMode::AbsoluteX)); // extras 1
//         codes.insert(0xde, OpCode::new("DEC", 3, 7, AddressingMode::AbsoluteX));
//         codes.insert(0xe0, OpCode::new("CPX", 2, 2, AddressingMode::Immediate));
//         codes.insert(0xe1, OpCode::new("SBC", 2, 6, AddressingMode::IndirectX));
//         codes.insert(0xe4, OpCode::new("CPX", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0xe5, OpCode::new("SBC", 2, 3, AddressingMode::ZeroPage));
//         codes.insert(0xe6, OpCode::new("INC", 2, 5, AddressingMode::ZeroPage));
//         codes.insert(0xe8, OpCode::new("INX", 1, 2, AddressingMode::Implied));
//         codes.insert(0xe9, OpCode::new("SBC", 2, 2, AddressingMode::Immediate));
//         codes.insert(0xea, OpCode::new("NOP", 1, 2, AddressingMode::Implied));
//         codes.insert(0xec, OpCode::new("CPX", 3, 4, AddressingMode::Absolute));
//         codes.insert(0xed, OpCode::new("SBC", 3, 4, AddressingMode::Absolute));
//         codes.insert(0xee, OpCode::new("INC", 3, 6, AddressingMode::Absolute));
//         codes.insert(0xf0, OpCode::new("BEQ", 2, 2, AddressingMode::Relative)); // extras 2
//         codes.insert(0xf1, OpCode::new("SBC", 2, 5, AddressingMode::IndirectY)); // extras 1
//         codes.insert(0xf5, OpCode::new("SBC", 2, 4, AddressingMode::ZeroPageX));
//         codes.insert(0xf6, OpCode::new("INC", 2, 6, AddressingMode::ZeroPageX));
//         codes.insert(0xf8, OpCode::new("SED", 1, 2, AddressingMode::Implied));
//         codes.insert(0xf9, OpCode::new("SBC", 3, 4, AddressingMode::AbsoluteY)); // extras 1
//         codes.insert(0xfd, OpCode::new("SBC", 3, 4, AddressingMode::AbsoluteX)); // extras 1
//         codes.insert(0xfe, OpCode::new("INC", 3, 7, AddressingMode::AbsoluteX));
//
//         codes
//     };
// }
