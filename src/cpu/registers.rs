//! CPU Registers.

use std::default::Default;
use std::fmt;
use std::num::Wrapping;
use std::ops::{AddAssign, SubAssign};

use byteorder::{ByteOrder, BigEndian};

use bytes::ByteExt;
use cpu;

bitflags! {
    /// CPU status flags.
    #[derive(Default)]
    pub struct Flags: u8 {
        /// Set if the value of the computation is zero.
        const ZERO          = 0b10000000;

        /// Set if the last operation was a subtraction.
        const SUBTRACT      = 0b01000000;

        /// Set if there was a carry from bit 3 to bit 4.
        const HALF_CARRY    = 0b00100000;

        /// Set if the result did not fit in the register.
        const CARRY         = 0b00010000;
    }
}

/// Two mutable registers treated as a pair (`BC`, `DE`, `HL`).
///
/// Addition and subtraction may be performed on each pair.
#[derive(Debug)]
pub struct RegisterPairMut<'a> {
    hi: &'a mut u8,
    lo: &'a mut u8,
}

impl<'a> RegisterPairMut<'a> {
    /// Returns the register pair as a word.
    pub fn as_word(&self) -> u16 {
        BigEndian::read_u16(&[*self.hi, *self.lo])
    }

    /// Write a word to the register pair.
    ///
    /// # Examples
    ///
    /// ```
    /// use feo_boy::cpu::Registers;
    ///
    /// let mut registers = Registers::new();
    ///
    /// registers.bc_mut().write(0xABCD);
    ///
    /// assert_eq!(registers.bc(), 0xABCD);
    /// assert_eq!(registers.b, 0xAB);
    /// assert_eq!(registers.c, 0xCD);
    /// ```
    pub fn write(&mut self, value: u16) {
        let mut bytes = [0u8; 2];
        BigEndian::write_u16(&mut bytes, value);

        *self.hi = bytes[0];
        *self.lo = bytes[1];
    }
}

impl<'a> AddAssign<u16> for RegisterPairMut<'a> {
    fn add_assign(&mut self, rhs: u16) {
        let pair = Wrapping(BigEndian::read_u16(&[*self.hi, *self.lo])) + Wrapping(rhs);

        self.write(pair.0)
    }
}

impl<'a> SubAssign<u16> for RegisterPairMut<'a> {
    fn sub_assign(&mut self, rhs: u16) {
        let pair = Wrapping(BigEndian::read_u16(&[*self.hi, *self.lo])) - Wrapping(rhs);

        self.write(pair.0)
    }
}

/// The registers.
///
/// # Examples
///
/// Registers are often operated on in pairs. For convenience, assigning addition and subtraction
/// may be performed on each pair.
///
/// ```
/// use feo_boy::cpu::Registers;
///
/// let mut registers = Registers::new();
/// {
///     let mut de = registers.de_mut();
///     de += 1;
/// }
///
/// assert_eq!(registers.de(), 0x0001);
/// assert_eq!(registers.d, 0x00);
/// assert_eq!(registers.e, 0x01);
/// ```
///
/// ```
/// use feo_boy::cpu::Registers;
///
/// let mut registers = Registers::new();
/// {
///     let mut hl = registers.hl_mut();
///     hl.write(0xFFFF);
///     hl -= 0xF;
/// }
///
/// assert_eq!(registers.hl(), 0xFFF0);
/// assert_eq!(registers.h, 0xFF);
/// assert_eq!(registers.l, 0xF0);
/// ```
///
/// To avoid saving the pair to a local variable, you may use the `AddAssign` and `SubAssign`
/// traits directly instead of the operator.
///
/// ```
/// use std::ops::{AddAssign, SubAssign};
/// use feo_boy::cpu::Registers;
///
/// let mut registers = Registers::new();
///
/// registers.bc_mut().add_assign(0xFF);
///
/// assert_eq!(registers.bc(), 0x00FF);
/// ```
#[derive(Debug, Default)]
pub struct Registers {
    /// Accumulator
    pub a: u8,

    /// Status flags
    pub f: Flags,

