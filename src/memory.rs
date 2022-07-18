pub struct Memory {
    storage: [u8; 0xFFFF],
}

impl Memory {
    pub fn new(storage: [u8; 0xFFFF]) -> Self {
        Memory {
            storage
        }
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.storage[addr as usize] = data;
    }

    pub fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let [lo, hi] = data.to_le_bytes();
        self.mem_write(pos, lo);
        self.mem_write(pos.wrapping_add(1), hi);
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.storage[addr as usize]
    }

    pub fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos);
        let hi = self.mem_read(pos.wrapping_add(1));

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
        let mut memory = Memory::new([0; 0xFFFF]);
        memory.mem_write(0x0000, 0x12);

        assert_eq!(memory.storage[0x0000], 0x12);
    }

    #[test]
    fn test_mem_write_u16() {
        let mut memory = Memory::new([0; 0xFFFF]);
        memory.mem_write_u16(0x0000, 0x1234);

        assert_eq!(memory.storage[0x0000], 0x34);
        assert_eq!(memory.storage[0x0001], 0x12);
    }

    #[test]
    fn test_mem_read() {
        let mut memory = Memory::new([0; 0xFFFF]);
        memory.mem_write(0x0000, 0x12);

        assert_eq!(memory.mem_read(0x0000), 0x12);
    }

    #[test]
    fn test_mem_read_u16() {
        let mut memory = Memory::new([0; 0xFFFF]);
        memory.mem_write_u16(0x0000, 0x1234);

        assert_eq!(memory.mem_read_u16(0x0000), 0x1234);
    }

    #[test]
    fn test_mem_load_program() {
        let mut memory = Memory::new([0; 0xFFFF]);
        let program = vec![0xa9];
        memory.load_program(program);

        assert_eq!(memory.mem_read(0x8000), 0xa9);
    }
}