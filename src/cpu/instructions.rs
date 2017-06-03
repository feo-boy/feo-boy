//! CPU instruction definition.

use std::ops::SubAssign;

use byteorder::{ByteOrder, LittleEndian};

use cpu::{Flags, ZERO, SUBTRACT, HALF_CARRY, CARRY};

/// A single instruction to be executed by the CPU.
#[derive(Debug, Copy, Clone)]
pub struct Instruction {
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

    /// An array containing the operand(s) for the instruction.
    ///
    /// Prefer using the `operands()` method over this field, as this array may contain nonsense
    /// data for instructions that use fewer than two operands.
    operand_bytes: [u8; 2],

    /// The number of operands that this instruction uses.
    num_operands: u8,
}

impl Instruction {
    /// Returns a slice containing the operand(s) of the instruction in little-endian order.
    pub fn operands(&self) -> &[u8] {
        &self.operand_bytes[..self.num_operands as usize]
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
                instructions[$byte] = Some(Instruction {
                    byte: $byte,
                    description: $description,
                    cycles: $cycles,
                    operand_bytes: Default::default(),
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

        let mut instruction =
            INSTRUCTIONS[byte as usize]
                .expect(&format!("could not find data for instruction {:#0x}", byte));

        for i in 0..instruction.num_operands {
            let operand = self.mmu.borrow().read_byte(self.reg.pc + 1 + i as u16);

            instruction.operand_bytes[i as usize] = operand;
        }

        instruction
    }

    /// Executes an instruction.
    ///
    /// All necessary side effects are performed, including updating the program counter and flag
    /// registers.
    pub fn execute(&mut self, instruction: &Instruction) {
        debug!("executing {:?}", instruction);

        match instruction.byte {
            // NOP
            0x00 => (),

            // JR NZ,r8
            0x20 => {
                if !self.reg.f.contains(ZERO) {
                    let jump = instruction.operands()[0] as i8;
                    let pc = self.reg.pc as i16;

                    self.reg.pc = (pc + jump as i16) as u16;

                    // FIXME: Need to add four clock cycles to this instruction
                    // in this case.
                }
            }

            // LD HL,d16
            0x21 => {
                self.reg.hl_mut().write(LittleEndian::read_u16(instruction.operands()))
            }

            // LD SP,d16
            0x31 => self.reg.sp = LittleEndian::read_u16(instruction.operands()),

            // LD (HL-),A
            0x32 => {
                self.mmu
                    .borrow_mut()
                    .write_byte(self.reg.hl(), self.reg.a);
                self.reg.hl_mut().sub_assign(1);
            }

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

            // XOR A
            0xaf => {
                // Effectively sets A to 0 and unconditionally sets the Zero flag.
                self.reg.a ^= self.reg.a;
                self.reg.f = Flags::empty();
                self.reg.f.set(ZERO, self.reg.a == 0);
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

        self.reg.pc += 1 + instruction.operands().len() as u16;
    }

    fn rst(&mut self, addr: u16) {
        let new_pc = self.reg.pc + 3;
        self.push(new_pc);
        self.reg.pc = addr;
    }
}

lazy_static! {
    /// Game Boy instruction set.
    ///
    /// Timings and other information taken from [here].
    ///
    /// [here]: http://pastraiser.com/cpu/gameboy/gameboy_opcodes.html
    // FIXME: This should be `[Instruction; 0x100]` once all instructions are implemented.
    static ref INSTRUCTIONS: Vec<Option<Instruction>> = instructions! {
        // byte     description     operands        cycles
        0x00,       "NOP",          0,              4;
        0x20,       "JR NZ,r8",     1,              8;
        0x21,       "LD HL,d16",    2,              12;
        0x31,       "LD SP,d16",    2,              12;
        0x32,       "LD (HL-),A",   0,              8;
        0xc7,       "RST 00H",      0,              16;
        0xd7,       "RST 10H",      0,              16;
        0xe7,       "RST 20H",      0,              16;
        0xf7,       "RST 30H",      0,              16;
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

    use cpu::Cpu;
    use memory::Mmu;

    use super::ByteExt;
    use super::INSTRUCTIONS;

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
    fn rst() {
        let mmu = Rc::new(RefCell::new(Mmu::default()));
        let mut cpu = Cpu::new(Rc::clone(&mmu));

        cpu.reg.sp = 0xFFF0;
        cpu.reg.pc = 0xAB;

        cpu.execute(&INSTRUCTIONS[0xFF].unwrap());

        assert_eq!(mmu.borrow().read_word(0xFFF0), 0xAB + 3);
        assert_eq!(cpu.reg.pc, 0x38);
    }

    #[test]
    fn jr_nz() {
        use std::cell::RefCell;
        use std::rc::Rc;

        use memory::Mmu;
        use cpu::Cpu;

        let mmu = Mmu::default();
        let mut cpu = Cpu::new(Rc::new(RefCell::new(mmu)));

        let mut instruction = super::INSTRUCTIONS[0x20].unwrap();

        // Move forward 10
        instruction.operand_bytes = [0x0a, 0];
        cpu.execute(&instruction);
        assert!(cpu.reg.pc == 12);

        // Move backward 10
        instruction.operand_bytes = [!0x0a + 1, 0];
        cpu.execute(&instruction);
        assert!(cpu.reg.pc == 4);
    }
}
