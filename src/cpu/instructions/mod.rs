//! CPU instruction pipeline implementation.
//!
//! The definitions of each instruction are generated at compile time in `build.rs`.
//!
//! Information concerning instruction definitions and implementations can be found on [devrs].
//!
//! [devrs]: http://www.devrs.com/gb/files/opcodes.html

use std::fmt::{self, Display};
use std::ops::{AddAssign, SubAssign};

use byteorder::{ByteOrder, LittleEndian};
use lazy_static::lazy_static;
use log::*;
use regex::{NoExpand, Regex};
use smallvec::SmallVec;

use crate::bus::Bus;
use crate::bytes::WordExt;
use crate::cpu::{arithmetic, Flags, MCycles, State, TCycles};

mod prefix;
use crate::cpu::instructions::prefix::PREFIX_INSTRUCTIONS;

/// Game Boy instruction set.
static INSTRUCTIONS: [InstructionDef; 0x100] =
    include!(concat!(env!("OUT_DIR"), "/instructions.rs"));

lazy_static! {
    /// Matches instruction descriptions that take operands.
    static ref DATA_RE: Regex = Regex::new("d8|d16|a8|a16|r8|PREFIX CB").unwrap();
}

/// A definition of a single instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
struct InstructionDef {
    /// The byte that identifies this instruction.
    pub byte: u8,

    /// A short, human readable representation of the instruction in Z80 assembly syntax.
    pub description: &'static str,

    /// The number of clock cycles it takes to execute this instruction.
    ///
    /// Prefix instructions have a cycle count of 0 in this representation. Use the `cycles()`
    /// method of `Instruction` to get the correct cycle count.
    ///
    /// Note that this is measured in clock cycles, not machine cycles. While Nintendo's official
    /// documentation records the timings in machine cycles, most online documentation uses clock
    /// cycles. Four clock cycles is equivalent to a single machine cycle.
    cycles: TCycles,

