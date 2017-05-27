//! CPU instruction definition.

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

            // LD HL,d16
            0x21 => self.reg.write_hl(LittleEndian::read_u16(instruction.operands())),

            // LD SP,d16
            0x31 => self.reg.sp = LittleEndian::read_u16(instruction.operands()),

            // LD (HL-),A
            0x32 => {
                self.mmu.borrow_mut().write_byte(self.reg.read_hl(), self.reg.a);
                self.reg.dec_hl();
            },

            // XOR A
            0xaf => {
                // Effectively sets A to 0 and unconditionally sets the Zero flag.
                self.reg.a ^= self.reg.a;
                self.reg.f = Flags::empty();
                self.reg.f.set(ZERO, self.reg.a == 0);
            },

            _ => panic!("unimplemented instruction: {:?}", instruction),
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
        0x21,       "LD HL,d16",    2,              12;
        0x31,       "LD SP,d16",    2,              12;
        0x32,       "LD (HL-),A",   0,              8;
        0xaf,       "XOR A",        0,              4;
    };
}
