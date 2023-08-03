use crate::cartridge::Cartridge;
use crate::errors::NesError;
use crate::memory::{Mem, RAM};

const CPU_RAM_START: u16 = 0x0000;
const CPU_MEMORY_END: u16 = 0x1fff;
const PPU_RAM_START: u16 = 0x2000;
const PPU_MEMORY_END: u16 = 0x3fff;
const CARTRIDGE_ROM_START: u16 = 0x8000;
const CARTRIDGE_ROM_END: u16 = 0xffff;

pub struct CpuBus {
    cpu_ram: RAM,
    cartridge: Cartridge,
}

impl Mem for CpuBus {
    fn mem_write(&mut self, address: u16, data: u8) -> Result<(), NesError> {
        match address {
            CPU_RAM_START..=CPU_MEMORY_END => {
                let address = address & 0b00000111_11111111;
                self.cpu_ram.mem_write(address, data)?;
                Ok(())
            }
            PPU_RAM_START..=PPU_MEMORY_END => {
                let address = address & 0b00000000_00000111;
                Err(NesError::new("PPU not implemented yet."))
            }
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => {
                Err(NesError::new("Writing to cartridge ROM"))
            }
            _ => Err(NesError::new(&format!(
                "Writing to address out of range {}",
                address
            ))),
        }
    }

    fn mem_read(&self, address: u16) -> Result<u8, NesError> {
        match address {
            CPU_RAM_START..=CPU_MEMORY_END => {
                let address = address & 0b00000111_11111111;
                Ok(self.cpu_ram.mem_read(address)?)
            }
            PPU_RAM_START..=PPU_MEMORY_END => {
                let address = address & 0b00000000_00000111;
                Err(NesError::new("PPU not implemented yet."))
            }
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => Ok(self.cartridge.cpu_read(address)),
            _ => Err(NesError::new(&format!(
                "Reading to address out of range {}",
                address
            ))),
        }
    }
}

impl CpuBus {
    pub fn new(cartridge: Cartridge) -> Self {
        CpuBus {
            cpu_ram: RAM::new(2048),
            cartridge,
        }
    }
}
