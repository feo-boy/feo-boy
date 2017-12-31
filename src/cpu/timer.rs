//! CPU timer management.

use std::fmt::{self, Display};

use bytes::ByteExt;

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Add, AddAssign)]
pub struct MCycles(pub u32);

impl Display for MCycles {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} M-cycles", self.0)
    }
}

impl From<TCycles> for MCycles {
    fn from(t_cycles: TCycles) -> Self {
        MCycles(t_cycles.0 / 4)
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Add, AddAssign)]
pub struct TCycles(pub u32);

impl Display for TCycles {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} T-cycles", self.0)
    }
}

impl From<MCycles> for TCycles {
    fn from(m_cycles: MCycles) -> Self {
        TCycles(m_cycles.0 * 4)
    }
}

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
    div_counter: u32,

    // TODO: We might be able to just use `div_counter` for this.
    /// Timer internal counter.
    timer_counter: u32,

    /// The amount of time ticked since the last call to `reset_diff`.
    diff: u32,

    pub reg: TimerRegisters,
}

impl Timer {
    /// Increment all timer-related registers, based on the M-time of the last instruction.
    ///
    /// Requests the timer interrupt if necessary.
    pub fn tick(&mut self, mtime: MCycles, interrupt_requested: &mut bool) {
        self.diff += mtime.0;
        self.div_counter += mtime.0;

        // The divider is always counting, regardless of whether the timer is enabled.
        while self.div_counter >= 64 {
            self.div_counter -= 64;
            self.reg.divider = self.reg.divider.wrapping_add(1);
        }

        if !self.is_enabled() {
            return;
        }

        self.timer_counter += mtime.0;

        // The timer will increment at a frequency determined by the control register.
        let threshold = match self.reg.control & 0x3 {
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
        while self.timer_counter >= threshold {
            self.timer_counter -= threshold;

            let (counter, overflow) = match self.reg.counter.checked_add(1) {
                Some(counter) => (counter, false),
                None => (self.reg.modulo, true),
            };

            self.reg.counter = counter;

            if overflow {
                *interrupt_requested = true;
            }
        }
    }

    /// Returns the number of M-cycles that have passed since the last call of this method.
    pub fn diff(&self) -> MCycles {
        MCycles(self.diff)
    }

    pub fn reset_diff(&mut self) {
        self.diff = 0;
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

    use super::{MCycles, Timer};

    #[test]
    fn div() {
        let mut interrupt_requested = false;
        let mut timer = Timer::default();

        for _ in 0..64 {
            timer.tick(MCycles(1), &mut interrupt_requested);
        }

        assert_eq!(timer.reg.divider(), 1);

        for _ in 0..128 {
            timer.tick(MCycles(1), &mut interrupt_requested);
        }

        assert_eq!(timer.reg.divider(), 3);
    }

    #[test]
    fn reset_div() {
        let mut interrupt_requested = false;
        let mut timer = Timer::default();

        for _ in 0..63 {
            timer.tick(MCycles(1), &mut interrupt_requested);
        }
        assert_eq!(timer.reg.divider(), 0);

        timer.reset_divider();
        assert_eq!(timer.reg.divider(), 0);

        for _ in 0..63 {
            timer.tick(MCycles(1), &mut interrupt_requested);
        }
        assert_eq!(timer.reg.divider(), 0);

        timer.tick(MCycles(1), &mut interrupt_requested);
        assert_eq!(timer.reg.divider(), 1);
    }

    #[test]
    fn tima() {
        let mut interrupt_requested = false;

        // Enable timer, increment every 64 M-cycles.
        let mut timer = Timer::default();
        timer.reg.control = 0x07;

        for _ in 0..63 {
            timer.tick(MCycles(1), &mut interrupt_requested);
        }
        assert_eq!(timer.reg.counter, 0);

        timer.tick(MCycles(1), &mut interrupt_requested);
        assert_eq!(timer.reg.counter, 1);

        // Enable timer, increment every 4 M-cycles.
        let mut timer = Timer::default();
        timer.reg.control = 0x05;

        timer.tick(MCycles(16), &mut interrupt_requested);
        assert_eq!(timer.reg.counter, 4);
    }

    #[test]
    fn tima_overflow() {
        let mut interrupt_requested = false;

        // Enable timer, increment every 4 M-cycles.
        let mut timer = Timer::default();
        timer.reg.control = 0x05;

        // The number of M-cycles it will take to trigger an interrupt, divided by 8 iterations.
        const INCREMENT: MCycles = MCycles(((u8::MAX as u16 * 4) / 8) as u32);

        for _ in 0..8 {
            timer.tick(INCREMENT, &mut interrupt_requested);
            assert!(!interrupt_requested);
        }

        timer.tick(INCREMENT, &mut interrupt_requested);
        assert!(interrupt_requested);
    }
}
