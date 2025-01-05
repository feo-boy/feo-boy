use feo_boy::Emulator;
use serde::Deserialize;
use test_each_file::test_each_file;

use feo_boy::bus::Bus;
use feo_boy::cpu::{Cpu, Flags, MCycles};

test_each_file! { in "./tests/sm83/v1" as sm83 => load_test }

fn load_test(content: &str) {
    let test_cases: Vec<TestCase> = serde_json::from_str(content).unwrap();

    for test_case in test_cases {
        test(test_case);
    }
}

fn test(test_case: TestCase) {
    let mut emulator = Emulator::new();

    emulator.cpu.reg.a = test_case.initial.a;
    emulator.cpu.reg.b = test_case.initial.b;
    emulator.cpu.reg.c = test_case.initial.c;
    emulator.cpu.reg.d = test_case.initial.d;
    emulator.cpu.reg.e = test_case.initial.e;
    emulator.cpu.reg.f = Flags::from_bits(test_case.initial.f).unwrap();
    emulator.cpu.reg.h = test_case.initial.h;
    emulator.cpu.reg.l = test_case.initial.l;
    emulator.cpu.reg.pc = test_case.initial.pc;
    emulator.cpu.reg.sp = test_case.initial.sp;

    for (addr, value) in test_case.initial.ram {
        emulator.bus.write_byte_no_tick(addr, value);
    }

    emulator.step();

    macro_rules! assert_reg {
        ($reg:ident) => {
            let expected = test_case.r#final.$reg;
            let actual = emulator.cpu.reg.$reg;
            assert_eq!(
                expected,
                actual,
                "expected register {} to be {:#02x}, was {:#02x}",
                stringify!($reg).to_uppercase(),
                expected,
                actual
            );
        };
    }

    // assert_reg!(pc); FIXME
    assert_reg!(sp);
    assert_reg!(a);
    assert_reg!(b);
    assert_reg!(c);
    assert_reg!(d);
    assert_reg!(e);
    assert_reg!(h);
    assert_reg!(l);
}

#[derive(Debug, Deserialize)]
struct TestCase {
    name: String,
    initial: State,
    r#final: State,
    cycles: Vec<(u16, Option<u8>, String)>,
}

#[derive(Debug, Deserialize)]
struct State {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    #[serde(skip)]
    ime: Option<u8>,
    #[serde(skip)]
    ei: Option<u8>,
    ram: Vec<(u16, u8)>,
}
