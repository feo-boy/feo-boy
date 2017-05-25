//! CPU instruction definition.

#[derive(Debug, Clone)]
pub struct Instruction {
    pub byte: u8,
    pub description: String,
    pub cycles: u8,
    pub operands: u8,
}

/// Macro to quickly define all CPU instructions for the Game Boy Z80 processor.
macro_rules! instructions {
    ( $( $byte:expr, $description:expr, $operands:expr, $cycles:expr ; )* ) => {
        {
            let mut instructions = vec![None; 0x100];

            $(
                instructions[$byte] = Some(Instruction {
                    byte: $byte,
                    description: ($description).into(),
                    cycles: $cycles,
                    operands: $operands,
                });
            )*

            instructions
        }
    }
}

pub fn fetch(byte: u8) -> &'static Instruction {
    INSTRUCTIONS[byte as usize]
        .as_ref()
        .expect(&format!("could not find data for instruction 0x{:0x}", byte))
}

lazy_static! {

    /// Game Boy instruction set.
    ///
    /// Timings and other information taken from [here].
    ///
    /// [here]: http://pastraiser.com/cpu/gameboy/gameboy_opcodes.html
    // FIXME: This should be `[Instruction; 0x100]` once all instructions are implemented.
    static ref INSTRUCTIONS: Vec<Option<Instruction>> = instructions! {
        // byte     description      operands        cycles
        0x00,       "NOP",           0,              1;
        0x31,       "LD SP,d16",     2,              12;
    };
}
