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
            ignored: false,
            break_flag: false,
            decimal: false,
            interrupt: false,
            zero: false,
            carry: false,
        }
    }

    pub fn write_flag(&mut self, flag: Flag, value: bool) {
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

    pub fn read_flag(&mut self, flag: Flag) -> bool {
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_write_flag() {
        let mut status = Status::new();
        status.write_flag(Flag::Negative, true);

        assert_eq!(status.negative, true);
    }

    #[test]
    fn test_read_flag() {
        let mut status = Status::new();
        status.write_flag(Flag::Negative, true);

        let negative = status.read_flag(Flag::Negative);

        assert_eq!(negative, true);
    }

    #[test]
    fn test_get_status_byte() {
        let mut status = Status::new();
        status.write_flag(Flag::Negative, true);
        status.write_flag(Flag::Overflow, true);
        status.write_flag(Flag::Interrupt, true);
        status.write_flag(Flag::Carry, true);

        let status_byte = status.get_status_byte();

        assert_eq!(status_byte, 0b1100_0101);
    }
}
