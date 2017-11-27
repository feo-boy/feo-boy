/// The clock.
#[derive(Debug, Default)]
pub struct Clock {
    /// Machine cycle state. One machine cycle = 4 clock cycles.
    m: u32,

    /// Clock cycle state.
    t: u32,

    diff: u32,
}

impl Clock {
    pub fn new() -> Self {
        Default::default()
    }

    /// Tick a number of T-cycles.
    pub fn tick(&mut self, t: u32) {
        self.t += t;
        self.m += t / 4;
        self.diff = t;
    }

    /// Return the number of T-cycles ticked in the last tick.
    pub fn diff(&self) -> u32 {
        self.diff
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::Clock;

    #[test]
    fn tick() {
        let mut clock = Clock::new();

        clock.tick(4);
        assert_eq!(clock.m, 1);
        assert_eq!(clock.t, 4);
        assert_eq!(clock.diff(), 4);

        clock.tick(12);
        assert_eq!(clock.m, 4);
        assert_eq!(clock.t, 16);
        assert_eq!(clock.diff(), 12);
    }
}
