use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use libtest_mimic::{Arguments, Test, Outcome, run_tests};
use walkdir::WalkDir;

use feo_boy::Emulator;

const MAX_DURATION: Duration = Duration::from_secs(120);
const TEST_ROOT: &str = "./tests/mooneye-gb/tests/build";

fn assert_rom(rom: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let rom = rom.as_ref();

    let mut emulator = Emulator::new();

    emulator.load_rom(&fs::read(rom)?)?;
    emulator.reset();
    emulator.update(MAX_DURATION)?;

    let regs = &emulator.cpu.reg;
    if regs.b != 3 || regs.c != 5 || regs.d != 8 || regs.e != 13 || regs.h != 21 || regs.l != 34 {
        return Err(format!("test failed\n\n{}", regs).into());
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if which::which("wla-gb").is_err() {
        println!("wla-gb not installed, skipping tests");
        return Ok(());
    }

    let output = Command::new("make")
        .current_dir("tests/mooneye-gb/tests")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        return Err("command failed".into());
    }

    let args = Arguments::from_args();

    let mut tests = vec![];

    for entry in WalkDir::new(TEST_ROOT) {
        let entry = entry?;

        if entry.path().extension().map(|ext| ext == "gb").unwrap_or(false) {
            tests.push(Test {
                name: entry.path().strip_prefix(TEST_ROOT).unwrap().to_str().unwrap().into(),
                kind: "".into(),
                is_ignored: false,
                is_bench: false,
                data: entry.path().to_owned(),
            });
        }
    }

    run_tests(&args, tests, |test| {
        match assert_rom(&test.data) {
            Ok(_) => Outcome::Passed,
            Err(e) => Outcome::Failed { msg: Some(e.to_string()) },
        }
    }).exit();
}
