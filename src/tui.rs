//! Terminal UI.

use std::process;

use errors::*;
use Emulator;

/// The commands that are available to the debugger.
pub static COMMANDS: &str = "sblrpdcq?";

/// Parse and execute a debugger command from a line of input.
///
/// Returns the number of clock cycles executed.
pub fn parse_command(emulator: &mut Emulator, command: &str) -> Result<u32> {
    match &command[..1] {
        "s" => {
            let step = parse_step(command)?.unwrap_or_else(|| 1);

            return Ok((0..step).into_iter().map(|_| emulator.step()).sum());
        }
        "b" => {
            let breakpoint = parse_breakpoint(command)?;
            emulator.add_breakpoint(breakpoint);
        }
        "l" => {
            let breakpoints = emulator.breakpoints();
            if breakpoints.is_empty() {
                println!("no breakpoints");
            } else {
                println!("breakpoints:");
                for breakpoint in emulator.breakpoints() {
                    println!("{:#06x}", breakpoint);
                }
            }
        }
        "r" => emulator.resume(),
        "p" => {
            let (address, instruction) = emulator.current_instruction();
            println!("{:#06x}: {}", address, instruction);
        }
        "d" => println!("{}", emulator.bus.to_string()),
        "c" => println!("{}", emulator.cpu.to_string()),
        "q" => process::exit(0),
        "?" => {
            println!("s: step emulator");
            println!("b: add breakpoint");
            println!("l: list breakpoints");
            println!("r: resume execution");
            println!("p: print current instruction");
            println!("d: dump memory");
            println!("c: cpu state");
            println!("q: quit");
        }
        _ => println!("unknown command"),
    }

    Ok(0)
}

fn parse_step(command: &str) -> Result<Option<i32>> {
    let components = command.split(' ').collect::<Vec<_>>();

    match components.len() {
        1 => return Ok(None),
        2 => (),
        _ => bail!("`s` takes a single optional argument"),
    }

    let step = components[1].parse().chain_err(
        || "could not parse step count",
    )?;

    Ok(Some(step))
}

fn parse_breakpoint(command: &str) -> Result<u16> {
    let components = command.split(' ').collect::<Vec<_>>();

    if components.len() != 2 {
        bail!("`b` takes a single argument");
    }

    let breakpoint = &components[1];
    if !breakpoint.starts_with("0x") {
        bail!("breakpoint must start with '0x'");
    }

    let breakpoint = u16::from_str_radix(&breakpoint[2..], 16).chain_err(
        || "could not parse hexadecimal number",
    )?;
    Ok(breakpoint)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_breakpoint() {
        assert_eq!(super::parse_breakpoint("b 0x174").unwrap(), 0x174);
    }
}
