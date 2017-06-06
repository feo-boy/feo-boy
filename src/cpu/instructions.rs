//! CPU instruction definition.

use std::fmt::{self, Display};
use std::ops::{AddAssign, SubAssign};

use byteorder::{ByteOrder, LittleEndian};
use regex::{Regex, NoExpand};
use smallvec::SmallVec;

use cpu::{Flags, ZERO, SUBTRACT, HALF_CARRY, CARRY};

/// A definition of a single instruction.
#[derive(Debug, Clone)]
struct InstructionDef {
    /// The byte that identifies this instruction.
    pub byte: u8,

    /// A short, human readable representation of the instruction in Z80 assembly syntax.
    pub description: &'static str,

    /// The number of clock cycles it takes to execute this instruction.
    ///
    /// Note that this is measured in clock cycles, not machine cycles. While Nintendo's official
    /// documentation records the timings in machine cycles, most online documentation uses clock
    /// cycles. Four clock cycles is equivalent to a single machine cycle.
    pub cycles: u8,

    /// The number of operands that this instruction uses.
    pub num_operands: u8,
}

/// A single instruction to be executed by the CPU.
#[derive(Debug, Clone)]
pub struct Instruction {
    def: &'static InstructionDef,

    /// Vector containing the operands of the instruction. May be empty.
    operands: SmallVec<[u8; 2]>,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        lazy_static! {
            static ref DATA_RE: Regex = Regex::new("d8|d16|a8|a16|r8").unwrap();
        }

        let instruction = if let Some(mat) = DATA_RE.find(self.def.description) {
            let replacement = match mat.as_str() {
                "d8" | "a8" | "r8" => format!("${:#02x}", &self.operands[0]),
                "d16" | "a16" => format!("${:#04x}", LittleEndian::read_u16(&self.operands)),
                _ => unreachable!(),
            };

            DATA_RE
                .replace_all(self.def.description, NoExpand(replacement.as_str()))
                .to_string()
        } else {
            self.def.description.to_string()
        };

        write!(f, "{}", instruction)
    }
}

/// Provides additional functionality for bit manipulation.
trait ByteExt {
    /// Returns whether the byte has its nth bit set.
    fn has_bit_set(&self, n: u8) -> bool;
}

impl ByteExt for u8 {
    fn has_bit_set(&self, n: u8) -> bool {
        if n > 7 {
            panic!("bit {} is out of range for u8", n);
        }

        (self & (1 << n)) != 0
    }
}

/// Macro to quickly define all CPU instructions for the Game Boy Z80 processor.
macro_rules! instructions {
    ( $( $byte:expr, $description:expr, $num_operands:expr, $cycles:expr ; )* ) => {
        {
            let mut instructions = vec![None; 0x100];

            $(
                instructions[$byte] = Some(InstructionDef {
                    byte: $byte,
                    description: $description,
                    cycles: $cycles,
                    num_operands: $num_operands,
                });
            )*

            instructions
        }
    }
}

impl super::Cpu {
    /// Decodes the next instruction.
    pub fn fetch(&self) -> Instruction {
        let byte = self.mmu.borrow().read_byte(self.reg.pc);

        let def = INSTRUCTIONS[byte as usize]
            .as_ref()
            .expect(&format!("could not find data for instruction {:#0x}", byte));

        let operands = (0..def.num_operands)
            .map(|i| self.mmu.borrow().read_byte(self.reg.pc + 1 + i as u16))
            .collect();

        Instruction {
            def: def,
            operands: operands,
        }
    }

