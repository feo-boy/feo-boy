//! CPU timer management.

use bytes::ByteExt;

#[derive(Debug, Default)]
pub struct TimerRegisters {
    pub counter: u8,
    pub modulo: u8,
    pub control: u8,

    divider: u8,
}

impl TimerRegisters {
    /// Returns the value of the divider register.
    pub fn divider(&self) -> u8 {
        self.divider
    }
}

#[derive(Debug, Default)]
pub struct Timer {
    /// Divider internal counter. Increments the divider register once it reaches `64` M-cycles.
    div: u8,

    /// Timer internal counter.
    timer_counter: u16,

    pub reg: TimerRegisters,
}

impl Timer {
    /// Increment all timer-related registers, based on the M-time of the last instruction.
    ///
    /// Returns whether the timer interrupt should be triggered.
    pub fn tick(&mut self, mtime: u8) -> bool {
        self.div += mtime;

        // The divider is always counting, regardless of whether the timer is enabled.
        if self.div >= 64 {
            self.div %= 64;
            self.reg.divider = self.reg.divider.wrapping_add(1);
        }

        if !self.is_enabled() {
            return false;
        }

        self.timer_counter += u16::from(mtime);

        // The timer will increment at a frequency determined by the control register.
        let threshold: u16 = match self.reg.control & 0x3 {
            0 => 256,       // 4KHz
            1 => 4,         // 256KHz
            2 => 16,        // 64KHz
            3 => 64,        // 16KHz
            _ => unreachable!(),
        };

        if self.timer_counter >= threshold {
            self.timer_counter %= threshold;

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

    pub fn reset_divider(&mut self) {
        self.reg.divider = 0;
        self.div = 0;
    }

    pub fn is_enabled(&self) -> bool {
        self.reg.control.has_bit_set(2)
    }
}

#[cfg(test)]
mod tests {
    use bus::Bus;

    use super::Timer;

    #[test]
    fn div() {
        let mut timer = Timer::default();

        for _ in 0..64 {
            timer.tick(1);
        }

        assert_eq!(timer.reg.divider(), 1);

        for _ in 0..128 {
            timer.tick(1);
        }

        assert_eq!(timer.reg.divider(), 3);
    }

    #[test]
    fn reset_div() {
        let mut timer = Timer::default();

        for _ in 0..63 {
            timer.tick(1);
        }
        assert_eq!(timer.reg.divider(), 0);

        timer.reset_divider();
        assert_eq!(timer.reg.divider(), 0);

        for _ in 0..63 {
            timer.tick(1);
        }
        assert_eq!(timer.reg.divider(), 0);

        timer.tick(1);
        assert_eq!(timer.reg.divider(), 1);
    }
}
