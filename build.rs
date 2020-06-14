//! Generates instruction definitions from the data in the `definitions` subdirectory.

#[macro_use]
extern crate quote;
#[macro_use]
extern crate serde_derive;

extern crate csv;
extern crate serde;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Instruction {
    #[serde(deserialize_with = "deserialize_hex_literal")]
    byte: u8,
    mnemonic: String,
    cycles: u32,
    condition_cycles: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrefixInstruction {
    #[serde(deserialize_with = "deserialize_hex_literal")]
    byte: u8,
    mnemonic: String,
}

fn deserialize_hex_literal<'de, D>(deserializer: D) -> std::result::Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    let string = string.trim_start_matches("0x");
    u8::from_str_radix(string, 16).map_err(serde::de::Error::custom)
}

fn parse_operands(description: &str) -> u8 {
    if description.contains("d8")
        || description.contains("a8")
        || description.contains("r8")
        || description.contains("PREFIX CB")
    {
        1
    } else if description.contains("d16") || description.contains("a16") {
        2
    } else {
        0
    }
}

fn parse_prefix_cycles(description: &str) -> u32 {
    // If the instruction accesses memory (through HL), the instruction will take 16 cycles.
    // Otherwise, it will take 8.
    if description.contains("(HL)") {
        // However, the BIT instruction is slightly faster.
        if description.contains("BIT") {
            12
        } else {
            16
        }
    } else {
        8
    }
}

fn write_instructions<P: AsRef<Path>>(filename: P) -> Result<()> {
    let instruction_definitions = File::open("definitions/instructions.tsv")?;
    let mut instruction_definitions = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(instruction_definitions);

    let mut instructions_out = File::create(filename)?;

    writeln!(instructions_out, "[")?;

    let mut instructions = vec![];
    for result in instruction_definitions.deserialize() {
        let instruction: Instruction = result?;
        instructions.push(instruction);
    }
    instructions.sort_unstable_by_key(|i| i.byte);

    for instruction in &instructions {
        let operands = parse_operands(&instruction.mnemonic);
        let Instruction {
            ref byte,
            ref mnemonic,
            ref cycles,
            ..
        } = *instruction;
        let condition_cycles = match instruction.condition_cycles {
            Some(cycles) => quote! { Some(TCycles(#cycles)) },
            None => quote! { None },
        };
        writeln!(
            instructions_out,
            "{}",
            quote! {
                InstructionDef {
                    byte: #byte,
                    description: #mnemonic,
                    num_operands: #operands,
                    cycles: TCycles(#cycles),
                    condition_cycles: #condition_cycles,
                },
            }
        )?;
    }

    write!(instructions_out, "]")?;

    Ok(())
}

fn write_prefix_instructions<P: AsRef<Path>>(filename: P) -> Result<()> {
    let instruction_definitions = File::open("definitions/prefix.tsv")?;
    let mut instruction_definitions = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(instruction_definitions);

    let mut instructions_out = File::create(filename)?;

    writeln!(instructions_out, "[")?;

    let mut instructions = vec![];
    for result in instruction_definitions.deserialize() {
        let instruction: PrefixInstruction = result?;
        instructions.push(instruction);
    }
    instructions.sort_unstable_by_key(|i| i.byte);

    for instruction in &instructions {
        let cycles = parse_prefix_cycles(&instruction.mnemonic);
        let PrefixInstruction {
            ref byte,
            ref mnemonic,
            ..
        } = *instruction;
        writeln!(
            instructions_out,
            "{}",
            quote! {
                PrefixInstructionDef {
                    byte: #byte,
                    description: #mnemonic,
                    cycles: TCycles(#cycles),
                },
            }
        )?;
    }

    write!(instructions_out, "]")?;

    Ok(())
}

fn main() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    write_instructions(out_dir.join("instructions.rs"))?;
    write_prefix_instructions(out_dir.join("prefix_instructions.rs"))?;

    Ok(())
}