    /// The number of clock cycles it takes to execute this instruction if the instruction's
    /// condition is true. Only conditional instructions will have this field set.
    ///
    /// For example, `RET Z` will only return if the zero flag is set. In that case, it will
    /// execute in 20 cycles instead of 8.
    pub condition_cycles: Option<TCycles>,

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

impl Instruction {
    /// The number of clock cycles it takes to execute this instruction.
    pub fn cycles(&self) -> TCycles {
        if self.def.byte == 0xCB {
            PREFIX_INSTRUCTIONS[self.operands[0] as usize].cycles
        } else {
            self.def.cycles
        }
    }
}

impl Default for Instruction {
    /// Returns the `NOP` instruction.
    fn default() -> Instruction {
        Instruction {
            def: &INSTRUCTIONS[0x00],
            operands: Default::default(),
        }
    }
}

impl Display for Instruction {
    /// Prints the instruction in assembly syntax, including the operands.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instruction = if let Some(mat) = DATA_RE.find(self.def.description) {
            let replacement = match mat.as_str() {
                "d8" | "a8" | "r8" => format!("${:#04x}", &self.operands[0]),
                "d16" | "a16" => format!("${:#06x}", LittleEndian::read_u16(&self.operands)),
                "PREFIX CB" => {
                    let opcode = self.operands[0] as usize;
                    PREFIX_INSTRUCTIONS[opcode].description.to_owned()
                }
                ty => unreachable!("unhandled data type: {}", ty),
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

impl super::Cpu {
    /// Retrieves the current instruction. Does not consume any cycles.
    pub fn current_instruction(&self, bus: &Bus) -> Instruction {
        let byte = bus.read_byte_no_tick(self.reg.pc);

        let def = &INSTRUCTIONS[byte as usize];

        let operands = (0..def.num_operands)
            .map(|i| bus.read_byte_no_tick(self.reg.pc + 1 + u16::from(i)))
            .collect();

        Instruction { def, operands }
    }

    /// Decodes the next instruction.
    pub fn fetch(&self, bus: &mut Bus) -> Instruction {
        let byte = bus.read_byte(self.reg.pc);

        let def = &INSTRUCTIONS[byte as usize];

        let operands = (0..def.num_operands)
            .map(|i| bus.read_byte(self.reg.pc + 1 + u16::from(i)))
            .collect();

        Instruction { def, operands }
    }

    /// Executes an instruction.
    ///
    /// All necessary side effects are performed, including updating the program counter, flag
    /// registers, and CPU clock.
    pub fn execute(&mut self, instruction: &Instruction, bus: &mut Bus) {
        debug!("executing {:#06x} {}", self.reg.pc, instruction.to_string());
        trace!("{:?}", instruction);

        // Check that we have exactly as many operands as the instruction requires.
        debug_assert_eq!(
            instruction.def.num_operands as usize,
            instruction.operands.len()
        );

        let mut condition_taken = false;

        if !self.halt_bug {
            // Increment the program counter (PC) *before* executing the instruction.
            //
            // This how the actual hardware handles the PC, as relative jumps and other PC-related
            // instructions assume that PC is pointing at the *next* instruction.
            self.reg.pc += 1 + instruction.operands.len() as u16;
        } else {
            self.halt_bug = false;
        }

        // Execute the instruction.
        match instruction.def.byte {
            // NOP
            0x00 => (),

            // STOP
            0x10 => self.state = State::Stopped,

            // JR NZ,r8
            0x20 => {
                if !self.reg.f.contains(Flags::ZERO) {
                    self.jr(instruction.operands[0] as i8);
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // JR NC,r8
            0x30 => {
                if !self.reg.f.contains(Flags::CARRY) {
                    self.jr(instruction.operands[0] as i8);
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // LD B,B
            #[allow(clippy::self_assignment)]
            0x40 => self.reg.b = self.reg.b,

            // LD D,B
            0x50 => self.reg.d = self.reg.b,

            // LD H,B
            0x60 => self.reg.h = self.reg.b,

            // LD (HL),B
            0x70 => bus.write_byte(self.reg.hl(), self.reg.b),

            // ADD A,B
            0x80 => {
                let b = self.reg.b;
                self.reg.add(b);
            }

            // SUB B
            0x90 => {
                let b = self.reg.b;
                self.reg.sub(b);
            }

            // AND B
            0xa0 => {
                let b = self.reg.b;
                self.reg.and(b);
            }

            // OR B
            0xb0 => {
                let b = self.reg.b;
                self.reg.or(b);
            }

            // RET NZ
            0xc0 => {
                // Internal delay
                bus.tick(MCycles(1));

                if !self.reg.f.contains(Flags::ZERO) {
                    self.ret(bus);
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // RET NC
            0xd0 => {
                // Internal delay
                bus.tick(MCycles(1));

                if !self.reg.f.contains(Flags::CARRY) {
                    self.ret(bus);
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // LDH (a8),A
            0xe0 => {
                let address = 0xff00u16 + &instruction.operands[0].into();
                bus.write_byte(address, self.reg.a)
            }

            // LDH A,(a8)
            0xf0 => {
                let address = 0xff00u16 + &instruction.operands[0].into();
                self.reg.a = bus.read_byte(address);
            }

            // LD BC,d16
            0x01 => self
                .reg
                .bc_mut()
                .write(LittleEndian::read_u16(&instruction.operands)),

            // LD DE,d16
            0x11 => self
                .reg
                .de_mut()
                .write(LittleEndian::read_u16(&instruction.operands)),

            // LD HL,d16
            0x21 => self
                .reg
                .hl_mut()
                .write(LittleEndian::read_u16(&instruction.operands)),

            // LD SP,d16
            0x31 => self.reg.sp = LittleEndian::read_u16(&instruction.operands),

            // LD B,C
            0x41 => self.reg.b = self.reg.c,

            // LD D,C
            0x51 => self.reg.d = self.reg.c,

            // LD H,C
            0x61 => self.reg.h = self.reg.c,

            // LD (HL),C
            0x71 => bus.write_byte(self.reg.hl(), self.reg.c),

            // ADD A,C
            0x81 => {
                let c = self.reg.c;
                self.reg.add(c);
            }

            // SUB C
            0x91 => {
                let c = self.reg.c;
                self.reg.sub(c);
            }

            // AND C
            0xa1 => {
                let c = self.reg.c;
                self.reg.and(c);
            }

            // OR C
            0xb1 => {
                let c = self.reg.c;
                self.reg.or(c);
            }

            // POP BC
            0xc1 => {
                let bc = self.pop(bus);
                self.reg.bc_mut().write(bc);
            }

            // POP DE
            0xd1 => {
                let de = self.pop(bus);
                self.reg.de_mut().write(de);
            }

            // POP HL
            0xe1 => {
                let hl = self.pop(bus);
                self.reg.hl_mut().write(hl);
            }

            // POP AF
            0xf1 => {
                let af = self.pop(bus);
                self.reg.a = af.hi();
                self.reg.f = Flags::from_bits_truncate(af.lo());
            }

            // LD (BC),A
            0x02 => bus.write_byte(self.reg.bc(), self.reg.a),

            // LD (DE),A
            0x12 => bus.write_byte(self.reg.de(), self.reg.a),

            // LD (HL+),A
            0x22 => {
                bus.write_byte(self.reg.hl(), self.reg.a);
                self.reg.hl_mut().add_assign(1);
            }

            // LD (HL-),A
            0x32 => {
                bus.write_byte(self.reg.hl(), self.reg.a);
                self.reg.hl_mut().sub_assign(1);
            }

            // LD B,D
            0x42 => self.reg.b = self.reg.d,

            // LD D,D
            #[allow(clippy::self_assignment)]
            0x52 => self.reg.d = self.reg.d,

            // LD H,D
            0x62 => self.reg.h = self.reg.d,

            // LD (HL),D
            0x72 => bus.write_byte(self.reg.hl(), self.reg.d),

            // ADD A,D
            0x82 => {
                let d = self.reg.d;
                self.reg.add(d);
            }

            // SUB D
            0x92 => {
                let d = self.reg.d;
                self.reg.sub(d);
            }

            // AND D
            0xa2 => {
                let d = self.reg.d;
                self.reg.and(d);
            }

            // OR D
            0xb2 => {
                let d = self.reg.d;
                self.reg.or(d);
            }

            // JP NZ,a16
            0xc2 => {
                if !self.reg.f.contains(Flags::ZERO) {
                    self.reg.pc = LittleEndian::read_u16(&instruction.operands);
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // JP NC,a16
            0xd2 => {
                if !self.reg.f.contains(Flags::CARRY) {
                    self.reg.pc = LittleEndian::read_u16(&instruction.operands);
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // LD (C),A
            // LD ($FF00+C),A
            0xe2 => {
                let address = 0xff00u16 + &self.reg.c.into();
                bus.write_byte(address, self.reg.a);
            }

            // LD A,(C)
            // LD A,($FF00+C)
            0xf2 => {
                let address = 0xff00u16 + &self.reg.c.into();
                self.reg.a = bus.read_byte(address);
            }

            // INC BC
            0x03 => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                self.reg.bc_mut().add_assign(1);
            }

            // INC DE
            0x13 => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                self.reg.de_mut().add_assign(1);
            }

            // INC HL
            0x23 => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                self.reg.hl_mut().add_assign(1);
            }

            // INC SP
            0x33 => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                self.reg.sp = self.reg.sp.wrapping_add(1);
            }

            // LD B,E
            0x43 => self.reg.b = self.reg.e,

            // LD D,E
            0x53 => self.reg.d = self.reg.e,

            // LD H,E
            0x63 => self.reg.h = self.reg.e,

            // LD (HL),E
            0x73 => bus.write_byte(self.reg.hl(), self.reg.e),

            // ADD A,E
            0x83 => {
                let e = self.reg.e;
                self.reg.add(e);
            }

            // SUB E
            0x93 => {
                let e = self.reg.e;
                self.reg.sub(e);
            }

            // AND E
            0xa3 => {
                let e = self.reg.e;
                self.reg.and(e);
            }

            // OR E
            0xb3 => {
                let e = self.reg.e;
                self.reg.or(e);
            }

            // JP a16
            0xc3 => {
                self.reg.pc = LittleEndian::read_u16(&instruction.operands);

                // Internal delay
                bus.tick(MCycles(1));
            }

            // UNUSED
            // 0xd3

            // UNUSED
            // 0xe3

            // DI
            0xF3 => bus.interrupts.enabled = false,

            // INC B
            0x04 => arithmetic::inc(&mut self.reg.b, &mut self.reg.f),

            // INC D
            0x14 => arithmetic::inc(&mut self.reg.d, &mut self.reg.f),

            // INC H
            0x24 => arithmetic::inc(&mut self.reg.h, &mut self.reg.f),

            // INC (HL)
            0x34 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::inc(&mut byte, &mut self.reg.f);
                bus.write_byte(self.reg.hl(), byte);
            }

            // LD B,H
            0x44 => self.reg.b = self.reg.h,

            // LD D,H
            0x54 => self.reg.d = self.reg.h,

            // LD H,H
            #[allow(clippy::self_assignment)]
            0x64 => self.reg.h = self.reg.h,

            // LD (HL),H
            0x74 => bus.write_byte(self.reg.hl(), self.reg.h),

            // ADD A,H
            0x84 => {
                let h = self.reg.h;
                self.reg.add(h);
            }

            // SUB H
            0x94 => {
                let h = self.reg.h;
                self.reg.sub(h);
            }

            // AND H
            0xa4 => {
                let h = self.reg.h;
                self.reg.and(h);
            }

            // OR H
            0xb4 => {
                let h = self.reg.h;
                self.reg.or(h);
            }

            // CALL NZ,a16
            0xc4 => {
                if !self.reg.f.contains(Flags::ZERO) {
                    let address = LittleEndian::read_u16(&instruction.operands);

                    // Internal delay
                    bus.tick(MCycles(1));

                    self.call(address, bus);
                    condition_taken = true;
                }
            }

            // CALL NC,a16
            0xd4 => {
                if !self.reg.f.contains(Flags::CARRY) {
                    let address = LittleEndian::read_u16(&instruction.operands);

                    // Internal delay
                    bus.tick(MCycles(1));

                    self.call(address, bus);
                    condition_taken = true;
                }
            }

            // UNUSED
            // 0xe4

            // UNUSED
            // 0xf4

            // DEC B
            0x05 => arithmetic::dec(&mut self.reg.b, &mut self.reg.f),

            // DEC D
            0x15 => arithmetic::dec(&mut self.reg.d, &mut self.reg.f),

            // DEC H
            0x25 => arithmetic::dec(&mut self.reg.h, &mut self.reg.f),

            // DEC (HL)
            0x35 => {
                let mut byte = bus.read_byte(self.reg.hl());
                arithmetic::dec(&mut byte, &mut self.reg.f);
                bus.write_byte(self.reg.hl(), byte);
            }

            // LD B,L
            0x45 => self.reg.b = self.reg.l,

            // LD D,L
            0x55 => self.reg.d = self.reg.l,

            // LD H,L
            0x65 => self.reg.h = self.reg.l,

            // LD (HL),L
            0x75 => bus.write_byte(self.reg.hl(), self.reg.l),

            // ADD A,L
            0x85 => {
                let l = self.reg.l;
                self.reg.add(l);
            }

            // SUB L
            0x95 => {
                let l = self.reg.l;
                self.reg.sub(l);
            }

            // AND L
            0xa5 => {
                let l = self.reg.l;
                self.reg.and(l);
            }

            // OR L
            0xb5 => {
                let l = self.reg.l;
                self.reg.or(l);
            }

            // PUSH BC
            0xc5 => {
                // Internal delay
                bus.tick(MCycles(1));

                let bc = self.reg.bc();
                self.push(bc, bus);
            }

            // PUSH DE
            0xd5 => {
                // Internal delay
                bus.tick(MCycles(1));

                let de = self.reg.de();
                self.push(de, bus);
            }

            // PUSH HL
            0xe5 => {
                // Internal delay
                bus.tick(MCycles(1));

                let hl = self.reg.hl();
                self.push(hl, bus);
            }

            // PUSH AF
            0xf5 => {
                // Internal delay
                bus.tick(MCycles(1));

                let af = self.reg.af();
                self.push(af, bus);
            }

            // LD B,d8
            0x06 => self.reg.b = instruction.operands[0],

            // LD D,d8
            0x16 => self.reg.d = instruction.operands[0],

            // LD H,d8
            0x26 => self.reg.h = instruction.operands[0],

            // LD (HL),d8
            0x36 => bus.write_byte(self.reg.hl(), instruction.operands[0]),

            // LD B,(HL)
            0x46 => self.reg.b = bus.read_byte(self.reg.hl()),

            // LD D,(HL)
            0x56 => self.reg.d = bus.read_byte(self.reg.hl()),

            // LD H,(HL)
            0x66 => self.reg.h = bus.read_byte(self.reg.hl()),

            // HALT
            // This behavior is documented in the giibiiadvance docs.
            //
            // See https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
            0x76 => {
                #[allow(clippy::if_same_then_else)]
                if bus.interrupts.enabled {
                    // HALT executed normally.
                    self.state = State::Halted;
                } else if !bus.interrupts.pending() {
                    // HALT mode entered, but interrupts aren't serviced.
                    self.state = State::Halted;
                } else {
                    // HALT mode is not entered, and HALT bug occurs.
                    self.halt_bug = true;
                }
            }

            // ADD A,(HL)
            0x86 => {
                let byte = bus.read_byte(self.reg.hl());
                self.reg.add(byte);
            }

            // SUB (HL)
            0x96 => {
                let byte = bus.read_byte(self.reg.hl());
                self.reg.sub(byte);
            }

            // AND (HL)
            0xa6 => {
                let byte = bus.read_byte(self.reg.hl());
                self.reg.and(byte);
            }

            // OR (HL)
            0xb6 => {
                let byte = bus.read_byte(self.reg.hl());
                self.reg.or(byte);
            }

            // ADD A,d8
            0xc6 => self.reg.add(instruction.operands[0]),

            // SUB d8
            0xd6 => self.reg.sub(instruction.operands[0]),

            // AND d8
            0xe6 => self.reg.and(instruction.operands[0]),

            // OR d8
            0xf6 => self.reg.or(instruction.operands[0]),

            // RLCA
            0x07 => self.reg.rlca(),

            // RLA
            0x17 => self.reg.rla(),

            // DAA
            0x27 => self.reg.daa(),

            // SCF
            0x37 => {
                self.reg.f.remove(Flags::SUBTRACT | Flags::HALF_CARRY);
                self.reg.f.insert(Flags::CARRY);
            }

            // LD B,A
            0x47 => self.reg.b = self.reg.a,

            // LD D,A
            0x57 => self.reg.d = self.reg.a,

            // LD H,A
            0x67 => self.reg.h = self.reg.a,

            // LD (HL),A
            0x77 => bus.write_byte(self.reg.hl(), self.reg.a),

            // ADD A,A
            0x87 => {
                let a = self.reg.a;
                self.reg.add(a);
            }

            // SUB A
            0x97 => {
                let a = self.reg.a;
                self.reg.sub(a);
            }

            // AND A
            0xa7 => {
                let a = self.reg.a;
                self.reg.and(a);
            }

            // OR A
            0xb7 => {
                let a = self.reg.a;
                self.reg.or(a);
            }

            // RST 00H
            0xc7 => {
                // Internal delay
                bus.tick(MCycles(1));

                self.rst(0x0000, bus);
            }

            // RST 10H
            0xd7 => {
                // Internal delay
                bus.tick(MCycles(1));

                self.rst(0x0010, bus);
            }

            // RST 20H
            0xe7 => {
                // Internal delay
                bus.tick(MCycles(1));

                self.rst(0x0020, bus);
            }

            // RST 30H
            0xf7 => {
                // Internal delay
                bus.tick(MCycles(1));

                self.rst(0x0030, bus);
            }

            // LD (a16),SP
            0x08 => {
                let address = LittleEndian::read_u16(&instruction.operands);
                bus.write_word(address, self.reg.sp);
            }

            // JR r8
            0x18 => {
                self.jr(instruction.operands[0] as i8);

                // Internal delay
                bus.tick(MCycles(1));
            }

            // JR Z,r8
            0x28 => {
                if self.reg.f.contains(Flags::ZERO) {
                    self.jr(instruction.operands[0] as i8);
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // JR C,r8
            0x38 => {
                if self.reg.f.contains(Flags::CARRY) {
                    self.jr(instruction.operands[0] as i8);
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // LD C,B
            0x48 => self.reg.c = self.reg.b,

            // LD E,B
            0x58 => self.reg.e = self.reg.b,

            // LD L,B
            0x68 => self.reg.l = self.reg.b,

            // LD A,B
            0x78 => self.reg.a = self.reg.b,

            // ADC A,B
            0x88 => {
                let b = self.reg.b;
                self.reg.adc(b);
            }

            // SBC A,B
            0x98 => {
                let b = self.reg.b;
                self.reg.sbc(b);
            }

            // XOR B
            0xa8 => {
                let b = self.reg.b;
                self.reg.xor(b)
            }

            // CP B
            0xb8 => {
                let b = self.reg.b;
                self.reg.cp(b);
            }

            // RET Z
            0xc8 => {
                // Internal delay
                bus.tick(MCycles(1));

                if self.reg.f.contains(Flags::ZERO) {
                    self.ret(bus);
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // RET C
            0xd8 => {
                // Internal delay
                bus.tick(MCycles(1));

                if self.reg.f.contains(Flags::CARRY) {
                    self.ret(bus);
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // ADD SP,r8
            0xe8 => {
                self.reg.add_sp(instruction.operands[0] as i8);

                // Internal delay
                bus.tick(MCycles(2));
            }

            // LD HL,SP+r8
            // LDHL SP,r8
            0xf8 => {
                self.reg.ld_hl_sp_r8(instruction.operands[0] as i8);

                // Internal delay
                bus.tick(MCycles(1));
            }

            // ADD HL,BC
            0x09 => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                let bc = self.reg.bc();
                self.reg.add_hl(bc);
            }

            // ADD HL,DE
            0x19 => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                let de = self.reg.de();
                self.reg.add_hl(de);
            }

            // ADD HL,HL
            0x29 => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                let hl = self.reg.hl();
                self.reg.add_hl(hl);
            }

            // ADD HL,SP
            0x39 => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                let sp = self.reg.sp;
                self.reg.add_hl(sp);
            }

            // LD C,C
            #[allow(clippy::self_assignment)]
            0x49 => self.reg.c = self.reg.c,

            // LD E,C
            0x59 => self.reg.e = self.reg.c,

            // LD L,C
            0x69 => self.reg.l = self.reg.c,

            // LD A,C
            0x79 => self.reg.a = self.reg.c,

            // ADC A,C
            0x89 => {
                let c = self.reg.c;
                self.reg.adc(c);
            }

            // SBC A,C
            0x99 => {
                let c = self.reg.c;
                self.reg.sbc(c);
            }

            // XOR C
            0xa9 => {
                let c = self.reg.c;
                self.reg.xor(c);
            }

            // CP C
            0xb9 => {
                let c = self.reg.c;
                self.reg.cp(c);
            }

            // RET
            0xc9 => {
                self.ret(bus);

                // Internal delay
                bus.tick(MCycles(1));
            }

            // RETI
            0xD9 => {
                self.ret(bus);
                bus.interrupts.enabled = true;

                // Internal delay
                bus.tick(MCycles(1));
            }

            // JP (HL)
            0xe9 => self.reg.pc = self.reg.hl(),

            // LD SP,HL
            0xf9 => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                self.reg.sp = self.reg.hl();
            }

            // LD A,(BC)
            0x0a => self.reg.a = bus.read_byte(self.reg.bc()),

            // LD A,(DE)
            0x1a => self.reg.a = bus.read_byte(self.reg.de()),

            // LD A,(HL+)
            0x2a => {
                self.reg.a = bus.read_byte(self.reg.hl());
                self.reg.hl_mut().add_assign(1);
            }

            // LD A,(HL-)
            0x3a => {
                self.reg.a = bus.read_byte(self.reg.hl());
                self.reg.hl_mut().sub_assign(1);
            }

            // LD C,D
            0x4a => self.reg.c = self.reg.d,

            // LD E,D
            0x5a => self.reg.e = self.reg.d,

            // LD L,D
            0x6a => self.reg.l = self.reg.d,

            // LD A,D
            0x7a => self.reg.a = self.reg.d,

            // ADC A,D
            0x8a => {
                let d = self.reg.d;
                self.reg.adc(d);
            }

            // SBC A,D
            0x9a => {
                let d = self.reg.d;
                self.reg.sbc(d);
            }

            // XOR D
            0xaa => {
                let d = self.reg.d;
                self.reg.xor(d);
            }

            // CP D
            0xba => {
                let d = self.reg.d;
                self.reg.cp(d);
            }

            // JP Z,a16
            0xca => {
                if self.reg.f.contains(Flags::ZERO) {
                    let address = LittleEndian::read_u16(&instruction.operands);
                    self.reg.pc = address;
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // JP C,a16
            0xda => {
                if self.reg.f.contains(Flags::CARRY) {
                    let address = LittleEndian::read_u16(&instruction.operands);
                    self.reg.pc = address;
                    condition_taken = true;

                    // Internal delay
                    bus.tick(MCycles(1));
                }
            }

            // LD (a16),A
            0xea => {
                let address = LittleEndian::read_u16(&instruction.operands);
                bus.write_byte(address, self.reg.a);
            }

            // LD A,(a16)
            0xfa => {
                let address = LittleEndian::read_u16(&instruction.operands);
                self.reg.a = bus.read_byte(address);
            }

            // DEC BC
            0x0b => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                self.reg.bc_mut().sub_assign(1);
            }

            // DEC DE
            0x1b => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                self.reg.de_mut().sub_assign(1);
            }

            // DEC HL
            0x2b => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                self.reg.hl_mut().sub_assign(1);
            }

            // DEC SP
            0x3b => {
                // Internal delay (not observable)
                bus.tick(MCycles(1));

                self.reg.sp = self.reg.sp.wrapping_sub(1);
            }

            // LD C,E
            0x4b => self.reg.c = self.reg.e,

            // LD E,E
            #[allow(clippy::self_assignment)]
            0x5b => self.reg.e = self.reg.e,

            // LD L,E
            0x6b => self.reg.l = self.reg.e,

            // LD A,E
            0x7b => self.reg.a = self.reg.e,

            // ADC A,E
            0x8b => {
                let e = self.reg.e;
                self.reg.adc(e);
            }

            // SBC A,E
            0x9b => {
                let e = self.reg.e;
                self.reg.sbc(e);
            }

            // XOR E
            0xab => {
                let e = self.reg.e;
                self.reg.xor(e);
            }

            // CP E
            0xbb => {
                let e = self.reg.e;
                self.reg.cp(e);
            }

            // PREFIX CB
            0xcb => {
                self.execute_prefix(instruction.operands[0], bus);
            }

            // UNUSED
            // 0xdb

            // UNUSED
            // 0xeb

            // EI
            0xFB => bus.interrupts.enabled = true,

            // INC C
            0x0c => arithmetic::inc(&mut self.reg.c, &mut self.reg.f),

            // INC E
            0x1c => arithmetic::inc(&mut self.reg.e, &mut self.reg.f),

            // INC L
            0x2c => arithmetic::inc(&mut self.reg.l, &mut self.reg.f),

            // INC A
            0x3c => arithmetic::inc(&mut self.reg.a, &mut self.reg.f),

            // LD C,H
            0x4c => self.reg.c = self.reg.h,

            // LD E,H
            0x5c => self.reg.e = self.reg.h,

            // LD L,H
            0x6c => self.reg.l = self.reg.h,

            // LD A,H
            0x7c => self.reg.a = self.reg.h,

            // ADC A,H
            0x8c => {
                let h = self.reg.h;
                self.reg.adc(h);
            }

            // SBC A,H
            0x9c => {
                let h = self.reg.h;
                self.reg.sbc(h);
            }

            // XOR H
            0xac => {
                let h = self.reg.h;
                self.reg.xor(h);
            }

            // CP H
            0xbc => {
                let h = self.reg.h;
                self.reg.cp(h);
            }

            // CALL Z,a16
            0xcc => {
                if self.reg.f.contains(Flags::ZERO) {
                    let address = LittleEndian::read_u16(&instruction.operands);

                    // Internal delay
                    bus.tick(MCycles(1));

                    self.call(address, bus);
                    condition_taken = true;
                }
            }

            // CALL C,a16
            0xdc => {
                if self.reg.f.contains(Flags::CARRY) {
                    let address = LittleEndian::read_u16(&instruction.operands);

                    // Internal delay
                    bus.tick(MCycles(1));

                    self.call(address, bus);
                    condition_taken = true;
                }
            }

            // UNUSED
            // 0xec

            // UNUSED
            // 0xfc

            // DEC C
            0x0d => arithmetic::dec(&mut self.reg.c, &mut self.reg.f),

            // DEC E
            0x1d => arithmetic::dec(&mut self.reg.e, &mut self.reg.f),

            // DEC L
            0x2d => arithmetic::dec(&mut self.reg.l, &mut self.reg.f),

            // DEC A
            0x3d => arithmetic::dec(&mut self.reg.a, &mut self.reg.f),

            // LD C,L
            0x4d => self.reg.c = self.reg.l,

            // LD E,L
            0x5d => self.reg.e = self.reg.l,

            // LD L,L
            #[allow(clippy::self_assignment)]
            0x6d => self.reg.l = self.reg.l,

            // LD A,L
            0x7d => self.reg.a = self.reg.l,

            // ADC A,L
            0x8d => {
                let l = self.reg.l;
                self.reg.adc(l);
            }

            // SBC A,L
            0x9d => {
                let l = self.reg.l;
                self.reg.sbc(l);
            }

            // XOR L
            0xad => {
                let l = self.reg.l;
                self.reg.xor(l);
            }

            // CP L
            0xbd => {
                let l = self.reg.l;
                self.reg.cp(l);
            }

            // CALL a16
            0xcd => {
                let address = LittleEndian::read_u16(&instruction.operands);

                // Internal delay
                bus.tick(MCycles(1));

                self.call(address, bus);
            }

            // UNUSED
            // 0xdd

            // UNUSED
            // 0xed

            // UNUSED
            // 0xfd

            // LD C,d8
            0x0e => self.reg.c = instruction.operands[0],

            // LD E,d8
            0x1e => self.reg.e = instruction.operands[0],

            // LD L,d8
            0x2e => self.reg.l = instruction.operands[0],

            // LD A,d8
            0x3e => self.reg.a = instruction.operands[0],

            // LD C,(HL)
            0x4e => self.reg.c = bus.read_byte(self.reg.hl()),

            // LD E,(HL)
            0x5e => self.reg.e = bus.read_byte(self.reg.hl()),

            // LD L,(HL)
            0x6e => self.reg.l = bus.read_byte(self.reg.hl()),

            // LD A,(HL)
            0x7e => self.reg.a = bus.read_byte(self.reg.hl()),

            // ADC A,(HL)
            0x8e => {
                let byte = bus.read_byte(self.reg.hl());
                self.reg.adc(byte);
            }

            // SBC A,(HL)
            0x9e => {
                let byte = bus.read_byte(self.reg.hl());
                self.reg.sbc(byte);
            }

            // XOR (HL)
            0xae => {
                let byte = bus.read_byte(self.reg.hl());
                self.reg.xor(byte);
            }

            // CP (HL)
            0xbe => {
                let byte = bus.read_byte(self.reg.hl());
                self.reg.cp(byte);
            }

            // ADC A,d8
            0xce => self.reg.adc(instruction.operands[0]),

            // SBC A,d8
            0xde => self.reg.sbc(instruction.operands[0]),

            // XOR d8
            0xee => self.reg.xor(instruction.operands[0]),

            // CP d8
            0xfe => self.reg.cp(instruction.operands[0]),

            // RRCA
            0x0f => self.reg.rrca(),

            // RRA
            0x1f => self.reg.rra(),

            // CPL
            0x2f => self.reg.cpl(),

            // CCF
            0x3f => self.reg.ccf(),

            // LD C,A
            0x4f => self.reg.c = self.reg.a,

            // LD E,A
            0x5f => self.reg.e = self.reg.a,

            // LD L,A
            0x6f => self.reg.l = self.reg.a,

            // LD A,A
            #[allow(clippy::self_assignment)]
            0x7f => self.reg.a = self.reg.a,

            // ADC A,A
            0x8f => {
                let a = self.reg.a;
                self.reg.adc(a);
            }

            // SBC A,A
            0x9f => {
                let a = self.reg.a;
                self.reg.sbc(a);
            }

            // XOR A
            0xaf => {
                // Effectively sets A to 0 and unconditionally sets the Zero flag.
                let a = self.reg.a;
                self.reg.xor(a);
            }

            // CP A
            0xbf => {
                let a = self.reg.a;
                self.reg.cp(a);
            }

            // RST 08H
            0xcf => {
                // Internal delay
                bus.tick(MCycles(1));

                self.rst(0x0008, bus);
            }

            // RST 18H
            0xdf => {
                // Internal delay
                bus.tick(MCycles(1));

                self.rst(0x0018, bus);
            }

            // RST 28H
            0xef => {
                // Internal delay
                bus.tick(MCycles(1));

                self.rst(0x0028, bus);
            }

            // RST 38H
            0xff => {
                // Internal delay
                bus.tick(MCycles(1));

                self.rst(0x0038, bus);
            }

            // Unused instructions
            0xe3 | 0xd3 | 0xf4 | 0xe4 | 0xeb | 0xdb | 0xfc | 0xec | 0xdd | 0xed | 0xfd => {
                self.state = State::Locked;
            }
        }

        if cfg!(debug_assertions) {
            let cycles = match (condition_taken, instruction.def.condition_cycles) {
                (true, Some(cycles)) => cycles,
                _ => instruction.cycles(),
            };

            debug_assert_eq!(
                bus.timer.diff(),
                MCycles::from(cycles),
                "incorrect timing for instruction {:#04x} ({})",
                instruction.def.byte,
                instruction.def.description
            );
        }
    }

    /// Pushes the current value of the program counter onto the stack, then jumps to a specific
    /// address.
    ///
    /// The current value of the program counter is assumed to be the address of the next
    /// instruction.
    pub fn rst(&mut self, addr: u16, bus: &mut Bus) {
        let pc = self.reg.pc;
        self.push(pc, bus);
        self.reg.pc = addr;
    }

    /// Performs a CALL operation. Does not modify any flags.
    fn call(&mut self, address: u16, bus: &mut Bus) {
        let pc = self.reg.pc;
        self.push(pc, bus);
        self.reg.pc = address;
    }

    /// Performs a RET operation. Does not modify any flags.
    fn ret(&mut self, bus: &mut Bus) {
        self.reg.pc = self.pop(bus);
    }

    /// Performs JR (relative jump) operation. Does not modify any flags.
    fn jr(&mut self, jump: i8) {
        let pc = self.reg.pc as i16;
        self.reg.pc = (pc + i16::from(jump)) as u16;
    }
}

#[cfg(test)]
mod tests {
    use smallvec::SmallVec;

    use crate::bus::Bus;
    use crate::cpu::{Cpu, Flags, MCycles, TCycles};

    use super::{Instruction, InstructionDef, INSTRUCTIONS};

    #[test]
    fn timings() {
        // These timings taken from blargg's instruction timing test ROM.
        #[rustfmt::skip]
        let timings = vec![
            1,3,2,2,1,1,2,1,5,2,2,2,1,1,2,1,
            0,3,2,2,1,1,2,1,3,2,2,2,1,1,2,1,
            2,3,2,2,1,1,2,1,2,2,2,2,1,1,2,1,
            2,3,2,2,3,3,3,1,2,2,2,2,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            2,2,2,2,2,2,0,2,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            2,3,3,4,3,4,2,4,2,4,3,0,3,6,2,4,
            2,3,3,0,3,4,2,4,2,4,3,0,3,0,2,4,
            3,3,2,0,0,4,2,4,4,1,4,0,0,0,2,4,
            3,3,2,1,0,4,2,4,3,2,4,1,0,0,2,4,
        ];

        #[rustfmt::skip]
        let condition_timings = vec![
            1,3,2,2,1,1,2,1,5,2,2,2,1,1,2,1,
            0,3,2,2,1,1,2,1,3,2,2,2,1,1,2,1,
            3,3,2,2,1,1,2,1,3,2,2,2,1,1,2,1,
            3,3,2,2,3,3,3,1,3,2,2,2,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            2,2,2,2,2,2,0,2,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
            5,3,4,4,6,4,2,4,5,4,4,0,6,6,2,4,
            5,3,4,0,6,4,2,4,5,4,4,0,6,0,2,4,
            3,3,2,0,0,4,2,4,4,1,4,0,0,0,2,4,
            3,3,2,1,0,4,2,4,3,2,4,1,0,0,2,4,
        ];

        for (byte, instruction) in INSTRUCTIONS.iter().enumerate() {
            let timing = MCycles(timings[byte as usize]);

            // Skip the assertion if the timing isn't tested.
            if timing.0 == 0 {
                continue;
            }

            if timing != MCycles::from(instruction.cycles) {
                panic!(
                    "wrong timing for {}: has {}, expected {}",
                    instruction.description, instruction.cycles, timing
                );
            }

            if let Some(condition_cycles) = instruction.condition_cycles {
                let clock_cycles = MCycles(condition_timings[byte as usize]);

                if clock_cycles != MCycles::from(condition_cycles) {
                    panic!(
                        "wrong condition timing for {}: has {}, expected {}",
                        instruction.description, condition_cycles, clock_cycles
                    );
                }
            }
        }
    }

    #[test]
    fn instructions() {
        let nop = &INSTRUCTIONS[0x00];
        assert_eq!(
            nop,
            &InstructionDef {
                byte: 0x00,
                description: "NOP",
                num_operands: 0,
                cycles: TCycles(4),
                condition_cycles: None,
            }
        );

        let ld = &INSTRUCTIONS[0x3e];
        assert_eq!(
            ld,
            &InstructionDef {
                byte: 0x3e,
                description: "LD A,d8",
                num_operands: 1,
                cycles: TCycles(8),
                condition_cycles: None,
            }
        );

        let jp = &INSTRUCTIONS[0xc3];
        assert_eq!(
            jp,
            &InstructionDef {
                byte: 0xc3,
                description: "JP a16",
                num_operands: 2,
                cycles: TCycles(16),
                condition_cycles: None,
            }
        );

        let conditional_jp = &INSTRUCTIONS[0xc8];
        assert_eq!(
            conditional_jp,
            &InstructionDef {
                byte: 0xc8,
                description: "RET Z",
                num_operands: 0,
                cycles: TCycles(8),
                condition_cycles: Some(TCycles(20)),
            }
        );

        let prefix_instruction = &INSTRUCTIONS[0xcb];
        assert_eq!(
            prefix_instruction,
            &InstructionDef {
                byte: 0xcb,
                description: "PREFIX CB",
                num_operands: 1,
                cycles: TCycles(0),
                condition_cycles: None,
            }
        );
    }

    #[test]
    fn cycles() {
        let rl_c = Instruction {
            def: &INSTRUCTIONS[0xCB],
            operands: SmallVec::from_slice(&[0x11]),
        };

        assert_eq!(rl_c.cycles(), TCycles(8));

        let res_0_hl = Instruction {
            def: &INSTRUCTIONS[0xCB],
            operands: SmallVec::from_slice(&[0x86]),
        };

        assert_eq!(res_0_hl.cycles(), TCycles(16));
    }

    #[test]
    fn instruction_display() {
        let nop = Instruction {
            def: &INSTRUCTIONS[0x00],
            operands: Default::default(),
        };

        assert_eq!(&nop.to_string(), "NOP");

        let jr_nz_r8 = Instruction {
            def: &INSTRUCTIONS[0x20],
            operands: SmallVec::from_slice(&[0xfe]),
        };

        assert_eq!(&jr_nz_r8.to_string(), "JR NZ,$0xfe");

        let ld_hl_d16 = Instruction {
            def: &INSTRUCTIONS[0x21],
            operands: SmallVec::from_slice(&[0xef, 0xbe]),
        };

        assert_eq!(&ld_hl_d16.to_string(), "LD HL,$0xbeef");

        let rl_c = Instruction {
            def: &INSTRUCTIONS[0xCB],
            operands: SmallVec::from_slice(&[0x11]),
        };

        assert_eq!(&rl_c.to_string(), "RL C");
    }

    #[test]
    fn fetch() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();
        bus.write_byte_no_tick(0xC000, 0x00);
        cpu.reg.pc = 0xC000;
        let nop = cpu.fetch(&mut bus);
        assert_eq!(nop.def.byte, 0x00);
        assert_eq!(nop.def.num_operands, 0);
        assert_eq!(nop.operands.len(), 0);

        let mut bus = Bus::default();
        let mut cpu = Cpu::new();
        bus.write_byte_no_tick(0xC000, 0xcb);
        bus.write_byte_no_tick(0xC001, 0x7c);
        cpu.reg.pc = 0xC000;
        let prefix_instruction = cpu.fetch(&mut bus);
        assert_eq!(prefix_instruction.def.byte, 0xcb);
        assert_eq!(prefix_instruction.def.num_operands, 1);
        assert_eq!(prefix_instruction.operands.into_vec().as_slice(), &[0x7c]);
    }

    #[test]
    fn execute() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        let nop = Instruction {
            def: &INSTRUCTIONS[0x00],
            ..Default::default()
        };
        bus.tick(MCycles(1));

        cpu.execute(&nop, &mut bus);
        assert_eq!(bus.timer.diff(), MCycles(1));
    }

    #[test]
    fn rst() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.sp = 0xFFF0;
        cpu.reg.pc = 0xAB;

        let instruction = Instruction {
            def: &INSTRUCTIONS[0xff],
            operands: Default::default(),
        };
        bus.tick(MCycles(1));
        cpu.execute(&instruction, &mut bus);

        assert_eq!(cpu.reg.pc, 0x38);
        assert_eq!(cpu.pop(&mut bus), 0xAB + 1);
    }

    #[test]
    fn jr_nz() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.pc = 0;

        // Move forward 10
        let instruction = Instruction {
            def: &INSTRUCTIONS[0x20],
            operands: SmallVec::from_slice(&[0x0a]),
        };
        bus.tick(MCycles(2));
        cpu.execute(&instruction, &mut bus);
        assert_eq!(cpu.reg.pc, 12);
        assert_eq!(bus.timer.diff(), MCycles(3));

        // Move backward 10
        bus.timer.reset_diff();
        let instruction = Instruction {
            def: &INSTRUCTIONS[0x20],
            operands: SmallVec::from_slice(&[!0x0a + 1]),
        };
        bus.tick(MCycles(2));
        cpu.execute(&instruction, &mut bus);
        assert_eq!(cpu.reg.pc, 4);
        assert_eq!(bus.timer.diff(), MCycles(3));

        // Do not jump
        bus.timer.reset_diff();
        cpu.reg.f.insert(Flags::ZERO);
        let instruction = Instruction {
            def: &INSTRUCTIONS[0x20],
            operands: SmallVec::from_slice(&[0x0a]),
        };
        bus.tick(MCycles(2));
        cpu.execute(&instruction, &mut bus);
        assert_eq!(cpu.reg.pc, 6);
        assert_eq!(bus.timer.diff(), MCycles(2));
    }

    #[test]
    fn jr() {
        let mut cpu = Cpu::new();

        cpu.reg.pc = 0x01;

        // Move forward 10
        cpu.jr(0x0a);
        assert_eq!(cpu.reg.pc, 0x0b);

        // Move backward 10
        cpu.jr(!0x0a + 1);
        assert_eq!(cpu.reg.pc, 0x01);
    }

    #[test]
    fn ld_addr_c_a() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        // Write to the SCY (Scroll Y) register.
        cpu.reg.c = 0x42;
        cpu.reg.a = 0xab;
        let instruction = Instruction {
            def: &INSTRUCTIONS[0xe2],
            operands: Default::default(),
        };
        bus.tick(MCycles(1));
        cpu.execute(&instruction, &mut bus);

        assert_eq!(bus.read_byte(0xFF42), 0xab);
    }

    #[test]
    fn ldh_c() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        // Write to SCX (Scroll X) register.
        bus.write_byte(0xFF42, 0xBE);
        cpu.reg.c = 0x42;
        let instruction = Instruction {
            def: &INSTRUCTIONS[0xf2],
            ..Default::default()
        };
        cpu.execute(&instruction, &mut bus);

        assert_eq!(cpu.reg.a, 0xBE);
    }

    #[test]
    fn ld_addr_a16_a() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.a = 0x11;

        let instruction = Instruction {
            def: &INSTRUCTIONS[0xea],
            operands: SmallVec::from_slice(&[0x00, 0xc0]),
        };
        bus.tick(MCycles(3));
        cpu.execute(&instruction, &mut bus);

        assert_eq!(bus.read_byte(0xc000), 0x11);
    }

    #[test]
    fn ld_a_addr_a16() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        bus.write_byte_no_tick(0xc000, 0xaa);

        let instruction = Instruction {
            def: &INSTRUCTIONS[0xfa],
            operands: SmallVec::from_slice(&[0x00, 0xc0]),
        };
        bus.tick(MCycles(3));
        cpu.execute(&instruction, &mut bus);

        assert_eq!(cpu.reg.a, 0xaa);
    }

    #[test]
    fn load_16() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.sp = 0xFFF8;

        let instruction = Instruction {
            def: &INSTRUCTIONS[0x08],
            operands: SmallVec::from_slice(&[0x00, 0xC1]),
        };
        bus.tick(MCycles(3));
        cpu.execute(&instruction, &mut bus);

        assert_eq!(bus.read_byte(0xC100), 0xF8);
        assert_eq!(bus.read_byte(0xC101), 0xFF);
    }

    #[test]
    fn call() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.sp = 0xffff;
        cpu.reg.pc = 1;
        cpu.call(4, &mut bus);

        assert_eq!(cpu.reg.pc, 4);
        assert_eq!(cpu.pop(&mut bus), 1);
    }

    #[test]
    fn ret() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.sp = 0xffff;
        cpu.push(5, &mut bus);
        cpu.ret(&mut bus);

        assert_eq!(cpu.reg.sp, 0xffff);
        assert_eq!(cpu.reg.pc, 5);
    }