    /// Executes an instruction.
    ///
    /// All necessary side effects are performed, including updating the program counter and flag
    /// registers.
    pub fn execute(&mut self, instruction: Instruction) {
        debug!("executing {:<20}", instruction.to_string());
        trace!("{:?}", instruction);

        // Check that we have exactly as many operands as the instruction requires.
        debug_assert_eq!(instruction.def.num_operands as usize,
                         instruction.operands.len());

        match instruction.def.byte {
            // NOP
            0x00 => (),

            // JR NZ,r8
            0x20 => {
                if !self.reg.f.contains(ZERO) {
                    let jump = instruction.operands[0] as i8;
                    let pc = self.reg.pc as i16;

                    self.reg.pc = (pc + jump as i16) as u16;

                    // FIXME: Need to add four clock cycles to this instruction
                    // in this case.
                }
            }

            // LD HL,d16
            0x21 => {
                self.reg
                    .hl_mut()
                    .write(LittleEndian::read_u16(&instruction.operands))
            }

            // LD SP,d16
            0x31 => self.reg.sp = LittleEndian::read_u16(&instruction.operands),

            // POP BC
            0xc1 => {
                let bc = self.pop();
                self.reg.bc_mut().write(bc);
            }

            // POP DE
            0xd1 => {
                let de = self.pop();
                self.reg.de_mut().write(de);
            }

            // POP HL
            0xe1 => {
                let hl = self.pop();
                self.reg.hl_mut().write(hl);
            }

            // POP AF
            0xf1 => {
                let af = self.pop();
                self.reg.af_mut().write(af);
            }

            // LD (HL-),A
            0x32 => {
                self.mmu.borrow_mut().write_byte(self.reg.hl(), self.reg.a);
                self.reg.hl_mut().sub_assign(1);
            }

            // LD (C),A
            // LD ($FF00+C),A
            0xe2 => {
                self.mmu
                    .borrow_mut()
                    .write_byte(0xFF00 + self.reg.c as u16, self.reg.a);
            }

            // INC BC
            0x03 => self.reg.bc_mut().add_assign(1),

            // INC DE
            0x13 => self.reg.de_mut().add_assign(1),

            // INC HL
            0x23 => self.reg.hl_mut().add_assign(1),

            // INC SP
            0x33 => self.reg.sp.add_assign(1),

            // INC B
            0x04 => Self::inc(&mut self.reg.b, &mut self.reg.f),

            // INC D
            0x14 => Self::inc(&mut self.reg.d, &mut self.reg.f),

            // INC H
            0x24 => Self::inc(&mut self.reg.h, &mut self.reg.f),

            // INC (HL)
            0x34 => {
                let mut byte = self.mmu.borrow().read_byte(self.reg.hl());
                Self::inc(&mut byte, &mut self.reg.f);
                self.mmu.borrow_mut().write_byte(self.reg.hl(), byte);
            }

            // PUSH BC
            0xc5 => {
                let bc = self.reg.bc();
                self.push(bc);
            }

            // PUSH DE
            0xd5 => {
                let de = self.reg.de();
                self.push(de);
            }

            // PUSH HL
            0xe5 => {
                let hl = self.reg.hl();
                self.push(hl);
            }

            // PUSH AF
            0xf5 => {
                let af = self.reg.af();
                self.push(af);
            }

            // LD B,d8
            0x06 => self.reg.b = instruction.operands[0],

            // LD D,d8
            0x16 => self.reg.d = instruction.operands[0],

            // LD H,d8
            0x26 => self.reg.h = instruction.operands[0],

            // RST 00H
            0xc7 => {
                self.rst(0x0000);
                return;
            }

            // RST 10H
            0xd7 => {
                self.rst(0x0010);
                return;
            }

            // RST 20H
            0xe7 => {
                self.rst(0x0020);
                return;
            }

            // RST 30H
            0xf7 => {
                self.rst(0x0030);
                return;
            }

            // XOR B
            0xa8 => {
                let b = self.reg.b;
                self.xor(b)
            }

            // XOR C
            0xa9 => {
                let c = self.reg.c;
                self.xor(c);
            }

            // XOR D
            0xaa => {
                let d = self.reg.d;
                self.xor(d);
            }

            // XOR E
            0xab => {
                let e = self.reg.e;
                self.xor(e);
            }

            // INC C
            0x0c => Self::inc(&mut self.reg.c, &mut self.reg.f),

            // INC E
            0x1c => Self::inc(&mut self.reg.e, &mut self.reg.f),

            // INC L
            0x2c => Self::inc(&mut self.reg.l, &mut self.reg.f),

            // INC A
            0x3c => Self::inc(&mut self.reg.a, &mut self.reg.f),

            // XOR H
            0xac => {
                let h = self.reg.h;
                self.xor(h);
            }

            // XOR L
            0xad => {
                let l = self.reg.l;
                self.xor(l);
            }

            // LD C,d8
            0x0e => self.reg.c = instruction.operands[0],

            // LD E,d8
            0x1e => self.reg.e = instruction.operands[0],

            // LD L,d8
            0x2e => self.reg.l = instruction.operands[0],

            // LD A,d8
            0x3e => self.reg.a = instruction.operands[0],

            // XOR (HL)
            0xae => {
                let byte = self.mmu.borrow().read_byte(self.reg.hl());
                self.xor(byte);
            }

            // XOR d8
            0xee => self.xor(instruction.operands[0]),

            // CP d8
            0xfe => self.cp(instruction.operands[0]),

            // XOR A
            0xaf => {
                // Effectively sets A to 0 and unconditionally sets the Zero flag.
                let a = self.reg.a;
                self.xor(a);
            }

            // RST 08H
            0xcf => {
                self.rst(0x0008);
                return;
            }

            // RST 18H
            0xdf => {
                self.rst(0x0018);
                return;
            }

            // RST 28H
            0xef => {
                self.rst(0x0028);
                return;
            }

            // RST 38H
            0xff => {
                self.rst(0x0038);
                return;
            }

            _ => panic!("unimplemented instruction: {:?}", instruction),
        }

        self.reg.pc += 1 + instruction.operands.len() as u16;
    }

