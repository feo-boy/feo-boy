//! Implementations of CPU arithmetic.
//!
//! This module should contain free functions that operate on bytes and flags.

use bytes::ByteExt;
use cpu::Flags;

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
    let (sum, is_half_carry) = byte.half_carry_add(1);
    *byte = sum;

    flags.set(Flags::ZERO, *byte == 0);
    flags.remove(Flags::SUBTRACT);
    flags.set(Flags::HALF_CARRY, is_half_carry);
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
    let (difference, is_half_carry) = byte.half_carry_sub(1);
    *byte = difference;

    flags.set(Flags::ZERO, *byte == 0);
    flags.insert(Flags::SUBTRACT);
    flags.set(Flags::HALF_CARRY, is_half_carry);
}

/// Rotate left, copying into the carry.
///
/// # Flags
///
/// | Flag       | Result
/// | ---------  | ---
/// | Zero       | Set if the result is 0.
/// | Subtract   | Reset.
/// | Half-carry | Reset.
/// | Carry      | Set to the old value of bit 7.
pub fn rlc(byte: &mut u8, flags: &mut Flags) {
    *byte = byte.rotate_left(1);

    flags.set(Flags::ZERO, *byte == 0);
    flags.remove(Flags::SUBTRACT | Flags::HALF_CARRY);
    flags.set(Flags::CARRY, byte.has_bit_set(0));
}

/// Rotate right, copying into the carry.
///
/// # Flags
///
/// | Flag       | Result
/// | ---------  | ---
/// | Zero       | Set if the result is 0.
/// | Subtract   | Reset.
/// | Half-carry | Reset.
/// | Carry      | Set to the old value of bit 0.
pub fn rrc(byte: &mut u8, flags: &mut Flags) {
    *byte = byte.rotate_right(1);

    flags.set(Flags::ZERO, *byte == 0);
    flags.remove(Flags::SUBTRACT | Flags::HALF_CARRY);
    flags.set(Flags::CARRY, byte.has_bit_set(7));
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

/// Rotate right through the carry flag.
///
/// # Flags
///
/// | Flag       | Result
/// | ---------- | ---
/// | Zero       | Set if the result is 0.
/// | Subtract   | Reset.
/// | Half-carry | Reset.
/// | Carry      | Set to the leaving bit on the right.
pub fn rr(byte: &mut u8, flags: &mut Flags) {
    let old_carry = flags.contains(Flags::CARRY);
    let new_carry = byte.has_bit_set(0);

    *flags = Flags::empty();

    *byte >>= 1;
    byte.set_bit(7, old_carry);

    flags.set(Flags::CARRY, new_carry);
    flags.set(Flags::ZERO, *byte == 0);
}

/// Tests bit b in register r
///
/// # Flags
///
/// | Flag       | Result
/// | ---------- | ---
/// | Zero       | Set if bit b of register r is 0
/// | Subtract   | Reset.
/// | Half-carry | Set.
/// | Carry      | Not affected.
pub fn bit(byte: u8, b: u8, flags: &mut Flags) {
    flags.set(Flags::ZERO, !byte.has_bit_set(b));
    flags.remove(Flags::SUBTRACT);
    flags.insert(Flags::HALF_CARRY);
}

pub fn sla(byte: &mut u8, flags: &mut Flags) {
    flags.set(Flags::CARRY, !byte.has_bit_set(7));
    *byte <<= 1;
    flags.set(Flags::ZERO, *byte == 0);
}

pub fn sra(byte: &mut u8, flags: &mut Flags) {
    flags.set(Flags::CARRY, !byte.has_bit_set(0));
    *byte = (*byte as i8 >> 1) as u8;
    flags.set(Flags::ZERO, *byte == 0);
}

pub fn srl(byte: &mut u8, flags: &mut Flags) {
    flags.set(Flags::CARRY, !byte.has_bit_set(0));
    *byte >>= 1;
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
    fn rlc() {
        let mut byte = 0x85;
        let mut flags = Flags::empty();
        super::rlc(&mut byte, &mut flags);
        assert_eq!(byte, 0x0B);
        assert_eq!(flags, Flags::CARRY);

        let mut byte = 0x00;
        let mut flags = Flags::empty();
        super::rlc(&mut byte, &mut flags);
        assert_eq!(byte, 0x00);
        assert_eq!(flags, Flags::ZERO);
    }

    #[test]
    fn rrc() {
        let mut byte = 0x01;
        let mut flags = Flags::empty();
        super::rrc(&mut byte, &mut flags);
        assert_eq!(byte, 0x80);
        assert_eq!(flags, Flags::CARRY);

        let mut byte = 0x00;
        let mut flags = Flags::empty();
        super::rrc(&mut byte, &mut flags);
        assert_eq!(byte, 0x00);
        assert_eq!(flags, Flags::ZERO);

        let mut byte = 0x11;
        let mut flags = Flags::empty();
        super::rrc(&mut byte, &mut flags);
        assert_eq!(byte, 0x88);
        assert_eq!(flags, Flags::CARRY);

        let mut byte = 0x10;
        let mut flags = Flags::empty();
        super::rrc(&mut byte, &mut flags);
        assert_eq!(byte, 0x08);
        assert_eq!(flags, Flags::empty());
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

    #[test]
    fn rr() {
        let mut byte = 0x01;
        let mut flags = Flags::empty();
        super::rr(&mut byte, &mut flags);
        assert_eq!(byte, 0x00);
        assert_eq!(flags, Flags::ZERO | Flags::CARRY);

        let mut byte = 0x8A;
        let mut flags = Flags::empty();
        super::rr(&mut byte, &mut flags);
        assert_eq!(byte, 0x45);
        assert_eq!(flags, Flags::empty());

        let mut byte = 0x11;
        let mut flags = Flags::empty();
        super::rr(&mut byte, &mut flags);
        assert_eq!(byte, 0x08);
        assert_eq!(flags, Flags::CARRY);

        let mut byte = 0x10;
        let mut flags = Flags::CARRY;
        super::rr(&mut byte, &mut flags);
        assert_eq!(byte, 0x88);
        assert_eq!(flags, Flags::empty());
    }
}
