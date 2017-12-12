use bus::Bus;
use cpu::{Cpu, arithmetic};
use memory::Addressable;

/// Macro to quickly define all prefix instructions.
macro_rules! prefix_instructions {
    ( $( $byte:expr, $description:expr ; )* ) => {
        {
            use $crate::regex::Regex;

            let memory_access_re = Regex::new(r"BIT .,\(HL\)").unwrap();

            let mut prefix_instructions = [
                $(
                    {
                        // If the instruction accesses memory (through HL), the instruction will
                        // take 16 cycles. Otherwise, it will take 8.
                        let cycles = if memory_access_re.is_match($description) {
                            12
                        } else if $description.contains("(HL)") {
                            16
                        } else {
                            8
                        };

                        PrefixInstructionDef {
                            byte: $byte,
                            description: $description,
                            cycles,
                        }
                    }
                ),*
            ];
            prefix_instructions.sort_unstable_by_key(|instruction| instruction.byte);
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

impl Cpu {
    pub fn execute_prefix(&mut self, instruction: u8, bus: &mut Bus) {
        match instruction {
            // RLC n
            0x00 => arithmetic::rlc(&mut self.reg.b, &mut self.reg.f),
            0x01 => arithmetic::rlc(&mut self.reg.c, &mut self.reg.f),
            0x02 => arithmetic::rlc(&mut self.reg.d, &mut self.reg.f),
            0x03 => arithmetic::rlc(&mut self.reg.e, &mut self.reg.f),
            0x04 => arithmetic::rlc(&mut self.reg.h, &mut self.reg.f),
            0x05 => arithmetic::rlc(&mut self.reg.l, &mut self.reg.f),
            0x06 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::rlc(&mut byte, &mut self.reg.f);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x07 => arithmetic::rlc(&mut self.reg.a, &mut self.reg.f),

            // RRC n
            0x08 => arithmetic::rrc(&mut self.reg.b, &mut self.reg.f),
            0x09 => arithmetic::rrc(&mut self.reg.c, &mut self.reg.f),
            0x0a => arithmetic::rrc(&mut self.reg.d, &mut self.reg.f),
            0x0b => arithmetic::rrc(&mut self.reg.e, &mut self.reg.f),
            0x0c => arithmetic::rrc(&mut self.reg.h, &mut self.reg.f),
            0x0d => arithmetic::rrc(&mut self.reg.l, &mut self.reg.f),
            0x0e => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::rrc(&mut byte, &mut self.reg.f);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x0f => arithmetic::rrc(&mut self.reg.a, &mut self.reg.f),

            // RL C
            0x10 => arithmetic::rl(&mut self.reg.b, &mut self.reg.f),
            0x11 => arithmetic::rl(&mut self.reg.c, &mut self.reg.f),
            0x12 => arithmetic::rl(&mut self.reg.d, &mut self.reg.f),
            0x13 => arithmetic::rl(&mut self.reg.e, &mut self.reg.f),
            0x14 => arithmetic::rl(&mut self.reg.h, &mut self.reg.f),
            0x15 => arithmetic::rl(&mut self.reg.l, &mut self.reg.f),
            0x16 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::rl(&mut byte, &mut self.reg.f);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x17 => arithmetic::rl(&mut self.reg.a, &mut self.reg.f),

            // RR n
            0x18 => arithmetic::rr(&mut self.reg.b, &mut self.reg.f),
            0x19 => arithmetic::rr(&mut self.reg.c, &mut self.reg.f),
            0x1a => arithmetic::rr(&mut self.reg.d, &mut self.reg.f),
            0x1b => arithmetic::rr(&mut self.reg.e, &mut self.reg.f),
            0x1c => arithmetic::rr(&mut self.reg.h, &mut self.reg.f),
            0x1d => arithmetic::rr(&mut self.reg.l, &mut self.reg.f),
            0x1e => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::rr(&mut byte, &mut self.reg.f);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x1f => arithmetic::rr(&mut self.reg.a, &mut self.reg.f),

            // SLA n
            0x20 => arithmetic::sla(&mut self.reg.b, &mut self.reg.f),
            0x21 => arithmetic::sla(&mut self.reg.c, &mut self.reg.f),
            0x22 => arithmetic::sla(&mut self.reg.d, &mut self.reg.f),
            0x23 => arithmetic::sla(&mut self.reg.e, &mut self.reg.f),
            0x24 => arithmetic::sla(&mut self.reg.h, &mut self.reg.f),
            0x25 => arithmetic::sla(&mut self.reg.l, &mut self.reg.f),
            0x26 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::sla(&mut byte, &mut self.reg.f);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x27 => arithmetic::sla(&mut self.reg.a, &mut self.reg.f),

            // SRA n
            0x28 => arithmetic::sra(&mut self.reg.b, &mut self.reg.f),
            0x29 => arithmetic::sra(&mut self.reg.c, &mut self.reg.f),
            0x2a => arithmetic::sra(&mut self.reg.d, &mut self.reg.f),
            0x2b => arithmetic::sra(&mut self.reg.e, &mut self.reg.f),
            0x2c => arithmetic::sra(&mut self.reg.h, &mut self.reg.f),
            0x2d => arithmetic::sra(&mut self.reg.l, &mut self.reg.f),
            0x2e => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::sra(&mut byte, &mut self.reg.f);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x2f => arithmetic::sra(&mut self.reg.a, &mut self.reg.f),

            // SWAP
            0x30 => arithmetic::swap(&mut self.reg.b, &mut self.reg.f),
            0x31 => arithmetic::swap(&mut self.reg.c, &mut self.reg.f),
            0x32 => arithmetic::swap(&mut self.reg.d, &mut self.reg.f),
            0x33 => arithmetic::swap(&mut self.reg.e, &mut self.reg.f),
            0x34 => arithmetic::swap(&mut self.reg.h, &mut self.reg.f),
            0x35 => arithmetic::swap(&mut self.reg.l, &mut self.reg.f),
            0x36 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::swap(&mut byte, &mut self.reg.f);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x37 => arithmetic::swap(&mut self.reg.a, &mut self.reg.f),

            // SRL n
            0x38 => arithmetic::srl(&mut self.reg.b, &mut self.reg.f),
            0x39 => arithmetic::srl(&mut self.reg.c, &mut self.reg.f),
            0x3a => arithmetic::srl(&mut self.reg.d, &mut self.reg.f),
            0x3b => arithmetic::srl(&mut self.reg.e, &mut self.reg.f),
            0x3c => arithmetic::srl(&mut self.reg.h, &mut self.reg.f),
            0x3d => arithmetic::srl(&mut self.reg.l, &mut self.reg.f),
            0x3e => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::srl(&mut byte, &mut self.reg.f);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x3f => arithmetic::srl(&mut self.reg.a, &mut self.reg.f),

            // BIT 0,r
            0x40 => arithmetic::bit(self.reg.b, 0, &mut self.reg.f),
            0x41 => arithmetic::bit(self.reg.c, 0, &mut self.reg.f),
            0x42 => arithmetic::bit(self.reg.d, 0, &mut self.reg.f),
            0x43 => arithmetic::bit(self.reg.e, 0, &mut self.reg.f),
            0x44 => arithmetic::bit(self.reg.h, 0, &mut self.reg.f),
            0x45 => arithmetic::bit(self.reg.l, 0, &mut self.reg.f),
            0x46 => arithmetic::bit(bus.read_byte(self.reg.hl()), 0, &mut self.reg.f),
            0x47 => arithmetic::bit(self.reg.a, 0, &mut self.reg.f),

            // BIT 1,r
            0x48 => arithmetic::bit(self.reg.b, 1, &mut self.reg.f),
            0x49 => arithmetic::bit(self.reg.c, 1, &mut self.reg.f),
            0x4a => arithmetic::bit(self.reg.d, 1, &mut self.reg.f),
            0x4b => arithmetic::bit(self.reg.e, 1, &mut self.reg.f),
            0x4c => arithmetic::bit(self.reg.h, 1, &mut self.reg.f),
            0x4d => arithmetic::bit(self.reg.l, 1, &mut self.reg.f),
            0x4e => arithmetic::bit(bus.read_byte(self.reg.hl()), 1, &mut self.reg.f),
            0x4f => arithmetic::bit(self.reg.a, 1, &mut self.reg.f),

            // BIT 2,r
            0x50 => arithmetic::bit(self.reg.b, 2, &mut self.reg.f),
            0x51 => arithmetic::bit(self.reg.c, 2, &mut self.reg.f),
            0x52 => arithmetic::bit(self.reg.d, 2, &mut self.reg.f),
            0x53 => arithmetic::bit(self.reg.e, 2, &mut self.reg.f),
            0x54 => arithmetic::bit(self.reg.h, 2, &mut self.reg.f),
            0x55 => arithmetic::bit(self.reg.l, 2, &mut self.reg.f),
            0x56 => arithmetic::bit(bus.read_byte(self.reg.hl()), 2, &mut self.reg.f),
            0x57 => arithmetic::bit(self.reg.a, 2, &mut self.reg.f),

            // BIT 3,r
            0x58 => arithmetic::bit(self.reg.b, 3, &mut self.reg.f),
            0x59 => arithmetic::bit(self.reg.c, 3, &mut self.reg.f),
            0x5a => arithmetic::bit(self.reg.d, 3, &mut self.reg.f),
            0x5b => arithmetic::bit(self.reg.e, 3, &mut self.reg.f),
            0x5c => arithmetic::bit(self.reg.h, 3, &mut self.reg.f),
            0x5d => arithmetic::bit(self.reg.l, 3, &mut self.reg.f),
            0x5e => arithmetic::bit(bus.read_byte(self.reg.hl()), 3, &mut self.reg.f),
            0x5f => arithmetic::bit(self.reg.a, 3, &mut self.reg.f),

            // BIT 4,r
            0x60 => arithmetic::bit(self.reg.b, 4, &mut self.reg.f),
            0x61 => arithmetic::bit(self.reg.c, 4, &mut self.reg.f),
            0x62 => arithmetic::bit(self.reg.d, 4, &mut self.reg.f),
            0x63 => arithmetic::bit(self.reg.e, 4, &mut self.reg.f),
            0x64 => arithmetic::bit(self.reg.h, 4, &mut self.reg.f),
            0x65 => arithmetic::bit(self.reg.l, 4, &mut self.reg.f),
            0x66 => arithmetic::bit(bus.read_byte(self.reg.hl()), 4, &mut self.reg.f),
            0x67 => arithmetic::bit(self.reg.a, 4, &mut self.reg.f),

            // BIT 5,r
            0x68 => arithmetic::bit(self.reg.b, 5, &mut self.reg.f),
            0x69 => arithmetic::bit(self.reg.c, 5, &mut self.reg.f),
            0x6a => arithmetic::bit(self.reg.d, 5, &mut self.reg.f),
            0x6b => arithmetic::bit(self.reg.e, 5, &mut self.reg.f),
            0x6c => arithmetic::bit(self.reg.h, 5, &mut self.reg.f),
            0x6d => arithmetic::bit(self.reg.l, 5, &mut self.reg.f),
            0x6e => arithmetic::bit(bus.read_byte(self.reg.hl()), 5, &mut self.reg.f),
            0x6f => arithmetic::bit(self.reg.a, 5, &mut self.reg.f),

            // BIT 6,r
            0x70 => arithmetic::bit(self.reg.b, 6, &mut self.reg.f),
            0x71 => arithmetic::bit(self.reg.c, 6, &mut self.reg.f),
            0x72 => arithmetic::bit(self.reg.d, 6, &mut self.reg.f),
            0x73 => arithmetic::bit(self.reg.e, 6, &mut self.reg.f),
            0x74 => arithmetic::bit(self.reg.h, 6, &mut self.reg.f),
            0x75 => arithmetic::bit(self.reg.l, 6, &mut self.reg.f),
            0x76 => arithmetic::bit(bus.read_byte(self.reg.hl()), 6, &mut self.reg.f),
            0x77 => arithmetic::bit(self.reg.a, 6, &mut self.reg.f),

            // BIT 7,r
            0x78 => arithmetic::bit(self.reg.b, 7, &mut self.reg.f),
            0x79 => arithmetic::bit(self.reg.c, 7, &mut self.reg.f),
            0x7a => arithmetic::bit(self.reg.d, 7, &mut self.reg.f),
            0x7b => arithmetic::bit(self.reg.e, 7, &mut self.reg.f),
            0x7c => arithmetic::bit(self.reg.h, 7, &mut self.reg.f),
            0x7d => arithmetic::bit(self.reg.l, 7, &mut self.reg.f),
            0x7e => arithmetic::bit(bus.read_byte(self.reg.hl()), 7, &mut self.reg.f),
            0x7f => arithmetic::bit(self.reg.a, 7, &mut self.reg.f),

            // RES 0,r
            0x80 => arithmetic::res(&mut self.reg.b, 0),
            0x81 => arithmetic::res(&mut self.reg.c, 0),
            0x82 => arithmetic::res(&mut self.reg.d, 0),
            0x83 => arithmetic::res(&mut self.reg.e, 0),
            0x84 => arithmetic::res(&mut self.reg.h, 0),
            0x85 => arithmetic::res(&mut self.reg.l, 0),
            0x86 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::res(&mut byte, 0);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x87 => arithmetic::res(&mut self.reg.a, 0),

            // RES 1,r
            0x88 => arithmetic::res(&mut self.reg.b, 1),
            0x89 => arithmetic::res(&mut self.reg.c, 1),
            0x8a => arithmetic::res(&mut self.reg.d, 1),
            0x8b => arithmetic::res(&mut self.reg.e, 1),
            0x8c => arithmetic::res(&mut self.reg.h, 1),
            0x8d => arithmetic::res(&mut self.reg.l, 1),
            0x8e => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::res(&mut byte, 1);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x8f => arithmetic::res(&mut self.reg.a, 1),

            // RES 2,r
            0x90 => arithmetic::res(&mut self.reg.b, 2),
            0x91 => arithmetic::res(&mut self.reg.c, 2),
            0x92 => arithmetic::res(&mut self.reg.d, 2),
            0x93 => arithmetic::res(&mut self.reg.e, 2),
            0x94 => arithmetic::res(&mut self.reg.h, 2),
            0x95 => arithmetic::res(&mut self.reg.l, 2),
            0x96 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::res(&mut byte, 2);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x97 => arithmetic::res(&mut self.reg.a, 2),

            // RES 3,r
            0x98 => arithmetic::res(&mut self.reg.b, 3),
            0x99 => arithmetic::res(&mut self.reg.c, 3),
            0x9a => arithmetic::res(&mut self.reg.d, 3),
            0x9b => arithmetic::res(&mut self.reg.e, 3),
            0x9c => arithmetic::res(&mut self.reg.h, 3),
            0x9d => arithmetic::res(&mut self.reg.l, 3),
            0x9e => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::res(&mut byte, 3);
                bus.write_byte(self.reg.hl(), byte);
            }
            0x9f => arithmetic::res(&mut self.reg.a, 3),

            // RES 4,r
            0xa0 => arithmetic::res(&mut self.reg.b, 4),
            0xa1 => arithmetic::res(&mut self.reg.c, 4),
            0xa2 => arithmetic::res(&mut self.reg.d, 4),
            0xa3 => arithmetic::res(&mut self.reg.e, 4),
            0xa4 => arithmetic::res(&mut self.reg.h, 4),
            0xa5 => arithmetic::res(&mut self.reg.l, 4),
            0xa6 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::res(&mut byte, 4);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xa7 => arithmetic::res(&mut self.reg.a, 4),

            // RES 5,r
            0xa8 => arithmetic::res(&mut self.reg.b, 5),
            0xa9 => arithmetic::res(&mut self.reg.c, 5),
            0xaa => arithmetic::res(&mut self.reg.d, 5),
            0xab => arithmetic::res(&mut self.reg.e, 5),
            0xac => arithmetic::res(&mut self.reg.h, 5),
            0xad => arithmetic::res(&mut self.reg.l, 5),
            0xae => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::res(&mut byte, 5);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xaf => arithmetic::res(&mut self.reg.a, 5),

            // RES 6,r
            0xb0 => arithmetic::res(&mut self.reg.b, 6),
            0xb1 => arithmetic::res(&mut self.reg.c, 6),
            0xb2 => arithmetic::res(&mut self.reg.d, 6),
            0xb3 => arithmetic::res(&mut self.reg.e, 6),
            0xb4 => arithmetic::res(&mut self.reg.h, 6),
            0xb5 => arithmetic::res(&mut self.reg.l, 6),
            0xb6 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::res(&mut byte, 6);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xb7 => arithmetic::res(&mut self.reg.a, 6),

            // RES 7,r
            0xb8 => arithmetic::res(&mut self.reg.b, 7),
            0xb9 => arithmetic::res(&mut self.reg.c, 7),
            0xba => arithmetic::res(&mut self.reg.d, 7),
            0xbb => arithmetic::res(&mut self.reg.e, 7),
            0xbc => arithmetic::res(&mut self.reg.h, 7),
            0xbd => arithmetic::res(&mut self.reg.l, 7),
            0xbe => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::res(&mut byte, 7);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xbf => arithmetic::res(&mut self.reg.a, 7),

            // SET 0,r
            0xc0 => arithmetic::set(&mut self.reg.b, 0),
            0xc1 => arithmetic::set(&mut self.reg.c, 0),
            0xc2 => arithmetic::set(&mut self.reg.d, 0),
            0xc3 => arithmetic::set(&mut self.reg.e, 0),
            0xc4 => arithmetic::set(&mut self.reg.h, 0),
            0xc5 => arithmetic::set(&mut self.reg.l, 0),
            0xc6 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::set(&mut byte, 0);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xc7 => arithmetic::set(&mut self.reg.a, 0),

            // SET 1,r
            0xc8 => arithmetic::set(&mut self.reg.b, 1),
            0xc9 => arithmetic::set(&mut self.reg.c, 1),
            0xca => arithmetic::set(&mut self.reg.d, 1),
            0xcb => arithmetic::set(&mut self.reg.e, 1),
            0xcc => arithmetic::set(&mut self.reg.h, 1),
            0xcd => arithmetic::set(&mut self.reg.l, 1),
            0xce => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::set(&mut byte, 1);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xcf => arithmetic::set(&mut self.reg.a, 1),

            // SET 2,r
            0xd0 => arithmetic::set(&mut self.reg.b, 2),
            0xd1 => arithmetic::set(&mut self.reg.c, 2),
            0xd2 => arithmetic::set(&mut self.reg.d, 2),
            0xd3 => arithmetic::set(&mut self.reg.e, 2),
            0xd4 => arithmetic::set(&mut self.reg.h, 2),
            0xd5 => arithmetic::set(&mut self.reg.l, 2),
            0xd6 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::set(&mut byte, 2);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xd7 => arithmetic::set(&mut self.reg.a, 2),

            // SET 3,r
            0xd8 => arithmetic::set(&mut self.reg.b, 3),
            0xd9 => arithmetic::set(&mut self.reg.c, 3),
            0xda => arithmetic::set(&mut self.reg.d, 3),
            0xdb => arithmetic::set(&mut self.reg.e, 3),
            0xdc => arithmetic::set(&mut self.reg.h, 3),
            0xdd => arithmetic::set(&mut self.reg.l, 3),
            0xde => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::set(&mut byte, 3);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xdf => arithmetic::set(&mut self.reg.a, 3),

            // SET 4,r
            0xe0 => arithmetic::set(&mut self.reg.b, 4),
            0xe1 => arithmetic::set(&mut self.reg.c, 4),
            0xe2 => arithmetic::set(&mut self.reg.d, 4),
            0xe3 => arithmetic::set(&mut self.reg.e, 4),
            0xe4 => arithmetic::set(&mut self.reg.h, 4),
            0xe5 => arithmetic::set(&mut self.reg.l, 4),
            0xe6 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::set(&mut byte, 4);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xe7 => arithmetic::set(&mut self.reg.a, 4),

            // SET 5,r
            0xe8 => arithmetic::set(&mut self.reg.b, 5),
            0xe9 => arithmetic::set(&mut self.reg.c, 5),
            0xea => arithmetic::set(&mut self.reg.d, 5),
            0xeb => arithmetic::set(&mut self.reg.e, 5),
            0xec => arithmetic::set(&mut self.reg.h, 5),
            0xed => arithmetic::set(&mut self.reg.l, 5),
            0xee => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::set(&mut byte, 5);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xef => arithmetic::set(&mut self.reg.a, 5),

            // SET 6,r
            0xf0 => arithmetic::set(&mut self.reg.b, 6),
            0xf1 => arithmetic::set(&mut self.reg.c, 6),
            0xf2 => arithmetic::set(&mut self.reg.d, 6),
            0xf3 => arithmetic::set(&mut self.reg.e, 6),
            0xf4 => arithmetic::set(&mut self.reg.h, 6),
            0xf5 => arithmetic::set(&mut self.reg.l, 6),
            0xf6 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::set(&mut byte, 6);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xf7 => arithmetic::set(&mut self.reg.a, 6),

            // SET 7,r
            0xf8 => arithmetic::set(&mut self.reg.b, 7),
            0xf9 => arithmetic::set(&mut self.reg.c, 7),
            0xfa => arithmetic::set(&mut self.reg.d, 7),
            0xfb => arithmetic::set(&mut self.reg.e, 7),
            0xfc => arithmetic::set(&mut self.reg.h, 7),
            0xfd => arithmetic::set(&mut self.reg.l, 7),
            0xfe => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::set(&mut byte, 7);
                bus.write_byte(self.reg.hl(), byte);
            }
            0xff => arithmetic::set(&mut self.reg.a, 7),

            // error
            catch => {
                panic!(
                    "unimplemented prefix instruction {:#0x} at {:#0x}",
                    catch,
                    self.reg.pc + 1
                )
            }
        }
    }
}

lazy_static! {
    /// Prefix instruction definitions.
    ///
    /// Descriptions taken from [here].
    ///
    /// [here]: http://pastraiser.com/cpu/gameboy/gameboy_opcodes.html
    pub static ref PREFIX_INSTRUCTIONS: [PrefixInstructionDef; 0x100] = prefix_instructions! {
        // byte     description
        0x00,       "RLC B";
        0x01,       "RLC C";
        0x02,       "RLC D";
        0x03,       "RLC E";
        0x04,       "RLC H";
        0x05,       "RLC L";
        0x06,       "RLC (HL)";
        0x07,       "RLC A";
        0x08,       "RRC B";
        0x09,       "RRC C";
        0x0a,       "RRC D";
        0x0b,       "RRC E";
        0x0c,       "RRC H";
        0x0d,       "RRC L";
        0x0e,       "RRC (HL)";
        0x0f,       "RRC A";
        0x10,       "RL B";
        0x11,       "RL C";
        0x12,       "RL D";
        0x13,       "RL E";
        0x14,       "RL H";
        0x15,       "RL L";
        0x16,       "RL (HL)";
        0x17,       "RL A";
        0x18,       "RR B";
        0x19,       "RR C";
        0x1a,       "RR D";
        0x1b,       "RR E";
        0x1c,       "RR H";
        0x1d,       "RR L";
        0x1e,       "RR (HL)";
        0x1f,       "RR A";
        0x20,       "SLA B";
        0x21,       "SLA C";
        0x22,       "SLA D";
        0x23,       "SLA E";
        0x24,       "SLA H";
        0x25,       "SLA L";
        0x26,       "SLA (HL)";
        0x27,       "SLA A";
        0x28,       "SRA B";
        0x29,       "SRA C";
        0x2a,       "SRA D";
        0x2b,       "SRA E";
        0x2c,       "SRA H";
        0x2d,       "SRA L";
        0x2e,       "SRA (HL)";
        0x2f,       "SRA A";
        0x30,       "SWAP B";
        0x31,       "SWAP C";
        0x32,       "SWAP D";
        0x33,       "SWAP E";
        0x34,       "SWAP H";
        0x35,       "SWAP L";
        0x36,       "SWAP (HL)";
        0x37,       "SWAP A";
        0x38,       "SRL B";
        0x39,       "SRL C";
        0x3a,       "SRL D";
        0x3b,       "SRL E";
        0x3c,       "SRL H";
        0x3d,       "SRL L";
        0x3e,       "SRL (HL)";
        0x3f,       "SRL A";
        0x40,       "BIT 0,B";
        0x41,       "BIT 0,C";
        0x42,       "BIT 0,D";
        0x43,       "BIT 0,E";
        0x44,       "BIT 0,H";
        0x45,       "BIT 0,L";
        0x46,       "BIT 0,(HL)";
        0x47,       "BIT 0,A";
        0x48,       "BIT 1,B";
        0x49,       "BIT 1,C";
        0x4a,       "BIT 1,D";
        0x4b,       "BIT 1,E";
        0x4c,       "BIT 1,H";
        0x4d,       "BIT 1,L";
        0x4e,       "BIT 1,(HL)";
        0x4f,       "BIT 1,A";
        0x50,       "BIT 2,B";
        0x51,       "BIT 2,C";
        0x52,       "BIT 2,D";
        0x53,       "BIT 2,E";
        0x54,       "BIT 2,H";
        0x55,       "BIT 2,L";
        0x56,       "BIT 2,(HL)";
        0x57,       "BIT 2,A";
        0x58,       "BIT 3,B";
        0x59,       "BIT 3,C";
        0x5a,       "BIT 3,D";
        0x5b,       "BIT 3,E";
        0x5c,       "BIT 3,H";
        0x5d,       "BIT 3,L";
        0x5e,       "BIT 3,(HL)";
        0x5f,       "BIT 3,A";
        0x60,       "BIT 4,B";
        0x61,       "BIT 4,C";
        0x62,       "BIT 4,D";
        0x63,       "BIT 4,E";
        0x64,       "BIT 4,H";
        0x65,       "BIT 4,L";
        0x66,       "BIT 4,(HL)";
        0x67,       "BIT 4,A";
        0x68,       "BIT 5,B";
        0x69,       "BIT 5,C";
        0x6a,       "BIT 5,D";
        0x6b,       "BIT 5,E";
        0x6c,       "BIT 5,H";
        0x6d,       "BIT 5,L";
        0x6e,       "BIT 5,(HL)";
        0x6f,       "BIT 5,A";
        0x70,       "BIT 6,B";
        0x71,       "BIT 6,C";
        0x72,       "BIT 6,D";
        0x73,       "BIT 6,E";
        0x74,       "BIT 6,H";
        0x75,       "BIT 6,L";
        0x76,       "BIT 6,(HL)";
        0x77,       "BIT 6,A";
        0x78,       "BIT 7,B";
        0x79,       "BIT 7,C";
        0x7a,       "BIT 7,D";
        0x7b,       "BIT 7,E";
        0x7c,       "BIT 7,H";
        0x7d,       "BIT 7,L";
        0x7e,       "BIT 7,(HL)";
        0x7f,       "BIT 7,A";
        0x80,       "RES 0,B";
        0x81,       "RES 0,C";
        0x82,       "RES 0,D";
        0x83,       "RES 0,E";
        0x84,       "RES 0,H";
        0x85,       "RES 0,L";
        0x86,       "RES 0,(HL)";
        0x87,       "RES 0,A";
        0x88,       "RES 1,B";
        0x89,       "RES 1,C";
        0x8a,       "RES 1,D";
        0x8b,       "RES 1,E";
        0x8c,       "RES 1,H";
        0x8d,       "RES 1,L";
        0x8e,       "RES 1,(HL)";
        0x8f,       "RES 1,A";
        0x90,       "RES 2,B";
        0x91,       "RES 2,C";
        0x92,       "RES 2,D";
        0x93,       "RES 2,E";
        0x94,       "RES 2,H";
        0x95,       "RES 2,L";
        0x96,       "RES 2,(HL)";
        0x97,       "RES 2,A";
        0x98,       "RES 3,B";
        0x99,       "RES 3,C";
        0x9a,       "RES 3,D";
        0x9b,       "RES 3,E";
        0x9c,       "RES 3,H";
        0x9d,       "RES 3,L";
        0x9e,       "RES 3,(HL)";
        0x9f,       "RES 3,A";
        0xa0,       "RES 4,B";
        0xa1,       "RES 4,C";
        0xa2,       "RES 4,D";
        0xa3,       "RES 4,E";
        0xa4,       "RES 4,H";
        0xa5,       "RES 4,L";
        0xa6,       "RES 4,(HL)";
        0xa7,       "RES 4,A";
        0xa8,       "RES 5,B";
        0xa9,       "RES 5,C";
        0xaa,       "RES 5,D";
        0xab,       "RES 5,E";
        0xac,       "RES 5,H";
        0xad,       "RES 5,L";
        0xae,       "RES 5,(HL)";
        0xaf,       "RES 5,A";
        0xb0,       "RES 6,B";
        0xb1,       "RES 6,C";
        0xb2,       "RES 6,D";
        0xb3,       "RES 6,E";
        0xb4,       "RES 6,H";
        0xb5,       "RES 6,L";
        0xb6,       "RES 6,(HL)";
        0xb7,       "RES 6,A";
        0xb8,       "RES 7,B";
        0xb9,       "RES 7,C";
        0xba,       "RES 7,D";
        0xbb,       "RES 7,E";
        0xbc,       "RES 7,H";
        0xbd,       "RES 7,L";
        0xbe,       "RES 7,(HL)";
        0xbf,       "RES 7,A";
        0xc0,       "SET 0,B";
        0xc1,       "SET 0,C";
        0xc2,       "SET 0,D";
        0xc3,       "SET 0,E";
        0xc4,       "SET 0,H";
        0xc5,       "SET 0,L";
        0xc6,       "SET 0,(HL)";
        0xc7,       "SET 0,A";
        0xc8,       "SET 1,B";
        0xc9,       "SET 1,C";
        0xca,       "SET 1,D";
        0xcb,       "SET 1,E";
        0xcc,       "SET 1,H";
        0xcd,       "SET 1,L";
        0xce,       "SET 1,(HL)";
        0xcf,       "SET 1,A";
        0xd0,       "SET 2,B";
        0xd1,       "SET 2,C";
        0xd2,       "SET 2,D";
        0xd3,       "SET 2,E";
        0xd4,       "SET 2,H";
        0xd5,       "SET 2,L";
        0xd6,       "SET 2,(HL)";
        0xd7,       "SET 2,A";
        0xd8,       "SET 3,B";
        0xd9,       "SET 3,C";
        0xda,       "SET 3,D";
        0xdb,       "SET 3,E";
        0xdc,       "SET 3,H";
        0xdd,       "SET 3,L";
        0xde,       "SET 3,(HL)";
        0xdf,       "SET 3,A";
        0xe0,       "SET 4,B";
        0xe1,       "SET 4,C";
        0xe2,       "SET 4,D";
        0xe3,       "SET 4,E";
        0xe4,       "SET 4,H";
        0xe5,       "SET 4,L";
        0xe6,       "SET 4,(HL)";
        0xe7,       "SET 4,A";
        0xe8,       "SET 5,B";
        0xe9,       "SET 5,C";
        0xea,       "SET 5,D";
        0xeb,       "SET 5,E";
        0xec,       "SET 5,H";
        0xed,       "SET 5,L";
        0xee,       "SET 5,(HL)";
        0xef,       "SET 5,A";
        0xf0,       "SET 6,B";
        0xf1,       "SET 6,C";
        0xf2,       "SET 6,D";
        0xf3,       "SET 6,E";
        0xf4,       "SET 6,H";
        0xf5,       "SET 6,L";
        0xf6,       "SET 6,(HL)";
        0xf7,       "SET 6,A";
        0xf8,       "SET 7,B";
        0xf9,       "SET 7,C";
        0xfa,       "SET 7,D";
        0xfb,       "SET 7,E";
        0xfc,       "SET 7,H";
        0xfd,       "SET 7,L";
        0xfe,       "SET 7,(HL)";
        0xff,       "SET 7,A";
    };
}

#[cfg(test)]
mod tests {
    use super::PREFIX_INSTRUCTIONS;

    #[test]
    fn timings() {
        // These timings taken from blargg's instruction timing test ROM.
        let timings: Vec<u8> = vec![
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
            2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
            2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
            2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
            2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        ];

        for (timing, instruction) in timings.iter().zip(PREFIX_INSTRUCTIONS.iter()) {
            let clock_cycles = timing * 4;

            if clock_cycles != instruction.cycles {
                panic!("wrong timing for {:?}: has {}, expected {}",
                       instruction.description, instruction.cycles, clock_cycles);
            }
        }
    }
}
