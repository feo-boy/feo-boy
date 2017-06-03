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
    pub fn fetch(&mut self) -> Instruction {
        self.fetch_instr(&INSTRUCTIONS)
    }

    pub fn fetch_instr(&mut self, table: &[Option<Instruction>]) -> Instruction {
        let byte = self.mmu.borrow().read_byte(self.reg.pc);

        // TODO remove after all opcodes done
        let in_cb = table[0].map(|x| x.description != "NOP").unwrap_or_default();

        let mut instruction = table[byte as usize]
            .expect(&format!("could not find data for instruction {:#0x} -- is 0xcb {}",
                            byte,
                            in_cb)); // TODO remove after all opcodes done

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
                self.reg
                    .hl_mut()
                    .write(LittleEndian::read_u16(instruction.operands()))
            }

            // LD SP,d16
            0x31 => self.reg.sp = LittleEndian::read_u16(instruction.operands()),

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
            0x06 => {
                self.reg.b = instruction.operands()[0];
            }

            // LD D,d8
            0x16 => {
                self.reg.d = instruction.operands()[0];
            }

            // LD H,d8
            0x26 => {
                self.reg.h = instruction.operands()[0];
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
            0x0e => {
                self.reg.c = instruction.operands()[0];
            }

            // LD E,d8
            0x1e => {
                self.reg.e = instruction.operands()[0];
            }

            // LD L,d8
            0x2e => {
                self.reg.l = instruction.operands()[0];
            }

            // LD A,d8
            0x3e => {
                self.reg.a = instruction.operands()[0];
            }

            // XOR (HL)
            0xae => {
                let byte = self.mmu.borrow().read_byte(self.reg.hl());
                self.xor(byte);
            }

            // XOR d8
            0xee => self.xor(instruction.operands()[0]),

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

            // PREFIX CB
            0xcb => {
                self.reg.pc += 1 + instruction.operands().len() as u16;
                let instruction_cb = self.fetch_instr(&INSTRUCTIONS_CB);
                self.execute_cb(&instruction_cb);
                return;
            }

            _ => panic!("unimplemented instruction: {:?}", instruction),
        }
        self.reg.pc += 1 + instruction.operands().len() as u16;
    }

    /// Performs an exclusive OR with the accumulator and sets the zero flag appropriately.
    fn xor(&mut self, rhs: u8) {
        self.reg.a ^= rhs;
        self.reg.f = Flags::empty();
        self.reg.f.set(ZERO, self.reg.a == 0);
    }

    fn rst(&mut self, addr: u16) {
        let new_pc = self.reg.pc + 3;
        self.push(new_pc);
        self.reg.pc = addr;
    }

    pub fn execute_cb(&mut self, instruction: &Instruction) {
        debug!("executing 0xcb{:?}", instruction);

        match instruction.byte {
            0x7c => {
                self.reg.f.set(ZERO, self.reg.h.has_bit_set(7));
                self.reg.f.remove(SUBTRACT);
                self.reg.f.insert(HALF_CARRY);
            }

            _ => panic!("unimplemented instruction: 0xcb{:?}", instruction),
        }

        self.reg.pc += 1 + instruction.operands().len() as u16;
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
        0xc1,       "POP BC",       0,              12;
        0xd1,       "POP DC",       0,              12;
        0xe1,       "POP HL",       0,              12;
        0xf1,       "POP AF",       0,              12;
        0x32,       "LD (HL-),A",   0,              8;
        0xe2,       "LD (C),A",     1,              8; // Alternatively LD ($rFF00+C),A
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
        0xcb,       "PREFIX CB",    0,              4;
        0xac,       "XOR H",        0,              4;
        0xad,       "XOR L",        0,              4;
        0x0e,       "LD C,d8",      1,              8;
        0x1e,       "LD E,d8",      1,              8;
        0x2e,       "LD L,d8",      1,              8;
        0x3e,       "LD A,d8",      1,              8;
        0xae,       "XOR (HL)",     0,              8;
        0xee,       "XOR d8",       1,              8;
        0xaf,       "XOR A",        0,              4;
        0xcf,       "RST 08H",      0,              16;
        0xdf,       "RST 18H",      0,              16;
        0xef,       "RST 28H",      0,              16;
        0xff,       "RST 38H",      0,              16;
    };

}

lazy_static! {
    static ref INSTRUCTIONS_CB: Vec<Option<Instruction>> = instructions! {
        // byte     description     operands        cycles
        //0x00,       "RLC B",        1,              8;
        0x7c,       "BIT 7,H",      0,              8;
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

    #[test]
    fn ld_addr_c_a() {
        let mmu = Mmu::default();
        let mut cpu = Cpu::new(Rc::new(RefCell::new(mmu)));

        let mut instruction = INSTRUCTIONS[0xe2].unwrap();

        cpu.reg.c = 0x11;
        cpu.reg.a = 0xab;
    }
}