    /// Performs an exclusive OR with the accumulator and sets the zero flag appropriately.
    fn xor(&mut self, rhs: u8) {
        self.reg.a ^= rhs;
        self.reg.f = Flags::empty();
        self.reg.f.set(ZERO, self.reg.a == 0);
    }

    /// Pushes the program counter (plus 3) onto the stack, then sets the program counter to a
    /// specific value.
    fn rst(&mut self, addr: u16) {
        let new_pc = self.reg.pc + 3;
        self.push(new_pc);
        self.reg.pc = addr;
    }

    /// Compares a byte with the accumulator.
    ///
    /// Performs a subtraction with the accumulator without actually setting the accumulator to the
    /// new value. Only the flags are set.
    fn cp(&mut self, rhs: u8) {
        let a = self.reg.a;

        self.reg.f.set(ZERO, a == rhs);
        self.reg.f.set(CARRY, a < rhs);
    }

    /// Increments a byte by 1 and sets the flags appropriately.
    fn inc(byte: &mut u8, flags: &mut Flags) {
        flags.set(HALF_CARRY, is_half_carry(*byte, 1));

        *byte += 1;

        flags.set(ZERO, *byte == 0);
        flags.remove(SUBTRACT);
    }
}

/// Returns `true` if the addition of two bytes would require a half carry (a carry from the low
/// nibble to the high nibble).
fn is_half_carry(a: u8, b: u8) -> bool {
    (((a & 0xf) + (b & 0xf)) & 0x10) == 0x10
}

