
use bus::Bus;
use cpu::{Cpu, arithmetic};
use memory::Addressable;

/// Macro to quickly define all prefix instructions.
macro_rules! prefix_instructions {
    ( $( $byte:expr, $description:expr ; )* ) => {
        {
            use $crate::regex::Regex;

            let memory_access_re = Regex::new(r"\(HL\)").unwrap();

            // FIXME: This should be an array once all instructions are defined.
            let mut prefix_instructions = vec![PrefixInstructionDef::default(); 0x100];

            $(
                // If the instruction accesses memory (through HL), the instruction will take 16
                // cycles. Otherwise, it will take 8.
                let cycles = if memory_access_re.is_match($description) {
                    16
                } else {
                    8
                };

                prefix_instructions[$byte] = PrefixInstructionDef {
                    byte: $byte,
                    description: $description,
                    cycles,
                };
            )*

            prefix_instructions
        }
    }
}

/// A definition of a prefix (`0xCB`) instruction.
///
/// These instructions are notably simpler than more general instructions, as they are always 8 or
/// 16 cycles each and do not have operands.
#[derive(Debug, Clone)]
pub struct PrefixInstructionDef {
    /// The byte that identifies this prefix instruction.
    pub byte: u8,

    /// A short, human readable representation of the prefix instruction in Z80 assembly syntax.
    pub description: &'static str,

    /// The number of clock cycles it takes to execute this instruction.
    ///
    /// See the note on `InstructionDef::cycles` for how this field differs from machine cycles.
    pub cycles: u8,
}

// FIXME: Remove this impl once all prefix instructions are implemented.
impl Default for PrefixInstructionDef {
    fn default() -> PrefixInstructionDef {
        PrefixInstructionDef {
            byte: 0,
            description: "UNDEFINED PREFIX INSTRUCTION",
            cycles: 8,
        }
    }
}

macro_rules! prefix_mut_flag {
    ($start:expr, $func:path, $instr:expr, $reg:expr, $bus:expr) => {
        if $instr == $start {
            $func(&mut $reg.b, &mut $reg.f);
            return;
        } else if $instr == $start+1 {
            $func(&mut $reg.c, &mut $reg.f);
            return;
        } else if $instr == $start+2 {
            $func(&mut $reg.d, &mut $reg.f);
            return;
        } else if $instr == $start+3 {
            $func(&mut $reg.e, &mut $reg.f);
            return;
        } else if $instr == $start+4 {
            $func(&mut $reg.h, &mut $reg.f);
            return;
        } else if $instr == $start+5 {
            $func(&mut $reg.l, &mut $reg.f);
            return;
        } else if $instr == $start+6 {
            let mut byte = $bus.read_byte($reg.hl());
            $func(&mut byte, &mut $reg.f);
            $bus.write_byte($reg.hl(), byte);
            return;
        } else if $instr == $start+7 {
            $func(&mut $reg.a, &mut $reg.f);
            return;
        }
    }
}

macro_rules! prefix_bit_enum_flag {
    ($start:expr, $func:path, $bit:expr, $instr:expr, $reg:expr, $bus:expr) => {
        if $instr == $start {
            $func($reg.b, $bit, &mut $reg.f);
            return;
        } else if $instr == $start+1 {
            $func($reg.c, $bit, &mut $reg.f);
            return;
        } else if $instr == $start+2 {
            $func($reg.d, $bit, &mut $reg.f);
            return;
        } else if $instr == $start+3 {
            $func($reg.e, $bit, &mut $reg.f);
            return;
        } else if $instr == $start+4 {
            $func($reg.h, $bit, &mut $reg.f);
            return;
        } else if $instr == $start+5 {
            $func($reg.l, $bit, &mut $reg.f);
            return;
        } else if $instr == $start+6 {
            $func($bus.read_byte($reg.hl()), $bit, &mut $reg.f);
            return;
        } else if $instr == $start+7 {
            $func($reg.a, $bit, &mut $reg.f);
            return;
        }
    }
}

