use crate::errors::NesError;

/// A memory object with read and write operations. Stores an array of 0xFFFF bytes.

pub trait Mem {
    fn mem_write(&mut self, address: u16, data: u8) -> Result<(), NesError>;

    fn mem_read(&self, address: u16) -> Result<u8, NesError>;

    fn mem_write_u16(&mut self, address: u16, data: u16) -> Result<(), NesError> {
        let [lo, hi] = data.to_le_bytes();
        self.mem_write(address, lo)?;
        self.mem_write(address.wrapping_add(1), hi)?;
        Ok(())
    }

    fn mem_read_u16(&self, address: u16) -> Result<u16, NesError> {
        let lo = self.mem_read(address)?;
        let hi = self.mem_read(address.wrapping_add(1))?;

        Ok(u16::from_le_bytes([lo, hi]))
    }

    fn mem_read_u16_wrapping_boundary(&self, address: u16) -> Result<u16, NesError> {
        let lo = self.mem_read(address)?;

        let hi_address = address.wrapping_add(1);

        if (hi_address & 0xff00) == (address & 0xff00) {
            Ok(u16::from_le_bytes([lo, self.mem_read(hi_address)?]))
        } else {
            Ok(u16::from_le_bytes([lo, self.mem_read(address & 0xff00)?]))
        }
    }
}

pub struct RAM {
    storage: Vec<u8>,
}

impl Mem for RAM {
    fn mem_write(&mut self, address: u16, data: u8) -> Result<(), NesError> {
        self.storage[address as usize] = data;
        Ok(())
    }

    fn mem_read(&self, address: u16) -> Result<u8, NesError> {
        Ok(self.storage[address as usize])
    }
}

impl RAM {
    pub fn new(size: usize) -> Self {
        RAM {
            storage: vec![0; size],
        }
    }

    // pub fn print_page(&self, page: u8) {
    //     for i in 0..(0xf + 1) {
    //         let i = (i << 4) as u8;
    //
    //         let start_address = u16::from_le_bytes([i, page]);
    //
    //         self.print_row(start_address);
    //     }
    // }

    // pub fn print_row(&self, start_address: u16) {
    //     let mut print_string = format!("{:04x}:", start_address);
    //
    //     for i in 0..(0xf + 1) {
    //         let i = i as u16;
    //
    //         let address = start_address + i;
    //
    //         let value = self.mem_read(address)?;
    //
    //         print_string.push_str(&format!(" {:02x}", value));
    //     }
    //
    //     println!("{}", print_string);
    // }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//
//     #[test]
//     fn test_mem_write() {
//         let mut memory = RAM::new(0xffff);
//         memory.mem_write(0x0000, 0x12);
//
//         assert_eq!(memory.storage[0x0000], 0x12);
//     }
//
//     #[test]
//     fn test_mem_write_u16() {
//         let mut memory = RAM::new(0xffff);
//         memory.mem_write_u16(0x0000, 0x1234);
//
//         assert_eq!(memory.storage[0x0000], 0x34);
//         assert_eq!(memory.storage[0x0001], 0x12);
//     }
//
//     #[test]
//     fn test_mem_read() {
//         let mut memory = RAM::new(0xffff);
//         memory.mem_write(0x0000, 0x12);
//
//         assert_eq!(memory.mem_read(0x0000), 0x12);
//     }
//
//     #[test]
//     fn test_mem_read_u16() {
//         let mut memory = RAM::new(0xffff);
//         memory.mem_write_u16(0x0000, 0x1234);
//
//         assert_eq!(memory.mem_read_u16(0x0000), 0x1234);
//     }
//
//     #[test]
//     fn test_print_row() {
//         let mut memory = RAM::new(0xffff);
//         memory.mem_write(0x0120, 0x0);
//         memory.mem_write(0x0121, 0x1);
//         memory.mem_write(0x0122, 0x2);
//         memory.mem_write(0x0123, 0x3);
//         memory.mem_write(0x0124, 0x4);
//         memory.mem_write(0x0125, 0x5);
//         memory.mem_write(0x0126, 0x6);
//         memory.mem_write(0x0127, 0x7);
//         memory.mem_write(0x0128, 0x8);
//         memory.mem_write(0x0129, 0x9);
//         memory.mem_write(0x012a, 0xa);
//         memory.mem_write(0x012b, 0xb);
//         memory.mem_write(0x012c, 0xc);
//         memory.mem_write(0x012d, 0xd);
//         memory.mem_write(0x012e, 0xe);
//         memory.mem_write(0x012f, 0xf);
//
//         memory.print_row(0x0120);
//     }
//
//     #[test]
//     fn test_print_page() {
//         let mut memory = RAM::new(0xffff);
//         memory.mem_write(0x0120, 0x0);
//         memory.mem_write(0x0121, 0x1);
//         memory.mem_write(0x0122, 0x2);
//         memory.mem_write(0x0123, 0x3);
//         memory.mem_write(0x0124, 0x4);
//         memory.mem_write(0x0125, 0x5);
//         memory.mem_write(0x0126, 0x6);
//         memory.mem_write(0x0127, 0x7);
//         memory.mem_write(0x0128, 0x8);
//         memory.mem_write(0x0129, 0x9);
//         memory.mem_write(0x012a, 0xa);
//         memory.mem_write(0x012b, 0xb);
//         memory.mem_write(0x012c, 0xc);
//         memory.mem_write(0x012d, 0xd);
//         memory.mem_write(0x012e, 0xe);
//         memory.mem_write(0x012f, 0xf);
//
//         memory.print_page(0x01);
//     }
// }
