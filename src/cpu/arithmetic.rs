//! Implementations of CPU arithmetic.
//!
//! This module should contain free functions that operate on bytes and flags.

use bytes::ByteExt;
use cpu::Flags;

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
