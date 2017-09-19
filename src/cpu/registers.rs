//! CPU Registers.

use std::default::Default;
use std::fmt;
use std::num::Wrapping;
use std::ops::{AddAssign, SubAssign};

use byteorder::{ByteOrder, BigEndian};

use bytes::{ByteExt, WordExt};

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

/// The registers. High speed data storage for the CPU.
///
/// 8-bit registers (`A`, `F`, `B`, `C`, `D`, `E`, `H`, and `L`), as well as the stack pointer and
/// the program counter may be accessed by their individual fields.
///
/// This struct also provides a number of methods for performing instructions. Many instructions
/// modify register `A` and set the flags as necessary.
///
/// Note that the flag register contains some additional methods to assist in setting flags, and is
/// not actually a `u8`. To access it as a `u8`, you must use the `bits` method. See the [`Flags`]
/// struct for more detail.
///
/// ```
/// use feo_boy::cpu::{Registers, Flags};
///
/// let mut registers = Registers::new();
/// registers.f.insert(Flags::ZERO | Flags::HALF_CARRY);
/// assert_eq!(registers.f.bits(), 0b10100000);
/// ```
///
/// In many instructions, registers may be accessed as a word pair (16 bits). The left register is
/// the high byte, and the right register is the low byte. The `Registers` struct provides methods
/// for accessing these pairs mutably and immutably.
///
/// ```
/// use feo_boy::cpu::Registers;
///
/// let mut registers = Registers::new();
/// registers.b = 0xAB;
/// registers.c = 0xCD;
/// assert_eq!(registers.bc(), 0xABCD);
/// ```
///
/// For convenience, assigning addition and subtraction may be performed on each pair.
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
///
/// [`Flags`]: ./struct.Flags.html
#[derive(Debug, Default)]
pub struct Registers {
    /// Accumulator.
    pub a: u8,

    /// Status flags.
    pub f: Flags,

    /// General purpose register `B`.
    pub b: u8,

    /// General purpose register `C`.
    pub c: u8,

    /// General purpose register `D`.
    pub d: u8,

    /// General purpose register `E`.
    pub e: u8,

    /// General purpose register `H`.
    pub h: u8,

    /// General purpose register `L`.
    pub l: u8,

    /// Program counter.
    pub pc: u16,

    /// Stack pointer.
    pub sp: u16,
}

impl Registers {
    /// Create a new register set.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns register pair `AF`.
    pub fn af(&self) -> u16 {
        BigEndian::read_u16(&[self.a, self.f.bits])
    }

