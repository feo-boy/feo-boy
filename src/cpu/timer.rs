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
    div_counter: u8,

    // TODO: We might be able to just use `div_counter` for this.
    /// Timer internal counter.
    timer_counter: u16,

    pub reg: TimerRegisters,
}

impl Timer {
    /// Increment all timer-related registers, based on the M-time of the last instruction.
    ///
    /// Returns whether the timer interrupt should be triggered.
    pub fn tick(&mut self, mtime: u8) -> bool {
        self.div_counter += mtime;

        // The divider is always counting, regardless of whether the timer is enabled.
        while self.div_counter >= 64 {
            self.div_counter -= 64;
            self.reg.divider = self.reg.divider.wrapping_add(1);
        }

        if !self.is_enabled() {
            return false;
        }

        self.timer_counter += u16::from(mtime);

        // The timer will increment at a frequency determined by the control register.
        let threshold: u16 = match self.reg.control & 0x3 {
            0 => 256, // 4KHz
            1 => 4,   // 256KHz
            2 => 16,  // 64KHz
            3 => 64,  // 16KHz
            _ => unreachable!(),
        };

        // NB: This is the source of a very common bug in timer implementations.
        //
        // Here, we need to increment the timer's internal counter relative to the tick size. The
        // counter may have to be incremented multiple times for a given tick. While this
        // technically could happen for the div internal counter, in practice it doesn't: no
        // instruction takes longer to execute than it takes to increment DIV once. However, it
        // _is_ possible to have the timer internal counter increment multiple times during a given
        // instruction.
        //
        // Notably, getting this wrong will cause blargg's instr_timing test ROM to fail with
        // the cryptic "Failure #255" message.
        let mut timer_overflow = false;
        while self.timer_counter >= threshold {
            self.timer_counter -= threshold;

            let (counter, overflow) = match self.reg.counter.checked_add(1) {
                Some(counter) => (counter, false),
                None => (self.reg.modulo, true),
            };

            self.reg.counter = counter;
            timer_overflow |= overflow;
        }

        timer_overflow
    }

    pub fn reset_divider(&mut self) {
        self.reg.divider = 0;
        self.div_counter = 0;
        self.timer_counter = 0;
    }

    pub fn is_enabled(&self) -> bool {
        self.reg.control.has_bit_set(2)
    }
}

#[cfg(test)]
mod tests {
    use std::u8;

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

    #[test]
    fn tima() {
        // Enable timer, increment every 64 M-cycles.
        let mut timer = Timer::default();
        timer.reg.control = 0x07;

        for _ in 0..63 {
            timer.tick(1);
        }
        assert_eq!(timer.reg.counter, 0);

        timer.tick(1);
        assert_eq!(timer.reg.counter, 1);

        // Enable timer, increment every 4 M-cycles.
        let mut timer = Timer::default();
        timer.reg.control = 0x05;

        timer.tick(16);
        assert_eq!(timer.reg.counter, 4);
    }

    #[test]
    fn tima_overflow() {
        // Enable timer, increment every 4 M-cycles.
        let mut timer = Timer::default();
        timer.reg.control = 0x05;

        // The number of M-cycles it will take to trigger an interrupt, divided by 8 iterations.
        const INCREMENT: u16 = (u8::MAX as u16 * 4) / 8;

        for _ in 0..8 {
            let interrupt_requested = timer.tick(INCREMENT as u8);
            assert!(!interrupt_requested);
        }

        let interrupt_requested = timer.tick(INCREMENT as u8);
        assert!(interrupt_requested);
    }
}
