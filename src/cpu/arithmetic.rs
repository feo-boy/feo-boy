//! Implementations of CPU arithmetic.
//!
//! This module should contain free functions that operate on bytes and flags.

use bytes::ByteExt;
use cpu::{self, Flags};

/// Increments by 1 (with overflow).
///
/// # Flags
///
/// | Flag       | Result
/// | ---------- | ---
/// | Zero       | Set if the result is 0.
/// | Subtract   | Reset.
/// | Half-carry | Set if there is a carry from bit 3.
/// | Carry      | Not affected.
pub fn inc(byte: &mut u8, flags: &mut Flags) {
    flags.set(Flags::HALF_CARRY, cpu::is_half_carry_add(*byte, 1));

    *byte = byte.wrapping_add(1);

    flags.set(Flags::ZERO, *byte == 0);
    flags.remove(Flags::SUBTRACT);
}

/// Decrements a byte by 1 (with underflow).
///
/// # Flags
///
/// | Flag       | Result
/// | ---------- | ---
/// | Zero       | Set if the result is 0.
/// | Subtract   | Set.
/// | Half-carry | Set if there is a borrow from bit 4.
/// | Carry      | Not affected.
pub fn dec(byte: &mut u8, flags: &mut Flags) {
    flags.set(Flags::HALF_CARRY, cpu::is_half_carry_sub(*byte, 1));

    *byte = byte.wrapping_sub(1);

    flags.set(Flags::ZERO, *byte == 0);
    flags.insert(Flags::SUBTRACT);
}

/// Rotate left through the carry flag.
///
/// # Flags
///
/// | Flag       | Result
/// | ---------- | ---
/// | Zero       | Set if the result is 0.
/// | Subtract   | Reset.
/// | Half-carry | Reset.
/// | Carry      | Set to the leaving bit on the left.
pub fn rl(byte: &mut u8, flags: &mut Flags) {
    let old_carry = flags.contains(Flags::CARRY);
    let new_carry = byte.has_bit_set(7);

    *flags = Flags::empty();

    *byte <<= 1;
    byte.set_bit(0, old_carry);

    flags.set(Flags::CARRY, new_carry);
    flags.set(Flags::ZERO, *byte == 0);
}

#[cfg(test)]
mod tests {
    use cpu::Flags;

    #[test]
    fn inc() {
        let mut byte = 0xFF;
        let mut flags = Flags::empty();
        super::inc(&mut byte, &mut flags);
        assert_eq!(byte, 0);
        assert_eq!(flags, Flags::ZERO | Flags::HALF_CARRY);

        let mut byte = 0x50;
        let mut flags = Flags::CARRY;
        super::inc(&mut byte, &mut flags);
        assert_eq!(byte, 0x51);
        assert_eq!(flags, Flags::CARRY);
    }

    #[test]
    fn dec() {
        let mut byte = 0x01;
        let mut flags = Flags::empty();
        super::dec(&mut byte, &mut flags);
        assert_eq!(byte, 0);
        assert_eq!(flags, Flags::ZERO | Flags::SUBTRACT);

        let mut byte = 0x00;
        let mut flags = Flags::CARRY;
        super::dec(&mut byte, &mut flags);
        assert_eq!(byte, 0xFF);
        assert_eq!(flags, Flags::SUBTRACT | Flags::HALF_CARRY | Flags::CARRY);
    }

    #[test]
    fn rl() {
        let mut byte = 0x80;
        let mut flags = Flags::empty();
        super::rl(&mut byte, &mut flags);
        assert_eq!(byte, 0);
        assert_eq!(flags, Flags::CARRY | Flags::ZERO);

        let mut byte = 0x95;
        let mut flags = Flags::CARRY;
        super::rl(&mut byte, &mut flags);
        assert_eq!(byte, 0x2B);
        assert_eq!(flags, Flags::CARRY);
    }
}
