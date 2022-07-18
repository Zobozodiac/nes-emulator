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
            address_mode
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

        codes.insert(0xa9, OpCode::new("LDA", 2, 2, AddressingMode::Immediate));
        codes.insert(0x00, OpCode::new("BRK", 1, 7, AddressingMode::NoneAddressing));

        codes
    };
}
