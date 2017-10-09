//! CPU timer management.

use bytes::ByteExt;

#[derive(Debug, Default)]
pub struct TimerRegisters {
    pub divider: u8,
    pub counter: u8,
    pub modulo: u8,
    pub control: u8,
}

#[derive(Debug, Default)]
pub struct Timer {
    main: u8,
    sub: u8,
    div: u8,

    pub reg: TimerRegisters,
}

impl Timer {
    /// Increment all timer-related registers, based on the M-time of the last instruction.
    ///
    /// Returns whether the timer interrupt should be triggered.
    pub fn tick(&mut self, mtime: u8) -> bool {
        self.sub += mtime;

        if self.sub >= 4 {
            self.main = self.main.wrapping_add(1);
            self.sub -= 4;

            self.div += 1;
            if self.div == 16 {
                self.reg.divider = self.reg.divider.wrapping_add(1);
                self.div = 0;
            }
        }

        if !self.reg.control.has_bit_set(2) {
            return false;
        }

        let threshold = match self.reg.control & 0x3 {
            0 => 64,    // 4K
            1 => 1,     // 256K
            2 => 4,     // 64K
            3 => 16,    // 16K
            _ => unreachable!(),
        };

        if self.main >= threshold {
            self.main = 0;

            let (counter, overflow) = match self.reg.counter.checked_add(1) {
                Some(counter) => (counter, false),
                None => (self.reg.modulo, true),
            };

            self.reg.counter = counter;
            overflow
        } else {
            false
        }
    }
}