    // General registers
    pub b: u8,
    pub c: u8,

    pub d: u8,
    pub e: u8,

    pub h: u8,
    pub l: u8,

    /// Program counter
    pub pc: u16,

    /// Stack pointer
    pub sp: u16,
}

impl Registers {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn af(&self) -> u16 {
        BigEndian::read_u16(&[self.a, self.f.bits])
    }

    pub fn af_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.a,
            lo: &mut self.f.bits,
        }
    }

    pub fn bc(&self) -> u16 {
        BigEndian::read_u16(&[self.b, self.c])
    }

    pub fn bc_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.b,
            lo: &mut self.c,
        }
    }

    pub fn de(&self) -> u16 {
        BigEndian::read_u16(&[self.d, self.e])
    }

    pub fn de_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.d,
            lo: &mut self.e,
        }
    }

    pub fn hl(&self) -> u16 {
        BigEndian::read_u16(&[self.h, self.l])
    }

    pub fn hl_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.h,
            lo: &mut self.l,
        }
    }

    /// Bitwise ANDs a byte with the accumulator and sets the flags appropriately.
    pub fn and(&mut self, rhs: u8) {
        self.a &= rhs;

        self.f.remove(SUBTRACT | CARRY);
        self.f.insert(HALF_CARRY);
        self.f.set(ZERO, self.a == 0);
    }

    /// Compares a byte with the accumulator.
    ///
    /// Performs a subtraction with the accumulator without actually setting the accumulator to the
    /// new value. Only the flags are set.
    pub fn cp(&mut self, rhs: u8) {
        let a = self.a;
        self.sub(rhs);
        self.a = a;
    }

    /// Adds a byte to the accumulator and sets the flags appropriately.
    pub fn add(&mut self, rhs: u8) {
        self.f.remove(CARRY);
        self.adc(rhs);
    }

    /// Adds a byte and the value of the carry to the accumulator and sets the flags appropriately.
    pub fn adc(&mut self, rhs: u8) {
        self.f.remove(SUBTRACT);

        let carry_bit = if self.f.contains(CARRY) { 1 } else { 0 };

        let is_half_carry = cpu::is_half_carry_add(self.a, rhs) ||
            cpu::is_half_carry_add(self.a.wrapping_add(rhs), carry_bit);
        self.f.set(HALF_CARRY, is_half_carry);

        let (a, carry) = {
            let (a, rhs_carry) = self.a.overflowing_add(rhs);
            let (a, bit_carry) = a.overflowing_add(carry_bit);

            (a, rhs_carry || bit_carry)
        };
        self.a = a;
        self.f.set(CARRY, carry);

        self.f.set(ZERO, self.a == 0);
    }

    /// Adds a 16-bit number to the HL register pair and sets the flags appropriately.
    pub fn add_hl(&mut self, rhs: u16) {
        let hl = self.hl();

        self.f.remove(SUBTRACT);
        self.f.set(HALF_CARRY, cpu::is_half_carry_add_16(hl, rhs));

        let (a, carry) = hl.overflowing_add(rhs);
        self.f.set(CARRY, carry);
        self.hl_mut().write(a);
    }

    /// Adds a signed byte to the stack pointer, SP, and sets the flags appropriately.
    pub fn add_sp(&mut self, rhs: i8) {
        self.set_sp_r8_flags(rhs);

        let sp = self.sp as i16;
        self.sp = (sp + rhs as i16) as u16;
    }

    /// Places the result of adding a signed byte to the stack pointer, SP, in the register pair
    /// HL, and sets the flags appropriately.
    pub fn ld_hl_sp_r8(&mut self, rhs: i8) {
        self.set_sp_r8_flags(rhs);

        let sp = self.sp as i16;
        self.hl_mut().write((sp + rhs as i16) as u16);
    }

    /// Subtracts a byte from the accumulator and sets the flags appropriately.
    pub fn sub(&mut self, rhs: u8) {
        self.f.remove(CARRY);
        self.sbc(rhs);
    }

    /// Subtracts a byte and the carry flag from the accumulator and sets the flags appropriately.
    pub fn sbc(&mut self, rhs: u8) {
        self.f.insert(SUBTRACT);

        let carry_bit = if self.f.contains(CARRY) { 1 } else { 0 };

        let is_half_carry = cpu::is_half_carry_sub(self.a, rhs) ||
            cpu::is_half_carry_sub(self.a.wrapping_sub(rhs), carry_bit);
        self.f.set(HALF_CARRY, is_half_carry);

        let (a, carry) = {
            let (a, rhs_carry) = self.a.overflowing_sub(rhs);
            let (a, bit_carry) = a.overflowing_sub(carry_bit);
            (a, rhs_carry || bit_carry)
        };
        self.a = a;
        self.f.set(CARRY, carry);

        self.f.set(ZERO, self.a == 0);
    }

    /// Performs an exclusive OR with the accumulator and sets the zero flag appropriately. Unsets
    /// the other flags.
    pub fn xor(&mut self, rhs: u8) {
        self.a ^= rhs;
        self.f = Flags::empty();
        self.f.set(ZERO, self.a == 0);
    }

    /// Peforms an OR with the accumulator and sets the zero flag appropriately. Unsets the other
    /// flags.
    pub fn or(&mut self, rhs: u8) {
        self.a |= rhs;
        self.f = Flags::empty();
        self.f.set(ZERO, self.a == 0);
    }

    /// Performs a decimal adjust (DAA) operation on register A so that the correct representation
    /// of Binary Coded Decimal (BCD) is obtained.
    pub fn daa(&mut self) {
        let mut correction = 0;
        let a = self.a;

        if self.a > 0x99 || self.f.contains(CARRY) {
            correction += 0x60;
            self.f.insert(CARRY);
        }

        if (self.a & 0xf) > 0x9 || self.f.contains(HALF_CARRY) {
            correction += 0x6;
        }

        if self.f.contains(SUBTRACT) {
            self.a = self.a.wrapping_sub(correction);
        } else {
            self.a = self.a.wrapping_add(correction);
        }

        // Set the half carry flag if there has been a carry/borrow between bits 3 and 4
        self.f.set(HALF_CARRY, ((a & 0x10) ^ (self.a & 0x10)) == 0);
        self.f.set(ZERO, self.a == 0);
    }

    /// Rotates register A left one bit, through the carry bit.
    ///
    /// The carry bit is set to the leaving bit on the left, and bit 0 is set to the old value of
    /// the carry bit.
    pub fn rl(&mut self) {
        let old_carry = self.f.contains(CARRY);
        let new_carry = self.a.has_bit_set(7);

        self.f = Flags::empty();
        self.a <<= 1;
        self.a.set_bit(0, old_carry);
        self.f.set(CARRY, new_carry);
    }

    /// Rotates register A left one bit and sets the flags appropriately.
    ///
    /// The leaving bit on the left is copied into the carry bit.
    pub fn rlc(&mut self) {
        self.f = Flags::empty();
        self.a = self.a.rotate_left(1);
        self.f.set(CARRY, self.a.has_bit_set(0));
    }

    /// Sets the flags appropriately for adding a signed byte to the stack pointer, SP. Note that
    /// the carry and half-carry flags are set as if the signed byte is unsigned and is being added
    /// to the low byte of SP.
    fn set_sp_r8_flags(&mut self, rhs: i8) {
        let low_byte = self.sp as u8;

        self.f.remove(ZERO);
        self.f.remove(SUBTRACT);
        self.f.set(
            HALF_CARRY,
            cpu::is_half_carry_add(low_byte, rhs as u8),
        );
        self.f.set(CARRY, low_byte.checked_add(rhs as u8).is_none());
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "A {:#04x}", self.a)?;
        writeln!(f, "B {:#04x}  {:#04x} C", self.b, self.c)?;
        writeln!(f, "D {:#04x}  {:#04x} E", self.d, self.e)?;
        writeln!(f, "H {:#04x}  {:#04x} L", self.h, self.l)?;
        writeln!(f)?;
        writeln!(f, "SP {:#06x}", self.sp)?;
        writeln!(f, "PC {:#06x}", self.pc)?;
        writeln!(f)?;
        writeln!(f, "  ZNHC")?;
        writeln!(f, "F {:08b}", self.f)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::SubAssign;

    use super::{Registers, Flags, ZERO, HALF_CARRY, SUBTRACT, CARRY};

    #[test]
    fn add() {
        let mut reg = Registers::default();
        reg.a = 0xFF;
        reg.add(0xFF);
        assert_eq!(reg.a, 0xFE);
        assert_eq!(reg.f, HALF_CARRY | CARRY);

        let mut reg = Registers::default();
        reg.a = 0xFF;
        reg.add(0x01);
        assert_eq!(reg.a, 0x00);
        assert_eq!(reg.f, ZERO | HALF_CARRY | CARRY);

        let mut reg = Registers::default();
        reg.a = 0x00;
        reg.add(0x01);
        assert_eq!(reg.a, 0x01);
        assert_eq!(reg.f, Flags::empty());

        let mut reg = Registers::default();
        reg.a = 0x3A;
        reg.add(0xC6);
        assert_eq!(reg.a, 0);
        assert_eq!(reg.f, ZERO | HALF_CARRY | CARRY);

        let mut reg = Registers::default();
        reg.a = 0x3C;
        reg.add(0xFF);
        assert_eq!(reg.a, 0x3B);
        assert_eq!(reg.f, HALF_CARRY | CARRY);

        let mut reg = Registers::default();
        reg.a = 0x3C;
        reg.add(0x12);
        assert_eq!(reg.a, 0x4E);
        assert_eq!(reg.f, Flags::empty());
    }

    #[test]
    fn adc() {
        let mut reg = Registers::default();
        reg.a = 0xE1;
        reg.f.insert(CARRY);
        reg.adc(0x0F);
        assert_eq!(reg.a, 0xF1);
        assert_eq!(reg.f, HALF_CARRY);

        let mut reg = Registers::default();
        reg.a = 0xE1;
        reg.f.insert(CARRY);
        reg.adc(0x3B);
        assert_eq!(reg.a, 0x1D);
        assert_eq!(reg.f, CARRY);

        let mut reg = Registers::default();
        reg.a = 0xE1;
        reg.f.insert(CARRY);
        reg.adc(0x1E);
        assert_eq!(reg.a, 0x00);
        assert_eq!(reg.f, ZERO | HALF_CARRY | CARRY);
    }

    #[test]
    fn add_hl() {
        let mut reg = Registers::default();
        reg.hl_mut().write(0xFFFF);
        reg.add_hl(0xFFFF);
        assert_eq!(reg.hl(), 0xFFFE);
        assert_eq!(reg.f, HALF_CARRY | CARRY);

        let mut reg = Registers::default();
        reg.hl_mut().write(0xFFFF);
        reg.add_hl(0x0001);
        assert_eq!(reg.hl(), 0);
        assert_eq!(reg.f, HALF_CARRY | CARRY); // ZERO flag is preserved

        let mut reg = Registers::default();
        reg.hl_mut().write(0x0000);
        reg.add_hl(0x0001);
        assert_eq!(reg.hl(), 1);
        assert_eq!(reg.f, Flags::empty());

        let mut reg = Registers::default();
        reg.hl_mut().write(0x8A23);
        reg.add_hl(0x0605);
        assert_eq!(reg.hl(), 0x9028);
        assert_eq!(reg.f, HALF_CARRY);

        let mut reg = Registers::default();
        reg.hl_mut().write(0x8A23);
        reg.add_hl(0x8A23);
        assert_eq!(reg.hl(), 0x1446);
        assert_eq!(reg.f, HALF_CARRY | CARRY);
    }

    #[test]
    fn add_sp() {
        let mut reg = Registers::default();
        reg.sp = 0xFFF8;
        reg.add_sp(2);
        assert_eq!(reg.sp, 0xFFFA);
        assert_eq!(reg.f, Flags::empty());
    }

    #[test]
    fn ld_hl_sp_r8() {
        let mut reg = Registers::default();
        reg.sp = 0xFFF8;
        reg.ld_hl_sp_r8(2);
        assert_eq!(reg.hl(), 0xFFFA);
        assert_eq!(reg.sp, 0xFFF8);
        assert_eq!(reg.f, Flags::empty());
    }

    #[test]
    fn sub() {
        let mut reg = Registers::default();
        reg.a = 0x3E;
        reg.sub(0x3E);
        assert_eq!(reg.a, 0);
        assert_eq!(reg.f, SUBTRACT | ZERO);

        let mut reg = Registers::default();
        reg.a = 0x3E;
        reg.sub(0x0F);
        assert_eq!(reg.a, 0x2F);
        assert_eq!(reg.f, SUBTRACT | HALF_CARRY);

        let mut reg = Registers::default();
        reg.a = 0x3E;
        reg.sub(0x40);
        assert_eq!(reg.a, 0xFE);
        assert_eq!(reg.f, SUBTRACT | CARRY);

        let mut reg = Registers::default();
        reg.a = 0x00;
        reg.sub(0x01);
        assert_eq!(reg.a, 0xFF);
        assert_eq!(reg.f, SUBTRACT | HALF_CARRY | CARRY);

        let mut reg = Registers::default();
        reg.a = 0xFF;
        reg.sub(0x0F);
        assert_eq!(reg.a, 0xF0);
        assert_eq!(reg.f, SUBTRACT);
    }

    #[test]
    fn sbc() {
        let mut reg = Registers::default();
        reg.a = 0x3B;
        reg.f.insert(CARRY);
        reg.sbc(0x2A);
        assert_eq!(reg.a, 0x10);
        assert_eq!(reg.f, SUBTRACT);

        let mut reg = Registers::default();
        reg.a = 0x3B;
        reg.f.insert(CARRY);
        reg.sbc(0x3A);
        assert_eq!(reg.a, 0);
        assert_eq!(reg.f, SUBTRACT | ZERO);

        let mut reg = Registers::default();
        reg.a = 0x3B;
        reg.f.insert(CARRY);
        reg.sbc(0x4F);
        assert_eq!(reg.a, 0xEB);
        assert_eq!(reg.f, SUBTRACT | HALF_CARRY | CARRY);
    }

    #[test]
    fn cp() {
        let mut reg = Registers::default();
        reg.a = 0x3C;
        reg.cp(0x2F);
        assert_eq!(reg.a, 0x3C);
        assert_eq!(reg.f, SUBTRACT | HALF_CARRY);

        let mut reg = Registers::default();
        reg.a = 0x3C;
        reg.cp(0x3C);
        assert_eq!(reg.a, 0x3C);
        assert_eq!(reg.f, SUBTRACT | ZERO);

        let mut reg = Registers::default();
        reg.a = 0x3C;
        reg.cp(0x40);
        assert_eq!(reg.a, 0x3C);
        assert_eq!(reg.f, SUBTRACT | CARRY);
    }

    #[test]
    fn rlc() {
        let mut reg = Registers::default();
        reg.a = 0x85;
        reg.rlc();

        // This is a different value than the GameBoy programming manual, which specifies `0x0A` as
        // the correct result.
        assert_eq!(reg.a, 0x0B);
        assert_eq!(reg.f, CARRY);
    }

    #[test]
    fn rl() {
        let mut reg = Registers::default();
        reg.a = 0x95;
        reg.f.insert(CARRY);
        reg.rl();
        assert_eq!(reg.a, 0x2B);
        assert_eq!(reg.f, CARRY);
    }

    #[test]
    fn wrap_pair() {
        let mut registers = Registers::default();

        registers.hl_mut().sub_assign(1);

        assert_eq!(registers.h, 0xFF);
        assert_eq!(registers.l, 0xFF);
    }

    #[test]
    fn conversion_equals_immutable() {
        let mut registers = Registers::default();

        registers.hl_mut().write(0xBEEF);

        assert_eq!(0xBEEF, registers.hl_mut().as_word());
        assert_eq!(registers.hl_mut().as_word(), registers.hl());
    }

    // FIXME: daa needs tests
}