    /// Returns a mutable reference to register pair `AF`.
    pub fn af_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.a,
            lo: &mut self.f.bits,
        }
    }

    /// Returns register pair `BC`.
    pub fn bc(&self) -> u16 {
        BigEndian::read_u16(&[self.b, self.c])
    }

    /// Returns a mutable reference to register pair `BC`.
    pub fn bc_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.b,
            lo: &mut self.c,
        }
    }

    /// Returns register pair `DE`.
    pub fn de(&self) -> u16 {
        BigEndian::read_u16(&[self.d, self.e])
    }

    /// Returns a mutable reference to register pair `DE`.
    pub fn de_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.d,
            lo: &mut self.e,
        }
    }

    /// Returns register pair `HL`.
    pub fn hl(&self) -> u16 {
        BigEndian::read_u16(&[self.h, self.l])
    }

    /// Returns a mutable reference to register pair `HL`.
    pub fn hl_mut(&mut self) -> RegisterPairMut {
        RegisterPairMut {
            hi: &mut self.h,
            lo: &mut self.l,
        }
    }

    /// Bitwise ANDs a byte with the accumulator and sets the flags appropriately.
    pub fn and(&mut self, rhs: u8) {
        self.a &= rhs;

        self.f.remove(Flags::SUBTRACT | Flags::CARRY);
        self.f.insert(Flags::HALF_CARRY);
        self.f.set(Flags::ZERO, self.a == 0);
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
        self.f.remove(Flags::CARRY);
        self.adc(rhs);
    }

    /// Adds a byte and the value of the carry to the accumulator and sets the flags appropriately.
    pub fn adc(&mut self, rhs: u8) {
        let carry_bit = self.f.contains(Flags::CARRY) as u8;

        let (sum, is_half_carry_rhs) = self.a.half_carry_add(rhs);
        let (_, is_half_carry_bit) = sum.half_carry_add(carry_bit);

        let (sum, is_carry_rhs) = self.a.overflowing_add(rhs);
        let (sum, is_carry_bit) = sum.overflowing_add(carry_bit);

        self.a = sum;

        self.f.set(Flags::ZERO, self.a == 0);
        self.f.remove(Flags::SUBTRACT);
        self.f.set(
            Flags::HALF_CARRY,
            is_half_carry_rhs || is_half_carry_bit,
        );
        self.f.set(Flags::CARRY, is_carry_rhs || is_carry_bit);
    }

    /// Adds a 16-bit number to the HL register pair and sets the flags appropriately.
    pub fn add_hl(&mut self, rhs: u16) {
        let hl = self.hl();

        let (sum, is_carry) = hl.overflowing_add(rhs);
        let (_, is_half_carry) = hl.half_carry_add(rhs);
        self.hl_mut().write(sum);

        self.f.remove(Flags::SUBTRACT);
        self.f.set(Flags::HALF_CARRY, is_half_carry);
        self.f.set(Flags::CARRY, is_carry);
    }

    /// Adds a signed byte to the stack pointer, SP, and sets the flags appropriately.
    pub fn add_sp(&mut self, rhs: i8) {
        self.set_sp_r8_flags(rhs);

        let sp = self.sp as i16;
        self.sp = (sp + i16::from(rhs)) as u16;
    }

    /// Places the result of adding a signed byte to the stack pointer, SP, in the register pair
    /// HL, and sets the flags appropriately.
    pub fn ld_hl_sp_r8(&mut self, rhs: i8) {
        self.set_sp_r8_flags(rhs);

        let sp = self.sp as i16;
        self.hl_mut().write((sp + i16::from(rhs)) as u16);
    }

    /// Subtracts a byte from the accumulator and sets the flags appropriately.
    pub fn sub(&mut self, rhs: u8) {
        self.f.remove(Flags::CARRY);
        self.sbc(rhs);
    }

    /// Subtracts a byte and the carry flag from the accumulator and sets the flags appropriately.
    pub fn sbc(&mut self, rhs: u8) {
        let carry_bit = self.f.contains(Flags::CARRY) as u8;

        let (difference, is_half_carry_rhs) = self.a.half_carry_sub(rhs);
        let (_, is_half_carry_bit) = difference.half_carry_sub(carry_bit);

        let (difference, is_carry_rhs) = self.a.overflowing_sub(rhs);
        let (difference, is_carry_bit) = difference.overflowing_sub(carry_bit);

        self.a = difference;

        self.f.set(Flags::ZERO, self.a == 0);
        self.f.insert(Flags::SUBTRACT);
        self.f.set(
            Flags::HALF_CARRY,
            is_half_carry_rhs || is_half_carry_bit,
        );
        self.f.set(Flags::CARRY, is_carry_rhs || is_carry_bit);
    }

    /// Performs an exclusive OR with the accumulator and sets the zero flag appropriately. Unsets
    /// the other flags.
    pub fn xor(&mut self, rhs: u8) {
        self.a ^= rhs;
        self.f = Flags::empty();
        self.f.set(Flags::ZERO, self.a == 0);
    }

    /// Peforms an OR with the accumulator and sets the zero flag appropriately. Unsets the other
    /// flags.
    pub fn or(&mut self, rhs: u8) {
        self.a |= rhs;
        self.f = Flags::empty();
        self.f.set(Flags::ZERO, self.a == 0);
    }

    /// Performs a decimal adjust (DAA) operation on register A so that the correct representation
    /// of Binary Coded Decimal (BCD) is obtained.
    pub fn daa(&mut self) {
        let mut correction = 0;
        let a = self.a;

        if self.a > 0x99 || self.f.contains(Flags::CARRY) {
            correction += 0x60;
            self.f.insert(Flags::CARRY);
        }

        if (self.a & 0xf) > 0x9 || self.f.contains(Flags::HALF_CARRY) {
            correction += 0x6;
        }

        if self.f.contains(Flags::SUBTRACT) {
            self.a = self.a.wrapping_sub(correction);
        } else {
            self.a = self.a.wrapping_add(correction);
        }

        // Set the half carry flag if there has been a carry/borrow between bits 3 and 4
        self.f.set(
            Flags::HALF_CARRY,
            ((a & 0x10) ^ (self.a & 0x10)) == 0,
        );
        self.f.set(Flags::ZERO, self.a == 0);
    }

    /// Rotates register A left one bit and sets the flags appropriately.
    ///
    /// The leaving bit on the left is copied into the carry bit.
    pub fn rlc(&mut self) {
        self.f = Flags::empty();
        self.a = self.a.rotate_left(1);
        self.f.set(Flags::CARRY, self.a.has_bit_set(0));
    }

    /// Rotates register A right one bit, through the carry bit.
    ///
    /// The carry bit is set to the leaving bit on the right, and bit 7 is set to the old value of
    /// the carry bit.
    pub fn rr(&mut self) {
        let old_carry = self.f.contains(Flags::CARRY);
        let new_carry = self.a.has_bit_set(0);

        self.f = Flags::empty();
        self.a >>= 1;
        self.a.set_bit(7, old_carry);
        self.f.set(Flags::CARRY, new_carry);
    }

    /// Rotates register A right one bit and sets the flags appropriately.
    ///
    /// The leaving bit on the right is copied into the carry bit. Other flags are reset.
    pub fn rrc(&mut self) {
        self.f = Flags::empty();
        self.f.set(Flags::CARRY, self.a.has_bit_set(0));
        self.a = self.a.rotate_right(1);
    }

    /// Inverts all bits in `A` and sets the flags appropriately.
    pub fn cpl(&mut self) {
        self.a = !self.a;
        self.f.insert(Flags::SUBTRACT | Flags::HALF_CARRY);
    }

    /// Complements the carry flag and resets all other flags.
    pub fn ccf(&mut self) {
        let old_carry = self.f.contains(Flags::CARRY);
        self.f.remove(Flags::SUBTRACT | Flags::HALF_CARRY);
        self.f.set(Flags::CARRY, !old_carry);
    }

    /// Sets the flags appropriately for adding a signed byte to the stack pointer, SP. Note that
    /// the carry and half-carry flags are set as if the signed byte is unsigned and is being added
    /// to the low byte of SP.
    fn set_sp_r8_flags(&mut self, rhs: i8) {
        let low_byte = self.sp as u8;

        self.f = Flags::empty();
        self.f.set(
            Flags::HALF_CARRY,
            low_byte.half_carry_add(rhs as u8).1,
        );
        self.f.set(
            Flags::CARRY,
            low_byte.checked_add(rhs as u8).is_none(),
        );
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

    use super::{Registers, Flags};

    #[test]
    fn add() {
        let mut reg = Registers::default();
        reg.a = 0xFF;
        reg.add(0xFF);
        assert_eq!(reg.a, 0xFE);
        assert_eq!(reg.f, Flags::HALF_CARRY | Flags::CARRY);

        let mut reg = Registers::default();
        reg.a = 0xFF;
        reg.add(0x01);
        assert_eq!(reg.a, 0x00);
        assert_eq!(reg.f, Flags::ZERO | Flags::HALF_CARRY | Flags::CARRY);

        let mut reg = Registers::default();
        reg.a = 0x00;
        reg.add(0x01);
        assert_eq!(reg.a, 0x01);
        assert_eq!(reg.f, Flags::empty());

        let mut reg = Registers::default();
        reg.a = 0x3A;
        reg.add(0xC6);
        assert_eq!(reg.a, 0);
        assert_eq!(reg.f, Flags::ZERO | Flags::HALF_CARRY | Flags::CARRY);

        let mut reg = Registers::default();
        reg.a = 0x3C;
        reg.add(0xFF);
        assert_eq!(reg.a, 0x3B);
        assert_eq!(reg.f, Flags::HALF_CARRY | Flags::CARRY);

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
        reg.f.insert(Flags::CARRY);
        reg.adc(0x0F);
        assert_eq!(reg.a, 0xF1);
        assert_eq!(reg.f, Flags::HALF_CARRY);

        let mut reg = Registers::default();
        reg.a = 0xE1;
        reg.f.insert(Flags::CARRY);
        reg.adc(0x3B);
        assert_eq!(reg.a, 0x1D);
        assert_eq!(reg.f, Flags::CARRY);

        let mut reg = Registers::default();
        reg.a = 0xE1;
        reg.f.insert(Flags::CARRY);
        reg.adc(0x1E);
        assert_eq!(reg.a, 0x00);
        assert_eq!(reg.f, Flags::ZERO | Flags::HALF_CARRY | Flags::CARRY);
    }

    #[test]
    fn add_hl() {
        let mut reg = Registers::default();
        reg.hl_mut().write(0xFFFF);
        reg.add_hl(0xFFFF);
        assert_eq!(reg.hl(), 0xFFFE);
        assert_eq!(reg.f, Flags::HALF_CARRY | Flags::CARRY);

        let mut reg = Registers::default();
        reg.hl_mut().write(0xFFFF);
        reg.add_hl(0x0001);
        assert_eq!(reg.hl(), 0);
        assert_eq!(reg.f, Flags::HALF_CARRY | Flags::CARRY); // zero flag is preserved

        let mut reg = Registers::default();
        reg.hl_mut().write(0x0000);
        reg.add_hl(0x0001);
        assert_eq!(reg.hl(), 1);
        assert_eq!(reg.f, Flags::empty());

        let mut reg = Registers::default();
        reg.hl_mut().write(0x8A23);
        reg.add_hl(0x0605);
        assert_eq!(reg.hl(), 0x9028);
        assert_eq!(reg.f, Flags::HALF_CARRY);

        let mut reg = Registers::default();
        reg.hl_mut().write(0x8A23);
        reg.add_hl(0x8A23);
        assert_eq!(reg.hl(), 0x1446);
        assert_eq!(reg.f, Flags::HALF_CARRY | Flags::CARRY);
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
        assert_eq!(reg.f, Flags::SUBTRACT | Flags::ZERO);

        let mut reg = Registers::default();
        reg.a = 0x3E;
        reg.sub(0x0F);
        assert_eq!(reg.a, 0x2F);
        assert_eq!(reg.f, Flags::SUBTRACT | Flags::HALF_CARRY);

        let mut reg = Registers::default();
        reg.a = 0x3E;
        reg.sub(0x40);
        assert_eq!(reg.a, 0xFE);
        assert_eq!(reg.f, Flags::SUBTRACT | Flags::CARRY);

        let mut reg = Registers::default();
        reg.a = 0x00;
        reg.sub(0x01);
        assert_eq!(reg.a, 0xFF);
        assert_eq!(reg.f, Flags::SUBTRACT | Flags::HALF_CARRY | Flags::CARRY);

        let mut reg = Registers::default();
        reg.a = 0xFF;
        reg.sub(0x0F);
        assert_eq!(reg.a, 0xF0);
        assert_eq!(reg.f, Flags::SUBTRACT);
    }

    #[test]
    fn sbc() {
        let mut reg = Registers::default();
        reg.a = 0x3B;
        reg.f.insert(Flags::CARRY);
        reg.sbc(0x2A);
        assert_eq!(reg.a, 0x10);
        assert_eq!(reg.f, Flags::SUBTRACT);

        let mut reg = Registers::default();
        reg.a = 0x3B;
        reg.f.insert(Flags::CARRY);
        reg.sbc(0x3A);
        assert_eq!(reg.a, 0);
        assert_eq!(reg.f, Flags::SUBTRACT | Flags::ZERO);

        let mut reg = Registers::default();
        reg.a = 0x3B;
        reg.f.insert(Flags::CARRY);
        reg.sbc(0x4F);
        assert_eq!(reg.a, 0xEB);
        assert_eq!(reg.f, Flags::SUBTRACT | Flags::HALF_CARRY | Flags::CARRY);
    }

    #[test]
    fn and() {
        let mut reg = Registers::default();
        reg.a = 0x5A;
        reg.and(0x3F);
        assert_eq!(reg.a, 0x1A);
        assert_eq!(reg.f, Flags::HALF_CARRY);

        let mut reg = Registers::default();
        reg.a = 0x5A;
        reg.and(0x38);
        assert_eq!(reg.a, 0x18);
        assert_eq!(reg.f, Flags::HALF_CARRY);

        let mut reg = Registers::default();
        reg.a = 0x5A;
        reg.and(0x00);
        assert_eq!(reg.a, 0);
        assert_eq!(reg.f, Flags::ZERO | Flags::HALF_CARRY);
    }

    #[test]
    fn or() {
        let mut reg = Registers::default();
        reg.a = 0x5A;
        reg.or(0x5A);
        assert_eq!(reg.a, 0x5A);
        assert!(reg.f.is_empty());

        let mut reg = Registers::default();
        reg.a = 0x5A;
        reg.or(3);
        assert_eq!(reg.a, 0x5B);
        assert!(reg.f.is_empty());

        let mut reg = Registers::default();
        reg.a = 0x5A;
        reg.or(0x0F);
        assert_eq!(reg.a, 0x5F);
        assert!(reg.f.is_empty());

        let mut reg = Registers::default();
        reg.a = 0;
        reg.or(0);
        assert_eq!(reg.a, 0);
        assert_eq!(reg.f, Flags::ZERO);
    }

    #[test]
    fn cp() {
        let mut reg = Registers::default();
        reg.a = 0x3C;
        reg.cp(0x2F);
        assert_eq!(reg.a, 0x3C);
        assert_eq!(reg.f, Flags::SUBTRACT | Flags::HALF_CARRY);

        let mut reg = Registers::default();
        reg.a = 0x3C;
        reg.cp(0x3C);
        assert_eq!(reg.a, 0x3C);
        assert_eq!(reg.f, Flags::SUBTRACT | Flags::ZERO);

        let mut reg = Registers::default();
        reg.a = 0x3C;
        reg.cp(0x40);
        assert_eq!(reg.a, 0x3C);
        assert_eq!(reg.f, Flags::SUBTRACT | Flags::CARRY);
    }

    #[test]
    fn rlc() {
        let mut reg = Registers::default();
        reg.a = 0x85;
        reg.rlc();

        // This is a different value than the GameBoy programming manual, which specifies `0x0A` as
        // the correct result.
        assert_eq!(reg.a, 0x0B);
        assert_eq!(reg.f, Flags::CARRY);
    }

    #[test]
    fn rrc() {
        let mut reg = Registers::default();
        reg.a = 0x11;
        reg.rrc();

        assert_eq!(reg.a, 0x88);
        assert_eq!(reg.f, Flags::CARRY);

        reg.a = 0x10;
        reg.rrc();

        assert_eq!(reg.a, 0x08);
        assert_eq!(reg.f, Flags::empty());
    }

    #[test]
    fn rr() {
        let mut reg = Registers::default();
        reg.a = 0x11;
        reg.rr();

        assert_eq!(reg.a, 0x08);
        assert_eq!(reg.f, Flags::CARRY);

        reg.a = 0x10;
        reg.f = Flags::CARRY;
        reg.rr();

        assert_eq!(reg.a, 0x88);
        assert_eq!(reg.f, Flags::empty());
    }

    #[test]
    fn cpl() {
        let mut reg = Registers::default();
        reg.a = 0x35;
        reg.cpl();
        assert_eq!(reg.a, 0xCA);
        assert_eq!(reg.f, Flags::SUBTRACT | Flags::HALF_CARRY);
    }

    quickcheck! {
        fn ccf(flags: u8) -> bool {
            let mut reg = Registers::default();
            reg.f = Flags::from_bits_truncate(flags);

            let carry_set = reg.f.contains(Flags::CARRY);
            let zero_set = reg.f.contains(Flags::ZERO);

            reg.ccf();

            !reg.f.intersects(Flags::SUBTRACT | Flags::HALF_CARRY)
                && carry_set != reg.f.contains(Flags::CARRY)
                && zero_set == reg.f.contains(Flags::ZERO)
        }
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

    #[test]
    fn daa() {
        // FIXME: We should decide what to do in the undocumented cases,
        // and maybe test them.

        let mut registers = Registers::default();

        // Test with no flags set
        for i in 0x00..0xff {
            registers.a = i;
            registers.f = Flags::empty();

            registers.daa();

            let lo = i & 0x0f;
            let hi = i & 0xf0;

            if hi <= 0x90 && lo <= 0x9 {
                assert_eq!(registers.a, i);
                assert!(!registers.f.contains(Flags::CARRY));
            } else if hi <= 0x80 && lo >= 0xa {
                assert_eq!(registers.a, i.wrapping_add(0x6));
                assert!(!registers.f.contains(Flags::CARRY));
            } else if hi >= 0xa0 && lo <= 0x9 {
                assert_eq!(registers.a, i.wrapping_add(0x60));
                assert!(registers.f.contains(Flags::CARRY));
            } else if hi >= 0x90 && lo >= 0xa {
                assert_eq!(registers.a, i.wrapping_add(0x66));
                assert!(registers.f.contains(Flags::CARRY));
            }
        }

        // Test with only carry flag set
        for i in 0x00..0xff {
            registers.a = i;
            registers.f = Flags::empty();
            registers.f.insert(Flags::CARRY);

            registers.daa();

            let lo = i & 0x0f;
            let hi = i & 0xf0;

            if hi <= 0x20 && lo <= 0x9 {
                assert_eq!(registers.a, i.wrapping_add(0x60));
                assert!(registers.f.contains(Flags::CARRY));
            } else if hi <= 0x20 && lo >= 0xa {
                assert_eq!(registers.a, i.wrapping_add(0x66));
                assert!(registers.f.contains(Flags::CARRY));
            }
        }


        // Test with only half-carry flag set
        for i in 0x00..0xff {
            registers.a = i;
            registers.f = Flags::empty();
            registers.f.insert(Flags::HALF_CARRY);

            registers.daa();

            let lo = i & 0x0f;
            let hi = i & 0xf0;

            if hi <= 0x90 && lo <= 0x3 {
                assert_eq!(registers.a, i.wrapping_add(0x6));
                assert!(!registers.f.contains(Flags::CARRY));
            } else if hi >= 0xa0 && lo <= 0x3 {
                assert_eq!(registers.a, i.wrapping_add(0x66));
                assert!(registers.f.contains(Flags::CARRY));
            }
        }

        // Test with carry and half-carry flags set
        for i in 0x00..0xff {
            registers.a = i;
            registers.f = Flags::HALF_CARRY | Flags::CARRY;

            registers.daa();

            let lo = i & 0x0f;
            let hi = i & 0xf0;

            if hi <= 0x30 && lo <= 0x3 {
                assert_eq!(registers.a, i.wrapping_add(0x66));
                assert!(registers.f.contains(Flags::CARRY));
            }
        }

        // Test with only subtraction flag set
        for i in 0x00..0xff {
            registers.a = i;
            registers.f = Flags::SUBTRACT;

            registers.daa();

            let lo = i & 0x0f;
            let hi = i & 0xf0;

            if hi <= 0x90 && lo <= 0x9 {
                assert_eq!(registers.a, i);
                assert!(!registers.f.contains(Flags::CARRY));
            }
        }

        // Test with subtraction and carry flags set
        for i in 0x00..0xff {
            registers.a = i;
            registers.f = Flags::SUBTRACT | Flags::CARRY;

            registers.daa();

            let lo = i & 0x0f;
            let hi = i & 0xf0;

            if hi >= 0x70 && lo <= 0x9 {
                assert_eq!(registers.a, i.wrapping_add(0xa0));
                assert!(registers.f.contains(Flags::CARRY));
            }
        }

        // Test with subtraction and half-carry flags set
        for i in 0x00..0xff {
            registers.a = i;
            registers.f = Flags::SUBTRACT | Flags::HALF_CARRY;

            registers.daa();

            let lo = i & 0x0f;
            let hi = i & 0xf0;

            if hi <= 0x80 && lo >= 0x6 {
                assert_eq!(registers.a, i.wrapping_add(0xfa));
                assert!(!registers.f.contains(Flags::CARRY));
            }
        }

        // Test with subtraction, carry, and half-carry flags set
        for i in 0x00..0xff {
            registers.a = i;
            registers.f = Flags::SUBTRACT | Flags::CARRY | Flags::HALF_CARRY;

            registers.daa();

            let lo = i & 0x0f;
            let hi = i & 0xf0;

            if hi >= 0x60 && lo >= 0x6 {
                assert_eq!(registers.a, i.wrapping_add(0x9a));
                assert!(registers.f.contains(Flags::CARRY));
            }
        }
    }
}
