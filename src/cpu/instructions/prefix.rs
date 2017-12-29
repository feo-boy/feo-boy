use bus::Bus;
use cpu::{arithmetic, Cpu};
use memory::Addressable;

/// Prefix instruction definitions.
pub(super) static PREFIX_INSTRUCTIONS: [PrefixInstructionDef; 0x100] =
    include!(concat!(env!("OUT_DIR"), "/prefix_instructions.rs"));

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
            catch => panic!(
                "unimplemented prefix instruction {:#0x} at {:#0x}",
                catch,
                self.reg.pc + 1
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PREFIX_INSTRUCTIONS;

    #[test]
    fn timings() {
        // These timings taken from blargg's instruction timing test ROM.
        #[cfg_attr(rustfmt, rustfmt_skip)]
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
                panic!(
                    "wrong timing for {:?}: has {}, expected {}",
                    instruction.description, instruction.cycles, clock_cycles
                );
            }
        }
    }
}
