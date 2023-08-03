use crate::cpu::CPU;
use crate::errors::NesError;
use crate::memory::Mem;

impl CPU {
    pub fn get_stack_address(&self) -> u16 {
        u16::from_le_bytes([self.stack_pointer, 0x01])
    }

    pub fn push_to_stack(&mut self, data: u8) -> Result<(), NesError> {
        let stack_address = self.get_stack_address();

        self.bus.mem_write(stack_address, data)?;
        self.stack_pointer = self.stack_pointer - 1;
        Ok(())
    }

    pub fn push_to_stack_u16(&mut self, data: u16) -> Result<(), NesError> {
        let [lo, hi] = u16::to_le_bytes(data);

        self.push_to_stack(hi)?;
        self.push_to_stack(lo)?;
        Ok(())
    }

    pub fn pull_from_stack(&mut self) -> Result<u8, NesError> {
        self.stack_pointer = self.stack_pointer + 1;
        let stack_address = self.get_stack_address();

        Ok(self.bus.mem_read(stack_address)?)
    }

    pub fn pull_from_stack_u16(&mut self) -> Result<u16, NesError> {
        let lo = self.pull_from_stack()?;
        let hi = self.pull_from_stack()?;

        Ok(u16::from_le_bytes([lo, hi]))
    }
}
