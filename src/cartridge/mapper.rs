pub trait Mapping {
    fn get_chr_address(address: &u8) -> u8;

    fn get_pgr_address(address: &u8) -> u8;
}

#[derive(PartialEq, Debug)]
pub enum Mapper {
    Mapper000 { mirror_bank: bool },
}

impl Mapper {
    pub fn get_pgr_address(&self, address: u16) -> u16 {
        match self {
            Mapper::Mapper000 { mirror_bank } => {
                if *mirror_bank {
                    address & 0x3fff
                } else {
                    address & 0x7fff
                }
            }
        }
    }

    pub fn get_chr_address(&self, address: u16) -> u16 {
        match self {
            Mapper::Mapper000 { mirror_bank: _ } => address,
        }
    }
}