    #[test]
    fn scf() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.f = Flags::empty();

        let instruction_1 = Instruction {
            def: &INSTRUCTIONS[0x37],
            operands: SmallVec::new(),
        };
        bus.tick(MCycles(1));
        cpu.execute(&instruction_1, &mut bus);

        assert_eq!(cpu.reg.f, Flags::CARRY);

        cpu.reg
            .f
            .insert(Flags::ZERO | Flags::SUBTRACT | Flags::HALF_CARRY | Flags::CARRY);

        bus.timer.reset_diff();
        let instruction_2 = Instruction {
            def: &INSTRUCTIONS[0x37],
            operands: SmallVec::new(),
        };
        bus.tick(MCycles(1));
        cpu.execute(&instruction_2, &mut bus);

        assert_eq!(cpu.reg.f, Flags::ZERO | Flags::CARRY);
    }

    #[test]
    fn jp_hl() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.pc = 0;
        cpu.reg.hl_mut().write(0xbeef);

        let instruction = Instruction {
            def: &INSTRUCTIONS[0xe9],
            operands: SmallVec::new(),
        };
        bus.tick(MCycles(1));
        cpu.execute(&instruction, &mut bus);

        assert_eq!(cpu.reg.pc, 0xbeef);
    }

    #[test]
    fn ld_sp_hl() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();

        cpu.reg.sp = 0;
        cpu.reg.hl_mut().write(0xbeef);

        let instruction = Instruction {
            def: &INSTRUCTIONS[0xf9],
            operands: SmallVec::new(),
        };
        bus.tick(MCycles(1));
        cpu.execute(&instruction, &mut bus);

        assert_eq!(cpu.reg.sp, 0xbeef);
    }

    #[test]
    fn pop_af() {
        let mut bus = Bus::default();
        let mut cpu = Cpu::new();
        cpu.reg.sp = 0xFFFD;
        bus.write_byte_no_tick(cpu.reg.sp, 0xFF);
        bus.write_byte_no_tick(cpu.reg.sp + 1, 0xFF);

        let instruction = Instruction {
            def: &INSTRUCTIONS[0xF1],
            ..Default::default()
        };
        bus.tick(MCycles(1));
        cpu.execute(&instruction, &mut bus);

        assert_eq!(cpu.reg.a, 0xFF);
        assert_eq!(cpu.reg.f.bits(), 0xF0);
        assert_eq!(cpu.reg.f, Flags::all());
    }
}
