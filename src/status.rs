pub enum Flag {
    Negative,
    Overflow,
    Ignored,
    Break,
    Decimal,
    Interrupt,
    Zero,
    Carry,
}

#[derive(Debug)]
pub struct Status {
    negative: bool,
    overflow: bool,
    ignored: bool,
    break_flag: bool,
    decimal: bool,
    interrupt: bool,
    zero: bool,
    carry: bool,
}

impl Status {
    pub fn new() -> Self {
        Status {
            negative: false,
            overflow: false,
            ignored: true,
            break_flag: false,
            decimal: false,
            interrupt: true,
            zero: false,
            carry: false,
        }
    }

    pub fn reset(&mut self) {
        self.negative = false;
        self.overflow = false;
        self.ignored = true;
        self.break_flag = false;
        self.decimal = false;
        self.interrupt = true;
        self.zero = false;
        self.carry = false;
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        match flag {
            Flag::Negative => {
                self.negative = value;
            }
            Flag::Overflow => {
                self.overflow = value;
            }
            Flag::Ignored => {
                self.ignored = value;
            }
            Flag::Break => {
                self.break_flag = value;
            }
            Flag::Decimal => {
                self.decimal = value;
            }
            Flag::Interrupt => {
                self.interrupt = value;
            }
            Flag::Zero => {
                self.zero = value;
            }
            Flag::Carry => {
                self.carry = value;
            }
        };
    }

    /// Helper function to set the negative value based on the first bit of a byte.
    pub fn set_negative_flag(&mut self, value: u8) {
        self.set_flag(Flag::Negative, (value & 0b1000_0000) != 0);
    }

    /// Helper function to set the zero flag based on if a byte is zero or not.
    pub fn set_zero_flag(&mut self, value: u8) {
        self.set_flag(Flag::Zero, value == 0);
    }

    pub fn set_decrement_flags(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(0b1111_1111);

        self.set_zero_flag(result);
        self.set_negative_flag(result);

        return result;
    }

    pub fn set_increment_flags(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        self.set_zero_flag(result);
        self.set_negative_flag(result);

        return result;
    }

    pub fn read_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Negative => self.negative,
            Flag::Overflow => self.overflow,
            Flag::Ignored => self.ignored,
            Flag::Break => self.break_flag,
            Flag::Decimal => self.decimal,
            Flag::Interrupt => self.interrupt,
            Flag::Zero => self.zero,
            Flag::Carry => self.carry,
        }
    }

    pub fn get_status_byte(&self) -> u8 {
        let negative = (self.negative as u8) << 7;
        let overflow = (self.overflow as u8) << 6;
        let ignored = (self.ignored as u8) << 5;
        let break_flag = (self.break_flag as u8) << 4;
        let decimal = (self.decimal as u8) << 3;
        let interrupt = (self.interrupt as u8) << 2;
        let zero = (self.zero as u8) << 1;
        let carry = self.carry as u8;

        negative | overflow | ignored | break_flag | decimal | interrupt | zero | carry
    }

    pub fn set_from_byte(&mut self, value: u8) {
        let negative_flag = value & 0b1000_0000;
        let overflow_flag = value & 0b0100_0000;
        let ignored_flag = value & 0b0010_0000;
        let break_flag = value & 0b0001_0000;
        let decimal_flag = value & 0b0000_1000;
        let interrupt_flag = value & 0b0000_0100;
        let zero_flag = value & 0b0000_0010;
        let carry_flag = value & 0b0000_0001;

        self.set_flag(Flag::Negative, negative_flag > 0);
        self.set_flag(Flag::Overflow, overflow_flag > 0);
        self.set_flag(Flag::Ignored, ignored_flag > 0);
        self.set_flag(Flag::Break, break_flag > 0);
        self.set_flag(Flag::Decimal, decimal_flag > 0);
        self.set_flag(Flag::Interrupt, interrupt_flag > 0);
        self.set_flag(Flag::Zero, zero_flag > 0);
        self.set_flag(Flag::Carry, carry_flag > 0);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set_flag() {
        let mut status = Status::new();
        status.set_flag(Flag::Negative, true);

        assert_eq!(status.negative, true);
    }

    #[test]
    fn test_read_flag() {
        let mut status = Status::new();
        status.set_flag(Flag::Negative, true);

        let negative = status.read_flag(Flag::Negative);

        assert_eq!(negative, true);
    }

    #[test]
    fn test_set_negative_flag() {
        let mut status = Status::new();
        status.set_negative_flag(0b1000_0000);

        assert_eq!(status.negative, true);
    }

    #[test]
    fn test_set_zero_flag() {
        let mut status = Status::new();
        status.set_zero_flag(0b0000_0000);

        assert_eq!(status.zero, true);
    }

    #[test]
    fn test_get_status_byte() {
        let mut status = Status::new();
        status.set_flag(Flag::Negative, true);
        status.set_flag(Flag::Overflow, true);
        status.set_flag(Flag::Interrupt, true);
        status.set_flag(Flag::Carry, true);

        let status_byte = status.get_status_byte();

        assert_eq!(status_byte, 0b1100_0101);
    }

    #[test]
    fn test_set_from_byte() {
        let mut status = Status::new();
        status.set_from_byte(0b0000_0011);

        let status_byte = status.get_status_byte();

        assert_eq!(status_byte, 0b0000_0011);
    }
}
