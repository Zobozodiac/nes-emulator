use crate::cartridge::mapper::Mapper;
use crate::memory::Mem;

pub const PRG_ROM_PAGE_SIZE: usize = 16384;
pub const CHR_ROM_PAGE_SIZE: usize = 8192;

pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: Mapper,
    pub mirroring_type: Mirroring,
}

mod mapper;

impl Cartridge {
    pub fn new(raw: &Vec<u8>) -> Self {
        let control_byte_6 = raw[6];
        let control_byte_7 = raw[7];

        let mapper_type = (control_byte_7 & 0b1111_0000) | (control_byte_6 >> 4);

        let ines_version: u8;

        let ines_byte = (control_byte_7 >> 2) & 0b11;

        match ines_byte {
            0 => ines_version = 1,
            0b10 => ines_version = 2,
            _ => {
                panic!("Unsupported iNES version.")
            }
        }

        let four_screen = (control_byte_6 & 0b1000) != 0;

        let vertical_mirroring = (control_byte_6 & 0b1) != 0;

        let screen_mirroring: Mirroring;

        if four_screen {
            screen_mirroring = Mirroring::FourScreen;
        } else if vertical_mirroring {
            screen_mirroring = Mirroring::Vertical;
        } else {
            screen_mirroring = Mirroring::Horizontal;
        }

        let prg_rom_pages = raw[4] as usize;
        let chr_rom_pages = raw[5] as usize;

        let prg_rom_size = prg_rom_pages * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = chr_rom_pages * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let mapper = match mapper_type {
            0 => Mapper::Mapper000 {
                mirror_bank: prg_rom_pages == 1,
            },
            _ => {
                panic!("Mapper {} not defined", mapper_type)
            }
        };

        Cartridge {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper,
            mirroring_type: screen_mirroring,
        }
    }
}

impl Cartridge {
    pub fn cpu_write(&mut self, address: u16, data: u8) {
        let mapper_address = self.mapper.get_pgr_address(address);
        self.prg_rom[mapper_address as usize] = data;
    }

    pub fn cpu_read(&self, address: u16) -> u8 {
        let mapper_address = self.mapper.get_pgr_address(address);
        self.prg_rom[mapper_address as usize]
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        let mapper_address = self.mapper.get_chr_address(address);
        self.chr_rom[mapper_address as usize] = data;
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        let mapper_address = self.mapper.get_chr_address(address);
        self.chr_rom[mapper_address as usize]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let mut contents: Vec<u8> = vec![
            0x4e,
            0x45,
            0x53,
            0x1a,
            0x02,
            0x02,
            0b0000_0001,
            0b0000_0000,
            0x00,
            0x00,
        ];

        contents.extend([0; 6]);
        contents.extend([0x01; PRG_ROM_PAGE_SIZE * 2]);
        contents.extend([0x02; CHR_ROM_PAGE_SIZE * 2]);

        let cartridge = Cartridge::new(&contents);

        assert_eq!(cartridge.mapper, Mapper::Mapper000 { mirror_bank: true });
        assert_eq!(cartridge.prg_rom, [0x01; PRG_ROM_PAGE_SIZE * 2]);
        assert_eq!(cartridge.chr_rom, [0x02; CHR_ROM_PAGE_SIZE * 2]);
    }
}
