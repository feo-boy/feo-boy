use std::cell::RefCell;
use std::rc::Rc;

use cpu::Cpu;
use memory::Mmu;

#[allow(unused)]

pub struct Gpu {
    mmu: Rc<RefCell<Mmu>>,
    cpu: Rc<RefCell<Cpu>>,
}

impl Gpu {
    pub fn new(mmu: Rc<RefCell<Mmu>>, cpu: Rc<RefCell<Cpu>>) -> Gpu {
        Gpu { mmu: mmu, cpu: cpu }
    }
}