lazy_static! {
    /// Game Boy instruction set.
    ///
    /// Timings and other information taken from [here].
    ///
    /// [here]: http://pastraiser.com/cpu/gameboy/gameboy_opcodes.html
    // FIXME: This should be `[Instruction; 0x100]` once all instructions are implemented.
    static ref INSTRUCTIONS: Vec<Option<InstructionDef>> = instructions! {
        // byte     description     operands        cycles
        0x00,       "NOP",          0,              4;
        0x20,       "JR NZ,r8",     1,              8;
        0x21,       "LD HL,d16",    2,              12;
        0x31,       "LD SP,d16",    2,              12;
        0xc1,       "POP BC",       0,              12;
        0xd1,       "POP DC",       0,              12;
        0xe1,       "POP HL",       0,              12;
        0xf1,       "POP AF",       0,              12;
        0x32,       "LD (HL-),A",   0,              8;
        0xe2,       "LD (C),A",     0,              8; // AKA LD ($rFF00+C),A
        0x03,       "INC BC",       0,              8;
        0x13,       "INC DE",       0,              8;
        0x23,       "INC HL",       0,              8;
        0x33,       "INC SP",       0,              8;
        0x04,       "INC B",        0,              4;
        0x14,       "INC D",        0,              4;
        0x24,       "INC H",        0,              4;
        0x34,       "INC (HL)",     0,              12;
        0xc5,       "PUSH BC",      0,              16;
        0xd5,       "PUSH DE",      0,              16;
        0xe5,       "PUSH HL",      0,              16;
        0xf5,       "PUSH AF",      0,              16;
        0x06,       "LD B,d8",      1,              8;
        0x16,       "LD D,d8",      1,              8;
        0x26,       "LD H,d8",      1,              8;
        0xc7,       "RST 00H",      0,              16;
        0xd7,       "RST 10H",      0,              16;
        0xe7,       "RST 20H",      0,              16;
        0xf7,       "RST 30H",      0,              16;
        0xa8,       "XOR B",        0,              4;
        0xa9,       "XOR C",        0,              4;
        0xaa,       "XOR D",        0,              4;
        0xab,       "XOR E",        0,              4;
        0x0c,       "INC C",        0,              4;
        0x1c,       "INC E",        0,              4;
        0x2c,       "INC L",        0,              4;
        0x3c,       "INC A",        0,              4;
        0xac,       "XOR H",        0,              4;
        0xad,       "XOR L",        0,              4;
        0x0e,       "LD C,d8",      1,              8;
        0x1e,       "LD E,d8",      1,              8;
        0x2e,       "LD L,d8",      1,              8;
        0x3e,       "LD A,d8",      1,              8;
        0xae,       "XOR (HL)",     0,              8;
        0xee,       "XOR d8",       1,              8;
        0xfe,       "CP d8",        1,              8;
        0xaf,       "XOR A",        0,              4;
        0xcf,       "RST 08H",      0,              16;
        0xdf,       "RST 18H",      0,              16;
        0xef,       "RST 28H",      0,              16;
        0xff,       "RST 38H",      0,              16;
    };
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use smallvec::SmallVec;

    use cpu::Cpu;
    use memory::Mmu;

    use super::{INSTRUCTIONS, Instruction, ByteExt};

    #[test]
    fn has_bit_set() {
        let byte = 0x80;
        assert!(byte.has_bit_set(7));
        assert!(!byte.has_bit_set(0));
    }

    #[test]
    #[should_panic(expected = "bit 8 is out of range for u8")]
    fn bit_out_of_range() {
        0xFF.has_bit_set(8);
    }

    #[test]
    fn half_carry() {
        assert!(super::is_half_carry(0x0f, 0x01));
        assert!(!super::is_half_carry(0x37, 0x44));
    }

    #[test]
    fn instruction_display() {
        let nop = Instruction {
            def: INSTRUCTIONS[0x00].as_ref().unwrap(),
            operands: Default::default(),
        };

        assert_eq!(&nop.to_string(), "NOP");

        let jr_nz_r8 = Instruction {
            def: INSTRUCTIONS[0x20].as_ref().unwrap(),
            operands: SmallVec::from_slice(&[0xfe]),
        };

        assert_eq!(&jr_nz_r8.to_string(), "JR NZ,$0xfe");

        let ld_hl_d16 = Instruction {
            def: INSTRUCTIONS[0x21].as_ref().unwrap(),
            operands: SmallVec::from_slice(&[0xef, 0xbe]),
        };

        assert_eq!(&ld_hl_d16.to_string(), "LD HL,$0xbeef");
    }

    #[test]
    fn fetch_nop() {
        let mmu = Rc::new(RefCell::new(Mmu::default()));
        let cpu = Cpu::new(Rc::clone(&mmu));

        // FIXME: This test works because the MMU will be empty initially (that is, full of NOPs).
        // However, this is fragile.
        let nop = cpu.fetch();

        assert_eq!(nop.def.byte, 0x00);
        assert_eq!(nop.def.num_operands, 0);
        assert_eq!(nop.operands.len(), 0);
    }

    #[test]
    fn rst() {
        let mmu = Rc::new(RefCell::new(Mmu::default()));
        let mut cpu = Cpu::new(Rc::clone(&mmu));

        cpu.reg.sp = 0xFFF0;
        cpu.reg.pc = 0xAB;

        let instruction = Instruction {
            def: INSTRUCTIONS[0xff].as_ref().unwrap(),
            operands: Default::default(),
        };
        cpu.execute(instruction);

        assert_eq!(mmu.borrow().read_word(0xFFF0), 0xAB + 3);
        assert_eq!(cpu.reg.pc, 0x38);
    }

    #[test]
    fn jr_nz() {
        let mmu = Mmu::default();
        let mut cpu = Cpu::new(Rc::new(RefCell::new(mmu)));

        // Move forward 10
        let instruction = Instruction {
            def: INSTRUCTIONS[0x20].as_ref().unwrap(),
            operands: SmallVec::from_slice(&[0x0a]),
        };
        cpu.execute(instruction);
        assert_eq!(cpu.reg.pc, 12);

        // Move backward 10
        let instruction = Instruction {
            def: INSTRUCTIONS[0x20].as_ref().unwrap(),
            operands: SmallVec::from_slice(&[!0x0a + 1]),
        };
        cpu.execute(instruction);
        assert_eq!(cpu.reg.pc, 4);
    }

    #[test]
    fn ld_addr_c_a() {
        let mmu = Mmu::default();
        let mut cpu = Cpu::new(Rc::new(RefCell::new(mmu)));

        cpu.reg.c = 0x11;
        cpu.reg.a = 0xab;

        let instruction = Instruction {
            def: INSTRUCTIONS[0xe2].as_ref().unwrap(),
            operands: Default::default(),
        };
        cpu.execute(instruction);
        // FIXME: We can't actually test this until the I/O memory
        // is implemented.
        // assert_eq!(cpu.mmu.borrow().read_byte(0xFF11), 0xab);
    }
}
