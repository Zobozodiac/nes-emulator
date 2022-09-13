/// A memory object with read and write operations. Stores an array of 0xFFFF bytes.
pub struct Memory {
    storage: [u8; 0xffff + 1],
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            storage: [0; 0xffff + 1],
        }
    }

    /// Read a single byte from the memory
    ///
    /// # Examples
    ///
    /// ```
    /// use nes_emulator::memory::Memory;
    ///
    /// let mut memory = Memory::new();
    /// memory.mem_write(0x0001, 0x12)
    /// ```
    pub fn mem_write(&mut self, address: u16, data: u8) {
        self.storage[address as usize] = data;
    }

    pub fn mem_write_u16(&mut self, address: u16, data: u16) {
        let [lo, hi] = data.to_le_bytes();
        self.mem_write(address, lo);
        self.mem_write(address.wrapping_add(1), hi);
    }

    pub fn mem_read(&self, address: u16) -> u8 {
        self.storage[address as usize]
    }

    pub fn mem_read_u16(&self, address: u16) -> u16 {
        let lo = self.mem_read(address);
        let hi = self.mem_read(address.wrapping_add(1));

        u16::from_le_bytes([lo, hi])
    }

    pub fn load_program(&mut self, program: Vec<u8>) {
        self.storage[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mem_write() {
        let mut memory = Memory::new();
        memory.mem_write(0x0000, 0x12);

        assert_eq!(memory.storage[0x0000], 0x12);
    }

    #[test]
    fn test_mem_write_u16() {
        let mut memory = Memory::new();
        memory.mem_write_u16(0x0000, 0x1234);

        assert_eq!(memory.storage[0x0000], 0x34);
        assert_eq!(memory.storage[0x0001], 0x12);
    }

    #[test]
    fn test_mem_read() {
        let mut memory = Memory::new();
        memory.mem_write(0x0000, 0x12);

        assert_eq!(memory.mem_read(0x0000), 0x12);
    }

    #[test]
    fn test_mem_read_u16() {
        let mut memory = Memory::new();
        memory.mem_write_u16(0x0000, 0x1234);

        assert_eq!(memory.mem_read_u16(0x0000), 0x1234);
    }

    #[test]
    fn test_mem_load_program() {
        let mut memory = Memory::new();
        let program = vec![0xa9];
        memory.load_program(program);

        assert_eq!(memory.mem_read(0x8000), 0xa9);
    }
}
