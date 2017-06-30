//! CPU instruction definition.

use std::fmt::{self, Display};
use std::ops::{AddAssign, SubAssign};

use byteorder::{ByteOrder, LittleEndian};
use regex::{Regex, NoExpand};
use smallvec::SmallVec;

use cpu::{Flags, ZERO, SUBTRACT, HALF_CARRY, CARRY};
use memory::Addressable;

lazy_static! {
    /// Matches instruction descriptions that take operands.
    static ref DATA_RE: Regex = Regex::new("d8|d16|a8|a16|r8").unwrap();
}

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
        let instruction = if let Some(mat) = DATA_RE.find(self.def.description) {
            let replacement = match mat.as_str() {
                "d8" | "a8" | "r8" => format!("${:#04x}", &self.operands[0]),
                "d16" | "a16" => format!("${:#06x}", LittleEndian::read_u16(&self.operands)),
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

/// Macro to quickly define all CPU instructions for the Game Boy Z80 processor.
macro_rules! instructions {
    ( $( $byte:expr, $description:expr, $cycles:expr ; )* ) => {
        {
            let mut instructions = vec![None; 0x100];

            $(
                let num_operands = DATA_RE.find($description).map(|mat| {
                    match mat.as_str() {
                        "d8" | "a8" | "r8" => 1,
                        "d16" | "a16" => 2,
                        ty => unreachable!("unhandled data type: {}", ty),
                    }
                }).unwrap_or_default();

                instructions[$byte] = Some(InstructionDef {
                    byte: $byte,
                    description: $description,
                    cycles: $cycles,
                    num_operands: num_operands,
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

        let def = INSTRUCTIONS[byte as usize].as_ref().expect(&format!(
            "could not find data for instruction {:#04x}",
            byte
        ));

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
    /// All necessary side effects are performed, including updating the program counter, flag
    /// registers, and CPU clock.
    ///
    /// Returns the number of clock cycles the instruction takes.
    pub fn execute(&mut self, instruction: Instruction) -> u32 {
        debug!("executing {:#06x} {}", self.reg.pc, instruction.to_string());
        trace!("{:?}", instruction);

        // Check that we have exactly as many operands as the instruction requires.
        debug_assert_eq!(
            instruction.def.num_operands as usize,
            instruction.operands.len()
        );

        let mut cycles = instruction.def.cycles as u32;

        // Increment the program counter (PC) *before* executing the instruction.
        //
        // This how the actual hardware handles the PC, as relative jumps and other PC-related
        // instructions assume that PC is pointing at the *next* instruction.
        self.reg.pc += 1 + instruction.operands.len() as u16;

        // Execute the instruction.
        match instruction.def.byte {
            // NOP
            0x00 => (),

            // STOP
            0x10 => self.halted = true,

            // JR NZ,r8
            0x20 => {
                if !self.reg.f.contains(ZERO) {
                    self.jr(instruction.operands[0] as i8);
                    cycles += 4;
                }
            }

            // JR NC,r8
            0x30 => {
                if !self.reg.f.contains(CARRY) {
                    self.jr(instruction.operands[0] as i8);
                    cycles += 4;
                }
            }

            // LD B,B
            0x40 => (),

            // LD D,B
            0x50 => self.reg.d = self.reg.b,

            // LD H,B
            0x60 => self.reg.h = self.reg.b,

            // LD (HL),B
            0x70 => self.mmu.borrow_mut().write_byte(self.reg.hl(), self.reg.b),

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

            // RET NZ
            0xc0 => {
                if !self.reg.f.contains(ZERO) {
                    self.ret();

                    cycles += 12;
                }
            }

            // RET NC
            0xd0 => {
                if !self.reg.f.contains(CARRY) {
                    self.ret();

                    cycles += 12;
                }
            }

            // LDH (a8),A
            0xe0 => {
                let address = 0xff00u16 + &instruction.operands[0].into();
                self.mmu.borrow_mut().write_byte(address, self.reg.a)
            }

            // LDH A,(a8)
            0xf0 => {
                let address = 0xff00u16 + &instruction.operands[0].into();
                self.reg.a = self.mmu.borrow().read_byte(address);
            }

            // LD BC,d16
            0x01 => {
                self.reg.bc_mut().write(LittleEndian::read_u16(
                    &instruction.operands,
                ))
            }

            // LD DE,d16
            0x11 => {
                self.reg.de_mut().write(LittleEndian::read_u16(
                    &instruction.operands,
                ))
            }

            // LD HL,d16
            0x21 => {
                self.reg.hl_mut().write(LittleEndian::read_u16(
                    &instruction.operands,
                ))
            }

            // LD SP,d16
            0x31 => self.reg.sp = LittleEndian::read_u16(&instruction.operands),

            // LD B,C
            0x41 => self.reg.b = self.reg.c,

            // LD D,C
            0x51 => self.reg.d = self.reg.c,

            // LD H,C
            0x61 => self.reg.h = self.reg.c,

            // LD (HL),C
            0x71 => self.mmu.borrow_mut().write_byte(self.reg.hl(), self.reg.c),

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

            // LD (BC),A
            0x02 => self.mmu.borrow_mut().write_byte(self.reg.bc(), self.reg.a),

            // LD (DE),A
            0x12 => self.mmu.borrow_mut().write_byte(self.reg.de(), self.reg.a),

            // LD (HL+),A
            0x22 => {
                self.mmu.borrow_mut().write_byte(self.reg.hl(), self.reg.a);
                self.reg.hl_mut().add_assign(1);
            }

            // LD (HL-),A
            0x32 => {
                self.mmu.borrow_mut().write_byte(self.reg.hl(), self.reg.a);
                self.reg.hl_mut().sub_assign(1);
            }

            // LD B,D
            0x42 => self.reg.b = self.reg.d,

            // LD D,D
            0x52 => (),

            // LD H,D
            0x62 => self.reg.h = self.reg.d,

            // LD (HL),D
            0x72 => self.mmu.borrow_mut().write_byte(self.reg.hl(), self.reg.d),

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

            // JP NZ,a16
            0xc2 => {
                if !self.reg.f.contains(ZERO) {
                    self.reg.pc = LittleEndian::read_u16(&instruction.operands);
                    cycles += 4;
                }
            }

            // JP NC,a16
            0xd2 => {
                if !self.reg.f.contains(CARRY) {
                    self.reg.pc = LittleEndian::read_u16(&instruction.operands);
                    cycles += 4;
                }
            }

            // LD (C),A
            // LD ($FF00+C),A
            0xe2 => {
                let address = 0xff00u16 + &self.reg.c.into();
                self.mmu.borrow_mut().write_byte(address, self.reg.a);
            }

            // LD A,(C)
            // LD A,($FF00+C)
            0xf2 => {
                let address = 0xff00u16 + &self.reg.c.into();
                self.reg.a = self.mmu.borrow().read_byte(address);
            }

            // INC BC
            0x03 => self.reg.bc_mut().add_assign(1),

            // INC DE
            0x13 => self.reg.de_mut().add_assign(1),

            // INC HL
            0x23 => self.reg.hl_mut().add_assign(1),

            // INC SP
            0x33 => self.reg.sp.add_assign(1),

            // LD B,E
            0x43 => self.reg.b = self.reg.e,

            // LD D,E
            0x53 => self.reg.d = self.reg.e,

            // LD H,E
            0x63 => self.reg.h = self.reg.e,

            // LD (HL),E
            0x73 => self.mmu.borrow_mut().write_byte(self.reg.hl(), self.reg.e),

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

            // JP a16
            0xc3 => self.reg.pc = LittleEndian::read_u16(&instruction.operands),

            // DI
            0xf3 => self.interrupts = false,

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

            // LD B,H
            0x44 => self.reg.b = self.reg.h,

            // LD D,H
            0x54 => self.reg.d = self.reg.h,

            // LD H,H
            0x64 => (),

            // LD (HL),H
            0x74 => self.mmu.borrow_mut().write_byte(self.reg.hl(), self.reg.h),

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

            // CALL NZ,a16
            0xc4 => {
                if !self.reg.f.contains(ZERO) {
                    self.call(LittleEndian::read_u16(&instruction.operands));

                    cycles += 12;
                }
            }

            // CALL NC,a16
            0xd4 => {
                if !self.reg.f.contains(CARRY) {
                    self.call(LittleEndian::read_u16(&instruction.operands));

                    cycles += 12;
                }
            }

            // DEC B
            0x05 => Self::dec(&mut self.reg.b, &mut self.reg.f),

            // DEC D
            0x15 => Self::dec(&mut self.reg.d, &mut self.reg.f),

            // DEC H
            0x25 => Self::dec(&mut self.reg.h, &mut self.reg.f),

            // DEC (HL)
            0x35 => {
                let mut byte = self.mmu.borrow().read_byte(self.reg.hl());
                Self::dec(&mut byte, &mut self.reg.f);
                self.mmu.borrow_mut().write_byte(self.reg.hl(), byte);
            }

            // LD B,L
            0x45 => self.reg.b = self.reg.l,

            // LD D,L
            0x55 => self.reg.d = self.reg.l,

            // LD H,L
            0x65 => self.reg.h = self.reg.l,

            // LD (HL),L
            0x75 => self.mmu.borrow_mut().write_byte(self.reg.hl(), self.reg.l),

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

            // LD (HL),d8
            0x36 => {
                self.mmu.borrow_mut().write_byte(
                    self.reg.hl(),
                    instruction.operands[0],
                )
            }

            // LD B,(HL)
            0x46 => self.reg.b = self.mmu.borrow().read_byte(self.reg.hl()),

            // LD D,(HL)
            0x56 => self.reg.d = self.mmu.borrow().read_byte(self.reg.hl()),

            // LD H,(HL)
            0x66 => self.reg.h = self.mmu.borrow().read_byte(self.reg.hl()),

            // ADD A,(HL)
            0x86 => {
                let byte = self.mmu.borrow().read_byte(self.reg.hl());
                self.reg.add(byte);
            }

            // SUB (HL)
            0x96 => {
                let byte = self.mmu.borrow().read_byte(self.reg.hl());
                self.reg.sub(byte);
            }

            // AND (HL)
            0xa6 => {
                let byte = self.mmu.borrow().read_byte(self.reg.hl());
                self.reg.and(byte);
            }

            // ADD A,d8
            0xc6 => self.reg.add(instruction.operands[0]),

            // SUB d8
            0xd6 => self.reg.sub(instruction.operands[0]),

            // AND d8
            0xe6 => {
                let byte = instruction.operands[0];
                self.reg.and(byte);
            }

            // LD B,A
            0x47 => self.reg.b = self.reg.a,

            // LD D,A
            0x57 => self.reg.d = self.reg.a,

            // LD H,A
            0x67 => self.reg.h = self.reg.a,

            // LD (HL),A
            0x77 => self.mmu.borrow_mut().write_byte(self.reg.hl(), self.reg.a),

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

            // RST 00H
            0xc7 => self.rst(0x0000),

            // RST 10H
            0xd7 => self.rst(0x0010),

            // RST 20H
            0xe7 => self.rst(0x0020),

            // RST 30H
            0xf7 => self.rst(0x0030),

            // JR r8
            0x18 => self.jr(instruction.operands[0] as i8),

            // JR Z,r8
            0x28 => {
                if self.reg.f.contains(ZERO) {
                    self.jr(instruction.operands[0] as i8);
                    cycles += 4;
                }
            }

            // JR C,r8
            0x38 => {
                if self.reg.f.contains(CARRY) {
                    self.jr(instruction.operands[0] as i8);
                    cycles += 4;
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

            // XOR B
            0xa8 => {
                let b = self.reg.b;
                self.reg.xor(b)
            }

            // RET Z
            0xc8 => {
                if self.reg.f.contains(ZERO) {
                    self.ret();

                    cycles += 12;
                }
            }

            // RET C
            0xd8 => {
                if self.reg.f.contains(CARRY) {
                    self.ret();

                    cycles += 12;
                }
            }

            // LD C,C
            0x49 => (),

            // LD E,C
            0x59 => self.reg.e = self.reg.c,

            // LD L,C
            0x69 => self.reg.l = self.reg.c,

            // LD A,C
            0x79 => self.reg.a = self.reg.c,

            // XOR C
            0xa9 => {
                let c = self.reg.c;
                self.reg.xor(c);
            }

            // RET
            0xc9 => {
                self.ret();
            }

            // RETI
            0xd9 => {
                self.ret();
                self.interrupts = true;
            }

            // LD A,(BC)
            0x0a => {
                let bc = self.reg.bc();
                self.reg.a = self.mmu.borrow().read_byte(bc);
            }

            // LD A,(DE)
            0x1a => {
                let de = self.reg.de();
                self.reg.a = self.mmu.borrow().read_byte(de);
            }

            // LD A,(HL+)
            0x2a => {
                self.reg.a = self.mmu.borrow().read_byte(self.reg.hl());
                self.reg.hl_mut().add_assign(1);
            }

            // LD A,(HL-)
            0x3a => {
                self.reg.a = self.mmu.borrow().read_byte(self.reg.hl());
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

            // XOR D
            0xaa => {
                let d = self.reg.d;
                self.reg.xor(d);
            }

            // LD (a16),A
            0xea => {
                let address = LittleEndian::read_u16(&instruction.operands);
                self.mmu.borrow_mut().write_byte(address, self.reg.a);
            }

            // LD A,(a16)
            0xfa => {
                let address = LittleEndian::read_u16(&instruction.operands);
                self.reg.a = self.mmu.borrow().read_byte(address);
            }

            // DEC BC
            0x0b => self.reg.bc_mut().sub_assign(1),

            // DEC DE
            0x1b => self.reg.de_mut().sub_assign(1),

            // DEC HL
            0x2b => self.reg.hl_mut().sub_assign(1),

            // DEC SP
            0x3b => self.reg.sp -= 1,

            // LD C,E
            0x4b => self.reg.c = self.reg.e,

            // LD E,E
            0x5b => (),

            // LD L,E
            0x6b => self.reg.l = self.reg.e,

            // LD A,E
            0x7b => self.reg.a = self.reg.e,

            // XOR E
            0xab => {
                let e = self.reg.e;
                self.reg.xor(e);
            }

            0xcb => {
                error!("unimplemented prefix instruction");
                self.reg.pc += 1;
            }

            // EI
            0xfb => self.interrupts = true,

            // INC C
            0x0c => Self::inc(&mut self.reg.c, &mut self.reg.f),

            // INC E
            0x1c => Self::inc(&mut self.reg.e, &mut self.reg.f),

            // INC L
            0x2c => Self::inc(&mut self.reg.l, &mut self.reg.f),

            // INC A
            0x3c => Self::inc(&mut self.reg.a, &mut self.reg.f),

            // LD C,H
            0x4c => self.reg.c = self.reg.h,

            // LD E,H
            0x5c => self.reg.e = self.reg.h,

            // LD L,H
            0x6c => self.reg.l = self.reg.h,

            // LD A,H
            0x7c => self.reg.a = self.reg.h,

            // XOR H
            0xac => {
                let h = self.reg.h;
                self.reg.xor(h);
            }

            // CALL Z,a16
            0xcc => {
                if self.reg.f.contains(ZERO) {
                    self.call(LittleEndian::read_u16(&instruction.operands));

                    cycles += 12;
                }
            }

            // CALL C,a16
            0xdc => {
                if self.reg.f.contains(CARRY) {
                    self.call(LittleEndian::read_u16(&instruction.operands));

                    cycles += 12;
                }
            }

            // DEC C
            0x0d => Self::dec(&mut self.reg.c, &mut self.reg.f),

            // DEC E
            0x1d => Self::dec(&mut self.reg.e, &mut self.reg.f),

            // DEC L
            0x2d => Self::dec(&mut self.reg.l, &mut self.reg.f),

            // DEC A
            0x3d => Self::dec(&mut self.reg.a, &mut self.reg.f),

            // LD C,L
            0x4d => self.reg.c = self.reg.l,

            // LD E,L
            0x5d => self.reg.e = self.reg.l,

            // LD L,L
            0x6d => (),

            // LD A,L
            0x7d => self.reg.a = self.reg.l,

            // XOR L
            0xad => {
                let l = self.reg.l;
                self.reg.xor(l);
            }

            // CALL a16
            0xcd => {
                self.call(LittleEndian::read_u16(&instruction.operands));
            }

            // LD C,d8
            0x0e => self.reg.c = instruction.operands[0],

            // LD E,d8
            0x1e => self.reg.e = instruction.operands[0],

            // LD L,d8
            0x2e => self.reg.l = instruction.operands[0],

            // LD A,d8
            0x3e => self.reg.a = instruction.operands[0],

            // LD C,(HL)
            0x4e => self.reg.c = self.mmu.borrow().read_byte(self.reg.hl()),

            // LD E,(HL)
            0x5e => self.reg.e = self.mmu.borrow().read_byte(self.reg.hl()),

            // LD L,(HL)
            0x6e => self.reg.l = self.mmu.borrow().read_byte(self.reg.hl()),

            // LD A,(HL)
            0x7e => self.reg.a = self.mmu.borrow().read_byte(self.reg.hl()),

            // XOR (HL)
            0xae => {
                let byte = self.mmu.borrow().read_byte(self.reg.hl());
                self.reg.xor(byte);
            }

            // XOR d8
            0xee => self.reg.xor(instruction.operands[0]),

            // CP d8
            0xfe => self.reg.cp(instruction.operands[0]),

            // LD C,A
            0x4f => self.reg.c = self.reg.a,

            // LD E,A
            0x5f => self.reg.e = self.reg.a,

            // LD L,A
            0x6f => self.reg.l = self.reg.a,

            // LD A,A
            0x7f => (),

            // XOR A
            0xaf => {
                // Effectively sets A to 0 and unconditionally sets the Zero flag.
                let a = self.reg.a;
                self.reg.xor(a);
            }

            // RST 08H
            0xcf => self.rst(0x0008),

            // RST 18H
            0xdf => self.rst(0x0018),

            // RST 28H
            0xef => self.rst(0x0028),

            // RST 38H
            0xff => self.rst(0x0038),

            _ => panic!("unimplemented instruction: {:?}", instruction),
        }

        self.clock.t += cycles;
        self.clock.m += cycles / 4;

        cycles
    }

    /// Pushes the current value of the program counter onto the stack, then jumps to a specific
    /// address.
    ///
    /// The current value of the program counter is assumed to be the address of the next
    /// instruction.
    fn rst(&mut self, addr: u16) {
        let pc = self.reg.pc;
        self.push(pc);
        self.reg.pc = addr;
    }

    /// Increments a byte by 1 and sets the flags appropriately.
    fn inc(byte: &mut u8, flags: &mut Flags) {
        flags.set(HALF_CARRY, is_half_carry_add(*byte, 1));

        *byte = byte.wrapping_add(1);

        flags.set(ZERO, *byte == 0);
        flags.remove(SUBTRACT);
    }

    /// Decrements a byte by 1 and sets the flags appropriately.
    fn dec(byte: &mut u8, flags: &mut Flags) {
        flags.set(HALF_CARRY, is_half_carry_sub(*byte, 1));

        *byte = byte.wrapping_sub(1);

        flags.set(ZERO, *byte == 0);
        flags.insert(SUBTRACT);
    }

    /// Performs a CALL operation. Does not modify any flags.
    fn call(&mut self, address: u16) {
        let pc = self.reg.pc;
        self.push(pc);
        self.reg.pc = address;
    }

    /// Performs a RET operation. Does not modify any flags.
    fn ret(&mut self) {
        self.reg.pc = self.pop();
    }

    /// Performs JR (relative jump) operation. Does not modify any flags.
    fn jr(&mut self, jump: i8) {
        let pc = self.reg.pc as i16;
        self.reg.pc = (pc + jump as i16) as u16;
    }
}

impl super::Registers {
    /// Bitwise ANDs a byte with the accumulator and sets the flags appropriately.
    fn and(&mut self, rhs: u8) {
        self.a &= rhs;

        self.f.remove(SUBTRACT | CARRY);
        self.f.insert(HALF_CARRY);
        self.f.set(ZERO, self.a == 0);
    }

    /// Compares a byte with the accumulator.
    ///
    /// Performs a subtraction with the accumulator without actually setting the accumulator to the
    /// new value. Only the flags are set.
    fn cp(&mut self, rhs: u8) {
        self.f.set(ZERO, self.a == rhs);
        self.f.insert(SUBTRACT);
        self.f.set(HALF_CARRY, is_half_carry_sub(self.a, rhs));
        self.f.set(CARRY, is_carry_sub(self.a, rhs));
    }

    /// Adds a byte to the accumulator and sets the flags appropriately.
    fn add(&mut self, rhs: u8) {
        self.f.remove(SUBTRACT);
        self.f.set(HALF_CARRY, is_half_carry_add(self.a, rhs));
        self.f.set(CARRY, is_carry_add(self.a, rhs));

        self.a = self.a.wrapping_add(rhs);

        self.f.set(ZERO, self.a == 0);
    }

    /// Subtracts a byte from the accumulator and sets the flags appropriately.
    fn sub(&mut self, rhs: u8) {
        self.cp(rhs);
        self.a = self.a.wrapping_sub(rhs);
    }

    /// Performs an exclusive OR with the accumulator and sets the zero flag appropriately.
    fn xor(&mut self, rhs: u8) {
        self.a ^= rhs;
        self.f = Flags::empty();
        self.f.set(ZERO, self.a == 0);
    }
}

/// Returns `true` if the addition of two bytes requires a carry.
fn is_carry_add(a: u8, b: u8) -> bool {
    ((a as u16 + b as u16) & 0x0100) == 0x0100
}

/// Returns `true` if the addition of two bytes would require a half carry (a carry from the low
/// nibble to the high nibble).
fn is_half_carry_add(a: u8, b: u8) -> bool {
    (((a & 0xf).wrapping_add(b & 0xf)) & 0x10) == 0x10
}

/// Returns `true` if the subtraction of two bytes requires a carry from the most significant bit.
fn is_carry_sub(a: u8, b: u8) -> bool {
    b > a
}

/// Returns `true` if the subtraction of two bytes would require a half carry (a borrow from the
/// high nibble to the low nibble).
fn is_half_carry_sub(a: u8, b: u8) -> bool {
    (b & 0xf) > (a & 0xf)
}

lazy_static! {
    /// Game Boy instruction set.
    ///
    /// Timings and other information taken from [here].
    ///
    /// [here]: http://pastraiser.com/cpu/gameboy/gameboy_opcodes.html
    // FIXME: This should be `[Instruction; 0x100]` once all instructions are implemented.
    static ref INSTRUCTIONS: Vec<Option<InstructionDef>> = instructions! {
        // byte     description     cycles
        0x00,       "NOP",          4;
        0x10,       "STOP",         4;
        0x20,       "JR NZ,r8",     8;
        0x30,       "JR NC,r8",     8;
        0x40,       "LD B,B",       4;
        0x50,       "LD D,B",       4;
        0x60,       "LD H,B",       4;
        0x70,       "LD (HL),B",    8;
        0x80,       "ADD A,B",      4;
        0x90,       "SUB B",        4;
        0xa0,       "AND B",        4;
        0xc0,       "RET NZ",       8;
        0xd0,       "RET NC",       8;
        0xe0,       "LDH (a8),A",   12;     // AKA LD A,($FF00+a8)
        0xf0,       "LDH A,(a8)",   12;     // AKA LD ($FF00+a8),A
        0x01,       "LD BC,d16",    12;
        0x11,       "LD DE,d16",    12;
        0x21,       "LD HL,d16",    12;
        0x31,       "LD SP,d16",    12;
        0x41,       "LD B,C",       4;
        0x51,       "LD D,C",       4;
        0x61,       "LD H,C",       4;
        0x71,       "LD (HL),C",    8;
        0x81,       "ADD A,C",      4;
        0x91,       "SUB C",        4;
        0xa1,       "AND C",        4;
        0xc1,       "POP BC",       12;
        0xd1,       "POP DC",       12;
        0xe1,       "POP HL",       12;
        0xf1,       "POP AF",       12;
        0x02,       "LD (BC),A",    8;
        0x12,       "LD (DE),A",    8;
        0x22,       "LD (HL+),A",   8;      // AKA LD (HLI),A or LDI A,(HL)
        0x32,       "LD (HL-),A",   8;      // AKA LD (HLD),A or LDD A,(HL)
        0x42,       "LD B,D",       4;
        0x52,       "LD D,D",       4;
        0x62,       "LD H,D",       4;
        0x72,       "LD (HL),D",    8;
        0x82,       "ADD A,D",      4;
        0x92,       "SUB D",        4;
        0xa2,       "AND D",        4;
        0xc2,       "JP NZ,a16",    12;
        0xd2,       "JP NC,a16",    12;
        0xe2,       "LD (C),A",     8;      // AKA LD ($FF00+C),A
        0xf2,       "LD A,(C)",     8;      // AKA LD A,($FF00+C)
        0x03,       "INC BC",       8;
        0x13,       "INC DE",       8;
        0x23,       "INC HL",       8;
        0x33,       "INC SP",       8;
        0x43,       "LD B,E",       4;
        0x53,       "LD D,E",       4;
        0x63,       "LD H,E",       4;
        0x73,       "LD (HL),E",    8;
        0x83,       "ADD A,E",      4;
        0x93,       "SUB E",        4;
        0xa3,       "AND E",        4;
        0xc3,       "JP a16",       16;
        0xf3,       "DI",           4;
        0x04,       "INC B",        4;
        0x14,       "INC D",        4;
        0x24,       "INC H",        4;
        0x34,       "INC (HL)",     12;
        0x44,       "LD B,H",       4;
        0x54,       "LD D,H",       4;
        0x64,       "LD H,H",       4;
        0x74,       "LD (HL),H",    8;
        0x84,       "ADD A,H",      4;
        0x94,       "SUB H",        4;
        0xa4,       "AND H",        4;
        0xc4,       "CALL NZ,a16",  12;
        0xd4,       "CALL NC,a16",  12;
        0x05,       "DEC B",        4;
        0x15,       "DEC D",        4;
        0x25,       "DEC H",        4;
        0x35,       "DEC (HL)",     12;
        0x45,       "LD B,L",       4;
        0x55,       "LD D,L",       4;
        0x65,       "LD H,L",       4;
        0x75,       "LD (HL),L",    8;
        0x85,       "ADD A,L",      4;
        0x95,       "SUB L",        4;
        0xa5,       "AND L",        4;
        0xc5,       "PUSH BC",      16;
        0xd5,       "PUSH DE",      16;
        0xe5,       "PUSH HL",      16;
        0xf5,       "PUSH AF",      16;
        0x06,       "LD B,d8",      8;
        0x16,       "LD D,d8",      8;
        0x26,       "LD H,d8",      8;
        0x36,       "LD (HL),d8",   12;
        0x46,       "LD B,(HL)",    8;
        0x56,       "LD D,(HL)",    8;
        0x66,       "LD H,(HL)",    8;
        0x86,       "ADD A,(HL)",   8;
        0x96,       "SUB (HL)",     8;
        0xa6,       "AND (HL)",     8;
        0xc6,       "ADD A,d8",     8;
        0xd6,       "SUB d8",       8;
        0xe6,       "AND d8",       8;
        0x47,       "LD B,A",       4;
        0x57,       "LD D,A",       4;
        0x67,       "LD H,A",       4;
        0x77,       "LD (HL),A",    8;
        0x87,       "ADD A,A",      4;
        0x97,       "SUB A",        4;
        0xa7,       "AND A",        4;
        0xc7,       "RST 00H",      16;
        0xd7,       "RST 10H",      16;
        0xe7,       "RST 20H",      16;
        0xf7,       "RST 30H",      16;
        0x18,       "JR r8",        12;
        0x28,       "JR Z,r8",      8;
        0x38,       "JR C,r8",      8;
        0x48,       "LD C,B",       4;
        0x58,       "LD E,B",       4;
        0x68,       "LD L,B",       4;
        0x78,       "LD A,B",       4;
        0xa8,       "XOR B",        4;
        0xc8,       "RET Z",        8;
        0xd8,       "RET C",        8;
        0x49,       "LD C,C",       4;
        0x59,       "LD E,C",       4;
        0x69,       "LD L,C",       4;
        0x79,       "LD A,C",       4;
        0xa9,       "XOR C",        4;
        0xc9,       "RET",          16;
        0xd9,       "RETI",         16;
        0x0a,       "LD A,(BC)",    8;
        0x1a,       "LD A,(DE)",    8;
        0x2a,       "LD A,(HL+)",   8;      // AKA LD A,(HLI) or LDI A,(HL)
        0x3a,       "LD A,(HL-)",   8;      // AKA LD A,(HLD) or LDD A,(HL)
        0x4a,       "LD C,D",       4;
        0x5a,       "LD E,D",       4;
        0x6a,       "LD L,D",       4;
        0x7a,       "LD A,D",       4;
        0xaa,       "XOR D",        4;
        0xea,       "LD (a16),A",   16;
        0xfa,       "LD A,(a16)",   16;
        0x0b,       "DEC BC",       8;
        0x1b,       "DEC DE",       8;
        0x2b,       "DEC HL",       8;
        0x3b,       "DEC SP",       8;
        0x4b,       "LD C,E",       4;
        0x5b,       "LD E,E",       4;
        0x6b,       "LD L,E",       4;
        0x7b,       "LD A,E",       4;
        0xab,       "XOR E",        4;
        0xcb,       "PREFIX CB",    0;
        0xfb,       "EI",           4;
        0x0c,       "INC C",        4;
        0x1c,       "INC E",        4;
        0x2c,       "INC L",        4;
        0x3c,       "INC A",        4;
        0x4c,       "LD C,H",       4;
        0x5c,       "LD E,H",       4;
        0x6c,       "LD L,H",       4;
        0x7c,       "LD A,H",       4;
        0xac,       "XOR H",        4;
        0xcc,       "CALL Z,a16",   12;
        0xdc,       "CALL C,a16",   12;
        0x0d,       "DEC C",        4;
        0x1d,       "DEC E",        4;
        0x2d,       "DEC L",        4;
        0x3d,       "DEC A",        4;
        0x4d,       "LD C,L",       4;
        0x5d,       "LD E,L",       4;
        0x6d,       "LD L,L",       4;
        0x7d,       "LD A,L",       4;
        0xad,       "XOR L",        4;
        0xcd,       "CALL a16",     24;
        0x0e,       "LD C,d8",      8;
        0x1e,       "LD E,d8",      8;
        0x2e,       "LD L,d8",      8;
        0x3e,       "LD A,d8",      8;
        0x4e,       "LD C,(HL)",    8;
        0x5e,       "LD E,(HL)",    8;
        0x6e,       "LD L,(HL)",    8;
        0x7e,       "LD A,(HL)",    8;
        0xae,       "XOR (HL)",     8;
        0xee,       "XOR d8",       8;
        0xfe,       "CP d8",        8;
        0x4f,       "LD C,A",       4;
        0x5f,       "LD E,A",       4;
        0x6f,       "LD L,A",       4;
        0x7f,       "LD A,A",       4;
        0xaf,       "XOR A",        4;
        0xcf,       "RST 08H",      16;
        0xdf,       "RST 18H",      16;
        0xef,       "RST 28H",      16;
        0xff,       "RST 38H",      16;
    };
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use smallvec::SmallVec;

    use cpu::Cpu;
    use memory::{Addressable, Mmu};

    use super::{INSTRUCTIONS, Instruction};

    #[test]
    fn half_carry() {
        assert!(super::is_half_carry_add(0x0f, 0x01));
        assert!(!super::is_half_carry_add(0x37, 0x44));

        assert!(super::is_half_carry_sub(0xf0, 0x01));
        assert!(!super::is_half_carry_sub(0xff, 0xf0));
    }

    #[test]
    fn carry() {
        assert!(super::is_carry_add(0xff, 0xff));
        assert!(super::is_carry_add(0xff, 0x01));
        assert!(!super::is_carry_add(0x00, 0x01));

        assert!(super::is_carry_sub(0x00, 0x01));
        assert!(!super::is_carry_sub(0xff, 0x0f));
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

        assert_eq!(cpu.reg.pc, 0x38);
        assert_eq!(cpu.pop(), 0xAB + 1);
    }

    #[test]
    fn jr_nz() {
        let mmu = Mmu::default();
        let mut cpu = Cpu::new(Rc::new(RefCell::new(mmu)));

        cpu.reg.pc = 0;

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
    fn jr() {
        let mmu = Mmu::default();
        let mut cpu = Cpu::new(Rc::new(RefCell::new(mmu)));

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

    // FIXME: Add test for 0xf2 after I/O memory is implemented.

    #[test]
    fn ld_addr_a16_a() {
        let mmu = Mmu::default();
        let mut cpu = Cpu::new(Rc::new(RefCell::new(mmu)));

        cpu.reg.a = 0x11;

        let instruction = Instruction {
            def: INSTRUCTIONS[0xea].as_ref().unwrap(),
            operands: SmallVec::from_slice(&[0x00, 0xc0]),
        };
        cpu.execute(instruction);

        assert_eq!(cpu.mmu.borrow().read_byte(0xc000), 0x11);
    }

    #[test]
    fn ld_a_addr_a16() {
        let mmu = Mmu::default();
        let mut cpu = Cpu::new(Rc::new(RefCell::new(mmu)));

        cpu.mmu.borrow_mut().write_byte(0xc000, 0xaa);

        let instruction = Instruction {
            def: INSTRUCTIONS[0xfa].as_ref().unwrap(),
            operands: SmallVec::from_slice(&[0x00, 0xc0]),
        };
        cpu.execute(instruction);

        assert_eq!(cpu.reg.a, 0xaa);
    }

    #[test]
    fn call() {
        let mmu = Mmu::default();
        let mut cpu = Cpu::new(Rc::new(RefCell::new(mmu)));

        cpu.reg.sp = 0xffff;
        cpu.reg.pc = 1;
        cpu.call(4);

        assert_eq!(cpu.reg.pc, 4);
        assert_eq!(cpu.pop(), 1);
    }

    #[test]
    fn ret() {
        let mmu = Mmu::default();
        let mut cpu = Cpu::new(Rc::new(RefCell::new(mmu)));

        cpu.reg.sp = 0xffff;
        cpu.push(5);
        cpu.ret();

        assert_eq!(cpu.reg.sp, 0xffff);
        assert_eq!(cpu.reg.pc, 5);
    }
}