macro_rules! prefix_bit_flag {
    ($start:expr, $func:path, $instr:expr, $reg:expr, $bus:expr) => {
        prefix_bit_enum_flag![$start, $func, 0, $instr, $reg, $bus];
        prefix_bit_enum_flag![$start+0x08, $func, 1, $instr, $reg, $bus];
        prefix_bit_enum_flag![$start+0x10, $func, 2, $instr, $reg, $bus];
        prefix_bit_enum_flag![$start+0x18, $func, 3, $instr, $reg, $bus];
        prefix_bit_enum_flag![$start+0x20, $func, 4, $instr, $reg, $bus];
        prefix_bit_enum_flag![$start+0x28, $func, 5, $instr, $reg, $bus];
        prefix_bit_enum_flag![$start+0x30, $func, 6, $instr, $reg, $bus];
        prefix_bit_enum_flag![$start+0x38, $func, 7, $instr, $reg, $bus];
    }
}

macro_rules! prefix_set_res_enum {
    ($start:expr, $func:path, $bit:expr, $instr:expr, $reg:expr, $bus:expr) => {
        if $instr == $start {
            $func(&mut $reg.b, $bit);
            return;
        } else if $instr == $start+1 {
            $func(&mut $reg.c, $bit);
            return;
        } else if $instr == $start+2 {
            $func(&mut $reg.d, $bit);
            return;
        } else if $instr == $start+3 {
            $func(&mut $reg.e, $bit);
            return;
        } else if $instr == $start+4 {
            $func(&mut $reg.h, $bit);
            return;
        } else if $instr == $start+5 {
            $func(&mut $reg.l, $bit);
            return;
        } else if $instr == $start+6 {
            let mut byte = $bus.read_byte($reg.hl());
            $func(&mut byte, $bit);
            $bus.write_byte($reg.hl(), byte);
            return;
        } else if $instr == $start+7 {
            $func(&mut $reg.a, $bit);
            return;
        }
    }
}

macro_rules! prefix_set_res {
    ($start:expr, $func:path, $instr:expr, $reg:expr, $bus:expr) => {
        prefix_set_res_enum![$start, $func, 0, $instr, $reg, $bus];
        prefix_set_res_enum![$start+0x08, $func, 1, $instr, $reg, $bus];
        prefix_set_res_enum![$start+0x10, $func, 2, $instr, $reg, $bus];
        prefix_set_res_enum![$start+0x18, $func, 3, $instr, $reg, $bus];
        prefix_set_res_enum![$start+0x20, $func, 4, $instr, $reg, $bus];
        prefix_set_res_enum![$start+0x28, $func, 5, $instr, $reg, $bus];
        prefix_set_res_enum![$start+0x30, $func, 6, $instr, $reg, $bus];
        prefix_set_res_enum![$start+0x38, $func, 7, $instr, $reg, $bus];
    }
}

impl Cpu {
    pub fn execute_prefix(&mut self, instruction: u8, bus: &mut Bus) {
        prefix_mut_flag![0x00, arithmetic::rlc, instruction, self.reg, bus];
        prefix_mut_flag![0x08, arithmetic::rrc, instruction, self.reg, bus];
        prefix_mut_flag![0x10, arithmetic::rl, instruction, self.reg, bus];
        prefix_mut_flag![0x18, arithmetic::rr, instruction, self.reg, bus];
        prefix_mut_flag![0x20, arithmetic::sla, instruction, self.reg, bus];
        prefix_mut_flag![0x28, arithmetic::sra, instruction, self.reg, bus];
        prefix_mut_flag![0x30, arithmetic::swap, instruction, self.reg, bus];
        prefix_mut_flag![0x38, arithmetic::srl, instruction, self.reg, bus];

        prefix_bit_flag![0x40, arithmetic::bit, instruction, self.reg, bus];

        prefix_set_res![0x80, arithmetic::res, instruction, self.reg, bus];
        prefix_set_res![0xc0, arithmetic::set, instruction, self.reg, bus];

        println!("Error!");
    }
}

lazy_static! {
    /// Prefix instruction definitions.
    ///
    /// Descriptions taken from [here].
    ///
    /// [here]: http://pastraiser.com/cpu/gameboy/gameboy_opcodes.html
    pub static ref PREFIX_INSTRUCTIONS: Vec<PrefixInstructionDef> = prefix_instructions! {
        // byte     description
        0x11,       "RL C";
        0x86,       "RES 0,(HL)";
    };
}
