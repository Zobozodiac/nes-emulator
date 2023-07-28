use std::cell::RefCell;

use crate::cartridge::Cartridge;
use crate::memory::{Mem, Storage};

const CPU_RAM_START: u16 = 0x0000;
const CPU_MEMORY_END: u16 = 0x1fff;
const PPU_RAM_START: u16 = 0x2000;
const PPU_MEMORY_END: u16 = 0x3fff;

pub struct Bus {
    cpu_ram: Storage,
    cartridge: Cartridge,
}

impl Mem for Bus {
    fn mem_write(&mut self, address: u16, data: u8) {
        match address {
            CPU_RAM_START..=CPU_MEMORY_END => {
                let address = address & 0b00000111_11111111;
                self.cpu_ram.mem_write(address, data);
            }
            PPU_RAM_START..=PPU_MEMORY_END => {
                let address = address & 0b00000000_00000111;
                panic!("PPU not implemented yet.");
            }
            _ => {
                panic!("Writing to address out of range {}", address);
            }
        }
    }

    fn mem_read(&self, address: u16) -> u8 {
        match address {
            CPU_RAM_START..=CPU_MEMORY_END => {
                let address = address & 0b00000111_11111111;
                self.cpu_ram.mem_read(address)
            }
            PPU_RAM_START..=PPU_MEMORY_END => {
                let address = address & 0b00000000_00000111;
                panic!("PPU not implemented yet.");
            }
            _ => {
                panic!("Reading to address out of range {}", address);
            }
        }
    }
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            cpu_ram: Storage::new(2048),
            cartridge,
        }
    }
}
