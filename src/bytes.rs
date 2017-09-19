//! Additional functionality for working with bytes.

/// Extension trait providing additional methods for `u8`.
pub trait ByteExt
where
    Self: Sized,
{
    /// Returns whether the byte has its nth bit set.
    fn has_bit_set(&self, n: u8) -> bool;

    /// If `set` is true, flips bit `n` on, and vice-versa.
    fn set_bit(&mut self, n: u8, set: bool);

    /// Returns a tuple containing the result of adding `other` to `self`, and a boolean indicating
    /// whether there was a half carry.
    ///
    /// A half carry is a carry from the low nibble to the high nibble (from bit 3 to bit 4).
    fn half_carry_add(&self, other: Self) -> (Self, bool);

    /// Returns a tuple containing the result of adding `other` to `self`, and a boolean indicating
    /// whether there was a half carry.
    ///
    /// A half carry occurs if the subtraction requires a borrow from the high nibble to the low
    /// nibble (from bit 4 to bit 3).
    fn half_carry_sub(&self, other: Self) -> (Self, bool);
}

impl ByteExt for u8 {
    fn has_bit_set(&self, n: u8) -> bool {
        if n > 7 {
            panic!("bit {} is out of range for u8", n);
        }

        (self & (1 << n)) != 0
    }

    fn set_bit(&mut self, n: u8, set: bool) {
        if n > 7 {
            panic!("bit {} is out of range for u8", n);
        }

        if set {
            *self |= 1 << n;
        } else {
            *self &= !(1 << n);
        }
    }

    fn half_carry_add(&self, other: u8) -> (u8, bool) {
        let is_half_carry = (((self & 0xf).wrapping_add(other & 0xf)) & 0x10) == 0x10;
        (self.wrapping_add(other), is_half_carry)
    }

    fn half_carry_sub(&self, other: u8) -> (u8, bool) {
        let is_half_carry = (self & 0xf) < (other & 0xf);
        (self.wrapping_sub(other), is_half_carry)
    }
}

/// Extension trait providing additional methods for `u16`.
pub trait WordExt {
    /// Returns the low byte (bits 0-7) of the word.
    fn lo(&self) -> u8;

    /// Returns the high byte (bits 8-15) of the word.
    fn hi(&self) -> u8;

    /// Returns a tuple containing the result of adding `other` to `self`, and a boolean indicating
    /// whether there was a half carry.
    ///
    /// A half carry is a carry from bit 11 to 12.
    fn half_carry_add(&self, other: u16) -> (u16, bool);
}

impl WordExt for u16 {
    fn lo(&self) -> u8 {
        *self as u8
    }

    fn hi(&self) -> u8 {
        ((self >> 8) & 0xff_u16) as u8
    }

    /// Returns `true` if the addition of two 16-bit numbers would require a half carry (a carry from
    /// bit 11 to 12, zero-indexed).
    fn half_carry_add(&self, other: u16) -> (u16, bool) {
        let is_half_carry = (((self & 0xfff).wrapping_add(other & 0xfff)) & 0x1000) == 0x1000;
        (self.wrapping_add(other), is_half_carry)
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteExt, WordExt};

    #[test]
    fn has_bit_set() {
        let byte = 0x80;
        assert!(byte.has_bit_set(7));
        assert!(!byte.has_bit_set(0));
    }

    #[test]
    #[should_panic(expected = "bit 8 is out of range for u8")]
    fn has_bit_out_of_range() {
        0xFF.has_bit_set(8);
    }

    #[test]
    fn set_bit() {
        let mut byte = 0xF0;
        byte.set_bit(0, true);
        byte.set_bit(1, false);
        byte.set_bit(3, true);
        byte.set_bit(7, false);
        byte.set_bit(6, true);

        assert_eq!(byte, 0b01111001);
    }

    #[test]
    #[should_panic(expected = "bit 8 is out of range for u8")]
    fn set_bit_out_of_range() {
        let mut byte = 0xFF;
        byte.set_bit(8, true);
    }

    #[test]
    fn high_and_low() {
        assert_eq!(0xabcd.lo(), 0xcd);
        assert_eq!(0xabcd.hi(), 0xab);
        assert_eq!(0xff00.lo(), 0x00);
        assert_eq!(0xff00.hi(), 0xff);
    }

    #[test]
    fn half_carry() {
        assert_eq!(0x0Fu8.half_carry_add(0x01), (0x10, true));
        assert_eq!(0x37u8.half_carry_add(0x44), (0x7B, false));

        assert_eq!(0x0FFFu16.half_carry_add(0x0FFF), (0x1FFE, true));
        assert_eq!(0x0FFFu16.half_carry_add(0x0001), (0x1000, true));
        assert_eq!(0x0000u16.half_carry_add(0x0001), (0x0001, false));

        assert_eq!(0xF0u8.half_carry_sub(0x01), (0xEF, true));
        assert_eq!(0xFFu8.half_carry_sub(0xF0), (0x0F, false));
        assert_eq!(0x3Eu8.half_carry_sub(0x0F), (0x2F, true));
    }
}
