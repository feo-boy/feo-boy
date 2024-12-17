use test_each_file::test_each_file;
use serde::Deserialize;

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
    let mut cpu = Cpu::new();
    let mut bus = Bus::default();

    cpu.reg.a = test_case.initial.a;
    cpu.reg.b = test_case.initial.b;
    cpu.reg.c = test_case.initial.c;
    cpu.reg.d = test_case.initial.d;
    cpu.reg.e = test_case.initial.e;
    cpu.reg.f = Flags::from_bits(test_case.initial.f).unwrap();
    cpu.reg.h = test_case.initial.h;
    cpu.reg.l = test_case.initial.l;

    for (addr, value) in test_case.initial.ram {
        bus.write_byte_no_tick(addr, value);
    }

    for cycle in test_case.cycles {
        bus.tick(MCycles(1));
    }

    assert_eq!(test_case.r#final.a, cpu.reg.a);
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
