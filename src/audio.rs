//! Audio-related functionality.
//!
//! Contains an implementation of the Game Boy sound hardware.

use bytes::ByteExt;

use memory::Addressable;

/// The sweep register data for a channel.
#[derive(Debug, Default)]
pub struct Sweep {
    /// The sweep time.
    pub time: u8,

    /// Whether the sweep increases (if false) or decreases (if true) the frequency.
    pub decrease: bool,

    /// The sweep shift number.
    pub shift: u8,
}

impl Sweep {
    /// Gets the result of reading the sweep register for the current sweep state.
    pub fn read(&self) -> u8 {
        let mut byte = self.time << 4;
        byte |= self.shift;
        byte.set_bit(3, self.decrease);

        byte
    }

    /// Modifies the sweep state according to the written byte.
    pub fn write(&mut self, byte: u8) {
        self.shift = byte & 0x7;
        self.decrease = byte.has_bit_set(3);
        self.time = (byte >> 4) & 0x7;
    }
}

/// The sound length/wave pattern duty for a channel.
#[derive(Debug, Default)]
pub struct Wave {
    /// Wave pattern duty (0-3).
    pub pattern: u8,

    /// Sound length (0-63).
    pub length: u8,
}

impl Wave {
    /// Gets the result of reading the wave register for the current register state.
    pub fn read(&self) -> u8 {
        let mut byte = self.pattern << 6;
        byte |= 0x3F;

        byte
    }

    /// Modifies the wave state according to the written byte.
    pub fn write(&mut self, byte: u8) {
        self.length = byte & 0x3F;
        self.pattern = byte >> 6;
    }
}

/// The volume envelope for a channel.
#[derive(Debug, Default)]
pub struct Envelope {
    /// The initial volume of the envelope (0-15).
    pub initial_vol: u8,

    /// The direction of the envelope - `true` means increase, `false` means decrease.
    pub direction_increase: bool,

    /// Number of envelope sweep (0-7). The length of one step of the sweep is n * (1/64).
    pub number: u8,
}

impl Envelope {
    /// Gets the result of reading the envelope register for the current register state.
    pub fn read(&self) -> u8 {
        let mut byte = self.initial_vol << 4;
        byte.set_bit(3, self.direction_increase);
        byte |= self.number;

        byte
    }

    /// Modifies the envelope state according to the written byte.
    pub fn write(&mut self, byte: u8) {
        self.initial_vol = byte >> 4;
        self.direction_increase = byte.has_bit_set(3);
        self.number = byte & 0x7;
    }
}

/// The frequency data for a channel.
#[derive(Debug, Default)]
pub struct Frequency {
    /// Initial (`true` = restart sound) (Write only).
    pub initial: bool,

    /// Counter/consecutive selection (`true` = stop output when length in wave pattern duty
    /// expires).
    pub counter: bool,

    /// The 11-bit frequency (Write only).
    pub frequency: u16,
}

impl Frequency {
    /// Modifies the lower 8 bits of the 11-bit frequency according to the written byte.
    pub fn write_lo(&mut self, byte: u8) {
        self.frequency = (self.frequency & 0xFF00) | u16::from(byte);
    }

    /// Gets the result of reading the high bits of the frequency data.
    pub fn read_hi(&self) -> u8 {
        let mut byte = 0xFF;
        byte.set_bit(6, self.counter);

        byte
    }

    /// Modifies the high bits of the frequency data according to the written byte.
    pub fn write_hi(&mut self, byte: u8) {
        self.initial = byte.has_bit_set(7);
        self.counter = byte.has_bit_set(6);
        self.frequency = ((u16::from(byte & 0x7)) << 8) | (self.frequency & 0xFF);
    }
}

/// The sound length for channel 3.
#[derive(Debug, Default)]
pub struct BigLength {
    /// Sound length (0-255)
    pub length: u8,
}

// TODO test
impl BigLength {
    /// Gets the result of reading the BigLength register for the current register state.
    pub fn read(&self) -> u8 {
        self.length
    }

    /// Modifies the BigLength state according to the written byte.
    pub fn write(&mut self, byte: u8) {
        self.length = byte;
    }
}

/// The output level.
#[derive(Debug, Default)]
pub struct OutputLevel {
    /// The output level (0-3).
    pub output_level: u8,
}

// TODO test
impl OutputLevel {
    /// Gets the result of reading the output level for the current register state.
    pub fn read(&self) -> u8 {
        self.output_level
    }

    /// Modifies the output level state according to the written byte.
    pub fn write(&mut self, byte: u8) {
        self.output_level = (byte >> 5) & 0x3;
    }
}

/// The sound length for channel 4.
#[derive(Debug, Default)]
pub struct Length {
    /// Sound length (0-63).
    pub length: u8,
}

// TODO test
impl Length {
    /// Gets the result of reading the length register for the current register state.
    pub fn read(&self) -> u8 {
        self.length
    }

    /// Modifies the length state according to the written byte.
    pub fn write(&mut self, byte: u8) {
        self.length = byte & 0x3F;
    }
}

/// The polynomial counter for channel 4.
#[derive(Debug, Default)]
pub struct PolynomialCounter {
    /// The shift clock frequency.
    pub shift_clock_frequency: u8,

    /// Counter step/width (false=15 bits, true=7 bits).
    pub counter_step: bool,

    /// Dividing ratio of frequencies.
    pub divide_ratio: u8,
}

// TODO test
impl PolynomialCounter {
    /// Gets the result of reading the PolynomialCounter state for the current register state.
    pub fn read(&self) -> u8 {
        let mut byte = self.shift_clock_frequency << 4;
        byte |= self.divide_ratio;
        byte.set_bit(3, self.counter_step);

        byte
    }

    /// Modifies the PolynomialCounter state according to the written byte.
    pub fn write(&mut self, byte: u8) {
        self.shift_clock_frequency = byte >> 4;
        self.counter_step = byte.has_bit_set(3);
        self.divide_ratio = byte & 0x7;
    }
}

/// The counter/consecutive selection and initial flag.
#[derive(Debug, Default)]
pub struct InitialCounterConsecutive {
    /// Initial flag (true = restart sound) - write only.
    pub initial: bool,

    /// Counter/consecutive selection (true = stop output when length in NR41 expires).
    pub counter: bool,
}

// TODO test
impl InitialCounterConsecutive {
    /// Gets the result of reading the InitialCounterConsecutive state for the current register
    /// state.
    pub fn read(&self) -> u8 {
        let mut byte = 0;
        byte.set_bit(6, self.counter);

        byte
    }

    /// Modifies the InitialCounterConsecutive state according to the written byte.
    pub fn write(&mut self, byte: u8) {
        self.initial = byte.has_bit_set(7);
        self.counter = byte.has_bit_set(6);
    }
}

/// A single Game Boy sound channel.
#[derive(Debug, Default)]
pub struct Sound {
    /// Whether or not the sound is enabled.
    pub is_on: bool,

    /// Whether to output this sound to SO1 terminal.
    pub so1_enabled: bool,

    /// Whether to output this sound to SO2 terminal.
    pub so2_enabled: bool,

    /// The sweep register data.
    pub sweep: Sweep,

    /// The sound length/wave pattern data.
    pub wave: Wave,

    /// The volume envelope data.
    pub envelope: Envelope,

    /// The frequency data.
    pub frequency: Frequency,
}

/// Sound channel 3.
#[derive(Debug, Default)]
pub struct Sound3 {
    /// Whether or not the sound is enabled.
    pub is_on: bool,

    /// Whether to output this sound to SO1 terminal.
    pub so1_enabled: bool,

    /// Whether to output this sound to SO2 terminal.
    pub so2_enabled: bool,

    /// The volume level of this sound.
    pub output_level: OutputLevel,

    /// The sound length.
    pub length: BigLength,

    /// The frequency data.
    pub frequency: Frequency,

    /// The wave pattern memory for storing arbitrary sound data. Holds 32 4-bit samples.
    pub wave_pattern: [u8; 16],
}

/// Sound channel 4.
#[derive(Debug, Default)]
pub struct Sound4 {
    /// Whether or not the sound is enabled.
    pub is_on: bool,

    /// Whether to output this sound to the SO1 terminal.
    pub so1_enabled: bool,

    /// Whether to output this sound to the SO2 terminal.
    pub so2_enabled: bool,

    /// The sound length.
    pub length: Length,

    /// The volumne envelope data.
    pub envelope: Envelope,

    /// The polynomial counter.
    pub polynomial_counter: PolynomialCounter,

    /// The initial flag and counter/consecutive selection.
    pub initial_counter_consecutive: InitialCounterConsecutive,
}

/// The controller for the four sound channels output by the Game Boy.
#[derive(Debug, Default)]
pub struct SoundController {
    /// Sound 1: Rectangle waveform with sweep and envelope functions.
    pub sound_1: Sound,

    /// Sound 2: Rectangle waveform with envelope function.
    pub sound_2: Sound,

    /// Sound 3: A waveform specificed by the waveform RAM.
    pub sound_3: Sound3,

    /// Sound 4: White noise with an envelope function.
    pub sound_4: Sound4,

    /// Toggle whether or not sound is enabled.
    pub sound_enabled: bool,

    /// The volume to output to the SO1 terminal.
    pub so1_vol: u8,

    /// The volume to output to the SO2 terminal.
    pub so2_vol: u8,

    /// Whether to output Vin to SO1.
    pub vin_so1: bool,

    /// Whether to output Vin to SO2.
    pub vin_so2: bool,
}

impl SoundController {
    pub fn new() -> SoundController {
        SoundController::default()
    }
}

impl Addressable for SoundController {
    /// Reads a byte of audio memory. Unreadable memory is always read as 1s.
    ///
    /// # Panics
    ///
    /// Panics if reading memory that is not managed by the sound controller.
    fn read_byte(&self, address: u16) -> u8 {
        // Access to sound registers, aside from 0xFF26, is disabled unless the sound is on.
        if !self.sound_enabled && address != 0xFF26 {
            return 0xFF;
        }

        #[cfg_attr(feature = "cargo-clippy", allow(match_same_arms))]
        match address {
            // NR10: Sound 1 sweep register
            // Bit 6-4 - Sweep time
            // Bit 3   - Sweep Increase/Decrease
            //            0: Addition    (frequency increases)
            //            1: Subtraction (frequency decreases)
            // Bit 2-0 - Number of sweep shift (n: 0-7)
            0xFF10 => self.sound_1.sweep.read(),

            // NR11: Sound 1 Sound length/Wave pattern duty
            // Bit 7-6 - Wave pattern duty
            // Bit 5-0 - Sound length data (Write only)
            0xFF11 => self.sound_1.wave.read(),

            // NR12: Channel 1 volume envelope
            // Bit 7-4 - Initial volume of the envelope (0-15) (0 = no sound)
            // Bit 3   - Envelope direction (0 = decrease, 1 = increase)
            // Bit 2-0 - Number of envelope sweep (n: 0-7) (If 0, stop the envelope operation)
            0xFF12 => self.sound_1.envelope.read(),

            // NR13: Channel 1 frequency low
            // Unreadable.
            0xFF13 => 0xFF,

            // NR14: Channel 1 frequency high
            // Bit 7   - Initial (1 = restart sound) (write only)
            // Bit 6   - Counter/consecutive selection (1 = stop output when length in NR11
            //           expires)
            // Bit 2-0 - Frequency's higher 3 bits (write only)
            0xFF14 => self.sound_1.frequency.read_hi(),

            // NR21: Sound 2 Sound length/Wave pattern duty
            // Bit 7-6 - Wave pattern duty
            // Bit 5-0 - Sound length data (Write only)
            0xFF16 => self.sound_2.wave.read(),

            // NR22: Channel 2 volume envelope
            // Bit 7-4 - Initial volume of the envelope (0-15) (0 = no sound)
            // Bit 3   - Envelope direction (0 = decrease, 1 = increase)
            // Bit 2-0 - Number of envelope sweep (n: 0-7) (If 0, stop the envelope operation)
            0xFF17 => self.sound_2.envelope.read(),

            // NR23: Channel 2 frequency low
            // Unreadable.
            0xFF18 => 0xFF,

            // NR24: Channel 2 frequency high
            // Bit 7   - Initial (1 = restart sound) (write only)
            // Bit 6   - Counter/consecutive selection (1 = stop output when length in NR21
            //           expires)
            // Bit 2-0 - Frequency's higher 3 bits (write only)
            0xFF19 => self.sound_2.frequency.read_hi(),

            // NR30: Channel 3 sound on/off
            // Bit 7 - Sound channel 3 off (0=Stop, 1=Playback)
            0xFF1A => if self.sound_3.is_on { 1 } else { 0 },

            // NR31: Channel 3 sound length
            // Bit 7-0 - Sound length (0-255)
            0xFF1B => self.sound_3.length.read(),

            // NR32: Channel 3 select output level
            // Bit 6-5 - Select output level:
            //           0 - mute
            //           1 - 100% volume
            //           2 - 50% volume
            //           3 - 25% volume
            0xFF1C => self.sound_3.output_level.read(),

            // NR33: Channel 3 frequency low
            // Unreadable.
            0xFF1D => 0xFF,

            // NR34: Channel 3 frequency high
            // Bit 7   - Initial (1 = restart sound) (write only)
            // Bit 6   - Counter/consecutive selection (1 = stop output when length in NR11
            //           expires)
            // Bit 2-0 - Frequency's higher 3 bits (write only)
            0xFF1E => self.sound_3.frequency.read_hi(),

            // NR41: Channel 4 sound length
            // Bit 5-0 - Sound length data (0-63)
            0xFF20 => self.sound_4.length.read(),

            // NR42: Channel 4 volume envelope
            // Bit 7-4 - Initial volume of envelope (0=No sound)
            // Bit 3   - Envelope direction (0=Decrease, 1=Increase)
            // Bit 2-0 - Number of envelope sweep (If zero, stop envelope operation)
            0xFF21 => self.sound_4.envelope.read(),

            // NR43: Channel 4 polynomial counter
            // Bit 7-4 - Shift clock frequency
            // Bit 3   - Counter step/width (0=15 bits, 1=7 bits)
            // Bit 2-0 - Dividing ratio of frequencies
            0xFF22 => self.sound_4.polynomial_counter.read(),

            // NR44: Channel 4 counter/consecutive; initial
            // Bit 7 - Initial (1=restart sound) (write only)
            // Bit 6 - Counter/consecutive selection (1=Stop output when length in NR41 expires)
            0xFF23 => self.sound_4.initial_counter_consecutive.read(),

            // NR50: Channel control / ON-OFF / Volume
            // Specifies the master volume for Left/Right sound output.
            //
            // Bit 7    - Output Vin to SO2 terminal (1=Enable)
            // Bits 6-4 - SO2 output level (volume)  (0-7)
            // Bit 3    - Output Vin to SO1 terminal (1=Enable)
            // Bits 2-0 - SO1 output level (volume)  (0-7)
            0xFF24 => {
                let mut byte: u8 = self.so2_vol << 4;
                byte |= self.so1_vol;

                byte.set_bit(3, self.vin_so1);
                byte.set_bit(7, self.vin_so2);

                byte
            }

            // NR51: Selection of sound output terminal
            //
            // Bit 7 - Output sound 4 to SO2 terminal
            // Bit 6 - Output sound 3 to SO2 terminal
            // Bit 5 - Output sound 2 to SO2 terminal
            // Bit 4 - Output sound 1 to SO2 terminal
            // Bit 3 - Output sound 4 to SO1 terminal
            // Bit 2 - Output sound 3 to SO1 terminal
            // Bit 1 - Output sound 2 to SO1 terminal
            // Bit 0 - Output sound 1 to SO1 terminal
            0xFF25 => {
                let mut byte: u8 = 0;

                byte.set_bit(0, self.sound_1.so1_enabled);
                byte.set_bit(4, self.sound_1.so2_enabled);

                byte.set_bit(1, self.sound_2.so1_enabled);
                byte.set_bit(5, self.sound_2.so2_enabled);

                byte.set_bit(2, self.sound_3.so1_enabled);
                byte.set_bit(6, self.sound_3.so2_enabled);

                byte.set_bit(3, self.sound_4.so1_enabled);
                byte.set_bit(7, self.sound_4.so2_enabled);

                byte
            }

            // NR52: Sound on/off
            //
            // Bit 7 - All sound on/off
            // Bit 3 - Sound 4 on flag
            // Bit 2 - Sound 3 on flag
            // Bit 1 - Sound 2 on flag
            // Bit 0 - sound 1 on flag
            0xFF26 => {
                let mut byte: u8 = 0;

                byte.set_bit(0, self.sound_1.is_on);
                byte.set_bit(1, self.sound_2.is_on);
                byte.set_bit(2, self.sound_3.is_on);
                byte.set_bit(3, self.sound_4.is_on);

                byte.set_bit(7, self.sound_enabled);

                byte
            }

            // Channel 3 Wave pattern memory
            // Waveform storage for arbitrary sound data. Holds 32 4-bit samples, which are played
            // back upper 4 bits first.
            0xFF30...0xFF3F => {
                let index = address - 0xFF30;
                self.sound_3.wave_pattern[index as usize]
            }

            _ => panic!(
                "read out-of-range address in the sound controller: {:#0x}",
                address
            ),
        }
    }

    /// Writes a byte of audio memory.
    ///
    /// # Panics
    ///
    /// Panics if writing memory that is not managed by the sound controller.
    fn write_byte(&mut self, address: u16, byte: u8) {
        // Access to sound registers, aside from 0xFF26, is disabled unless sound is on.
        if !self.sound_enabled && address != 0xFF26 {
            return;
        }

        match address {
            // NR10: Sound 1 sweep register
            // Bit 6-4 - Sweep time
            // Bit 3   - Sweep Increase/Decrease
            //            0: Addition    (frequency increases)
            //            1: Subtraction (frequency decreases)
            // Bit 2-0 - Number of sweep shift (n: 0-7)
            0xFF10 => self.sound_1.sweep.write(byte),

            // NR11: Sound 1 Sound length/Wave pattern duty
            // Bit 7-6 - Wave pattern duty
            // Bit 5-0 - Sound length data (Write only)
            0xFF11 => self.sound_1.wave.write(byte),

            // NR12: Channel 1 volume envelope
            // Bit 7-4 - Initial volume of the envelope (0-15) (0 = no sound)
            // Bit 3   - Envelope direction (0 = decrease, 1 = increase)
            // Bit 2-0 - Number of envelope sweep (n: 0-7) (If 0, stop the envelope operation)
            0xFF12 => self.sound_1.envelope.write(byte),

            // NR13: Channel 1 Frequency low
            // Lower 8 bits of the 11-bit frequency
            0xFF13 => self.sound_1.frequency.write_lo(byte),

            // NR14: Channel 1 frequency high
            // Bit 7   - Initial (1 = restart sound) (write only)
            // Bit 6   - Counter/consecutive selection (1 = stop output when length in NR11
            //           expires)
            // Bit 2-0 - Frequency's higher 3 bits (write only)
            0xFF14 => self.sound_1.frequency.write_hi(byte),

            // NR21: Sound 2 Sound length/Wave pattern duty
            // Bit 7-6 - Wave pattern duty
            // Bit 5-0 - Sound length data (Write only)
            0xFF16 => self.sound_2.wave.write(byte),

            // NR22: Channel 2 volume envelope
            // Bit 7-4 - Initial volume of the envelope (0-15) (0 = no sound)
            // Bit 3   - Envelope direction (0 = decrease, 1 = increase)
            // Bit 2-0 - Number of envelope sweep (n: 0-7) (If 0, stop the envelope operation)
            0xFF17 => self.sound_2.envelope.write(byte),

            // NR23: Channel 2 frequency low
            // Lower 8 bits of the 11-bit frequency
            0xFF18 => self.sound_2.frequency.write_lo(byte),

            // NR24: Channel 2 frequency high
            // Bit 7   - Initial (1 = restart sound) (write only)
            // Bit 6   - Counter/consecutive selection (1 = stop output when length in NR21
            //           expires)
            // Bit 2-0 - Frequency's higher 3 bits (write only)
            0xFF19 => self.sound_2.frequency.write_hi(byte),

            // NR30: Channel 3 sound on/off
            // Bit 7 - Sound channel 3 off (0=Stop, 1=Playback)
            0xFF1A => self.sound_3.is_on = byte.has_bit_set(7),

            // NR31: Channel 3 sound length
            // Bit 7-0 - Sound length (0-255)
            0xFF1B => self.sound_3.length.write(byte),

            // NR32: Channel 3 select output level
            // Bit 6-5 - Select output level:
            //           0 - mute
            //           1 - 100% volume
            //           2 - 50% volume
            //           3 - 25% volume
            0xFF1C => self.sound_3.output_level.write(byte),

            // NR33: Channel 3 frequency low
            // Lower 8 bits of the 11-bit frequency
            0xFF1D => self.sound_3.frequency.write_lo(byte),

            // NR34: Channel 3 frequency high
            // Bit 7   - Initial (1 = restart sound) (write only)
            // Bit 6   - Counter/consecutive selection (1 = stop output when length in NR11
            //           expires)
            // Bit 2-0 - Frequency's higher 3 bits (write only)
            0xFF1E => self.sound_3.frequency.write_hi(byte),

            // NR41: Channel 4 sound length
            // Bit 5-0 - Sound length data (0-63)
            0xFF20 => self.sound_4.length.write(byte),

            // NR42: Channel 4 volume envelope
            // Bit 7-4 - Initial volume of envelope (0=No sound)
            // Bit 3   - Envelope direction (0=Decrease, 1=Increase)
            // Bit 2-0 - Number of envelope sweep (If zero, stop envelope operation)
            0xFF21 => self.sound_4.envelope.write(byte),

            // NR43: Channel 4 polynomial counter
            // Bit 7-4 - Shift clock frequency
            // Bit 3   - Counter step/width (0=15 bits, 1=7 bits)
            // Bit 2-0 - Dividing ratio of frequencies
            0xFF22 => self.sound_4.polynomial_counter.write(byte),

            // NR44: Channel 4 counter/consecutive; initial
            // Bit 7 - Initial (1=restart sound) (write only)
            // Bit 6 - Counter/consecutive selection (1=Stop output when length in NR41 expires)
            0xFF23 => self.sound_4.initial_counter_consecutive.write(byte),

            // NR50: Channel control / ON-OFF / Volume
            // Specifies the master volume for Left/Right sound output.
            //
            // Bit 7    - Output Vin to SO2 terminal (1=Enable)
            // Bits 6-4 - SO2 output level (volume)  (0-7)
            // Bit 3    - Output Vin to SO1 terminal (1=Enable)
            // Bits 2-0 - SO1 output level (volume)  (0-7)
            0xFF24 => {
                self.so1_vol = byte & 0x7;
                self.vin_so1 = byte.has_bit_set(3);

                self.so2_vol = (byte >> 4) & 0x7;
                self.vin_so2 = byte.has_bit_set(7);
            }

            // NR51: Selection of sound output terminal
            //
            // Bit 7 - Output sound 4 to SO2 terminal
            // Bit 6 - Output sound 3 to SO2 terminal
            // Bit 5 - Output sound 2 to SO2 terminal
            // Bit 4 - Output sound 1 to SO2 terminal
            // Bit 3 - Output sound 4 to SO1 terminal
            // Bit 2 - Output sound 3 to SO1 terminal
            // Bit 1 - Output sound 2 to SO1 terminal
            // Bit 0 - Output sound 1 to SO1 terminal
            0xFF25 => {
                self.sound_1.so1_enabled = byte.has_bit_set(0);
                self.sound_1.so2_enabled = byte.has_bit_set(4);

                self.sound_2.so1_enabled = byte.has_bit_set(1);
                self.sound_2.so2_enabled = byte.has_bit_set(5);

                self.sound_3.so1_enabled = byte.has_bit_set(2);
                self.sound_3.so2_enabled = byte.has_bit_set(6);

                self.sound_4.so1_enabled = byte.has_bit_set(3);
                self.sound_4.so2_enabled = byte.has_bit_set(7);
            }

            // NR52: Sound on/off
            // Writing to bit 7 of this address enables or disables all sound. The other bits of
            // this address are not writable.
            0xFF26 => {
                let enable_sound = byte.has_bit_set(7);
                self.sound_enabled = enable_sound;

                // TODO: Disabling sound allegedly destroys all the contents of the sound
                // registers.
            }

            // Channel 3 Wave pattern memory
            // Waveform storage for arbitrary sound data. Holds 32 4-bit samples, which are played
            // back upper 4 bits first.
            0xFF30...0xFF3F => {
                let index = address - 0xFF30;
                self.sound_3.wave_pattern[index as usize] = byte;
            }

            _ => panic!(
                "write out-of-range address in the sound controller: {:#0x}",
                address
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::u8;

    use bytes::ByteExt;

    use memory::Addressable;

    use super::{Envelope, Frequency, SoundController, Sweep, Wave};

    #[test]
    fn sweep_read() {
        let mut sweep = Sweep::default();

        for shift_num in 0..8 {
            for inc_dec in 0..2 {
                for time in 0..8 {
                    let expected = (time << 4) | (inc_dec << 3) | shift_num;

                    sweep.time = time;
                    sweep.decrease = if inc_dec == 1 { true } else { false };
                    sweep.shift = shift_num;

                    assert_eq!(sweep.read(), expected);
                }
            }
        }
    }

    #[test]
    fn sweep_write() {
        let mut sweep = Sweep::default();

        for shift_num in 0..8 {
            for inc_dec in 0..2 {
                for time in 0..8 {
                    for extra in 0..2 {
                        let byte = (extra << 7) | (time << 4) | (inc_dec << 3) | shift_num;

                        let expected_time = time;
                        let expected_decrease = if inc_dec == 1 { true } else { false };
                        let expected_shift = shift_num;

                        sweep.write(byte);

                        assert_eq!(sweep.time, expected_time);
                        assert_eq!(sweep.decrease, expected_decrease);
                        assert_eq!(sweep.shift, expected_shift);
                    }
                }
            }
        }
    }

    #[test]
    fn wave_read() {
        let mut wave = Wave::default();

        for pattern in 0..4 {
            for length in 0..64 {
                let expected = (pattern << 6) | 0x3F;

                wave.pattern = pattern;
                wave.length = length;

                assert_eq!(wave.read(), expected);
            }
        }
    }

    #[test]
    fn wave_write() {
        let mut wave = Wave::default();

        for pattern in 0..4 {
            for length in 0..64 {
                let byte = (pattern << 6) | length;

                let expected_pattern = pattern;
                let expected_length = length;

                wave.write(byte);

                assert_eq!(wave.pattern, expected_pattern);
                assert_eq!(wave.length, expected_length);
            }
        }
    }

    #[test]
    fn envelope_read() {
        let mut envelope = Envelope::default();

        for initial_vol in 0..16 {
            for direction_increase in 0..2 {
                for number in 0..8 {
                    let expected = (initial_vol << 4) | (direction_increase << 3) | number;

                    envelope.initial_vol = initial_vol;
                    envelope.direction_increase =
                        if direction_increase == 1 { true } else { false };
                    envelope.number = number;

                    assert_eq!(envelope.read(), expected);
                }
            }
        }
    }

    #[test]
    fn envelope_write() {
        let mut envelope = Envelope::default();

        for initial_vol in 0..16 {
            for direction_increase in 0..2 {
                for number in 0..8 {
                    let byte = (initial_vol << 4) | (direction_increase << 3) | number;

                    let expected_initial_vol = initial_vol;
                    let expected_direction_increase =
                        if direction_increase == 1 { true } else { false };
                    let expected_number = number;

                    envelope.write(byte);

                    assert_eq!(envelope.initial_vol, expected_initial_vol);
                    assert_eq!(envelope.direction_increase, expected_direction_increase);
                    assert_eq!(envelope.number, expected_number);
                }
            }
        }
    }

    #[test]
    fn frequency_write_low() {
        let mut frequency = Frequency::default();

        for freq in 0..4096 {
            let byte = (freq & 0xFF) as u8;

            frequency.write_lo(byte);

            assert_eq!((frequency.frequency & 0xFF) as u8, byte);

            frequency.frequency = 4095;
            frequency.write_lo(byte);

            assert_eq!((frequency.frequency & 0xFF) as u8, byte);
        }
    }

    #[test]
    fn frequency_read_high() {
        let mut frequency = Frequency::default();

        for initial in 0..2 {
            for counter in 0..2 {
                for freq in 0..4096 {
                    let mut expected = 0xFF;
                    expected.set_bit(6, counter == 1);

                    frequency.initial = if initial == 1 { true } else { false };
                    frequency.counter = if counter == 1 { true } else { false };
                    frequency.frequency = freq;

                    assert_eq!(frequency.read_hi(), expected);
                }
            }
        }
    }

    #[test]
    fn frequency_write_high() {
        let mut frequency = Frequency::default();

        for initial in 0..2 {
            for counter in 0..2 {
                for freq in 0..4096 {
                    let byte = (initial << 7) | (counter << 6) | (((freq >> 8) as u8) & 0x7);

                    let expected_initial = if initial == 1 { true } else { false };
                    let expected_counter = if counter == 1 { true } else { false };
                    let expected_frequency = freq & 0x0700;

                    frequency.write_hi(byte);

                    assert_eq!(frequency.initial, expected_initial);
                    assert_eq!(frequency.counter, expected_counter);
                    assert_eq!(frequency.frequency, expected_frequency);
                }
            }
        }
    }

    #[test]
    fn ff24_read() {
        let mut sc = SoundController::new();

        for so1_vol in 0..8 {
            for so2_vol in 0..8 {
                for vin_so1 in vec![false, true] {
                    for vin_so2 in vec![false, true] {
                        sc.so1_vol = so1_vol;
                        sc.so2_vol = so2_vol;
                        sc.vin_so1 = vin_so1;
                        sc.vin_so2 = vin_so2;

                        let mut expected = 0;
                        expected.set_bit(0, so1_vol.has_bit_set(0));
                        expected.set_bit(1, so1_vol.has_bit_set(1));
                        expected.set_bit(2, so1_vol.has_bit_set(2));
                        expected.set_bit(3, vin_so1);
                        expected.set_bit(4, so2_vol.has_bit_set(0));
                        expected.set_bit(5, so2_vol.has_bit_set(1));
                        expected.set_bit(6, so2_vol.has_bit_set(2));
                        expected.set_bit(7, vin_so2);

                        sc.sound_enabled = false;
                        assert_eq!(sc.read_byte(0xFF24), 0xFF);

                        sc.sound_enabled = true;
                        assert_eq!(sc.read_byte(0xFF24), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn ff24_write() {
        let mut sc = SoundController::new();

        for so1_vol in 0..8 {
            for so2_vol in 0..8 {
                for vin_so1 in vec![false, true] {
                    for vin_so2 in vec![false, true] {
                        sc.so1_vol = 0;
                        sc.so2_vol = 0;
                        sc.vin_so1 = false;
                        sc.vin_so2 = false;

                        let mut byte = so1_vol & (so2_vol << 4);
                        byte.set_bit(0, so1_vol.has_bit_set(0));
                        byte.set_bit(1, so1_vol.has_bit_set(1));
                        byte.set_bit(2, so1_vol.has_bit_set(2));
                        byte.set_bit(3, vin_so1);
                        byte.set_bit(4, so2_vol.has_bit_set(0));
                        byte.set_bit(5, so2_vol.has_bit_set(1));
                        byte.set_bit(6, so2_vol.has_bit_set(2));
                        byte.set_bit(7, vin_so2);

                        sc.sound_enabled = false;
                        sc.write_byte(0xFF24, byte);

                        assert_eq!(sc.so1_vol, 0);
                        assert_eq!(sc.so2_vol, 0);
                        assert_eq!(sc.vin_so1, false);
                        assert_eq!(sc.vin_so2, false);

                        sc.sound_enabled = true;
                        sc.write_byte(0xFF24, byte);

                        assert_eq!(sc.so1_vol, so1_vol);
                        assert_eq!(sc.so2_vol, so2_vol);
                        assert_eq!(sc.vin_so1, vin_so1);
                        assert_eq!(sc.vin_so2, vin_so2);
                    }
                }
            }
        }
    }

    #[test]
    fn ff25_read() {
        let mut sc = SoundController::new();

        for i_large in 0usize..256 {
            let i = i_large as u8;

            sc.sound_1.so1_enabled = i.has_bit_set(0);
            sc.sound_1.so2_enabled = i.has_bit_set(4);

            sc.sound_2.so1_enabled = i.has_bit_set(1);
            sc.sound_2.so2_enabled = i.has_bit_set(5);

            sc.sound_3.so1_enabled = i.has_bit_set(2);
            sc.sound_3.so2_enabled = i.has_bit_set(6);

            sc.sound_4.so1_enabled = i.has_bit_set(3);
            sc.sound_4.so2_enabled = i.has_bit_set(7);

            sc.sound_enabled = false;
            assert_eq!(sc.read_byte(0xFF25), 0xFF);

            sc.sound_enabled = true;
            assert_eq!(sc.read_byte(0xFF25), i);
        }
    }

    #[test]
    fn ff25_write() {
        let mut sc = SoundController::new();

        for i_large in 0usize..256 {
            let i = i_large as u8;

            // Make up a default state - writing with sound disabled shouldn't change this
            sc.sound_1.so1_enabled = false;
            sc.sound_1.so2_enabled = false;

            sc.sound_2.so1_enabled = true;
            sc.sound_2.so2_enabled = false;

            sc.sound_3.so1_enabled = false;
            sc.sound_3.so2_enabled = true;

            sc.sound_4.so1_enabled = true;
            sc.sound_4.so2_enabled = true;

            sc.sound_enabled = false;
            sc.write_byte(0xFF25, i);

            assert_eq!(sc.sound_1.so1_enabled, false);
            assert_eq!(sc.sound_1.so2_enabled, false);

            assert_eq!(sc.sound_2.so1_enabled, true);
            assert_eq!(sc.sound_2.so2_enabled, false);

            assert_eq!(sc.sound_3.so1_enabled, false);
            assert_eq!(sc.sound_3.so2_enabled, true);

            assert_eq!(sc.sound_4.so1_enabled, true);
            assert_eq!(sc.sound_4.so2_enabled, true);

            sc.sound_enabled = true;
            sc.write_byte(0xFF25, i);

            assert_eq!(sc.sound_1.so1_enabled, i.has_bit_set(0));
            assert_eq!(sc.sound_1.so2_enabled, i.has_bit_set(4));

            assert_eq!(sc.sound_2.so1_enabled, i.has_bit_set(1));
            assert_eq!(sc.sound_2.so2_enabled, i.has_bit_set(5));

            assert_eq!(sc.sound_3.so1_enabled, i.has_bit_set(2));
            assert_eq!(sc.sound_3.so2_enabled, i.has_bit_set(6));

            assert_eq!(sc.sound_4.so1_enabled, i.has_bit_set(3));
            assert_eq!(sc.sound_4.so2_enabled, i.has_bit_set(7));
        }
    }

    #[test]
    fn ff26_read() {
        let mut sc = SoundController::new();

        for i in 0u8..32 {
            let mut expected: u8 = i & 0x0F;
            expected.set_bit(7, i.has_bit_set(4));

            sc.sound_1.is_on = i.has_bit_set(0);
            sc.sound_2.is_on = i.has_bit_set(1);
            sc.sound_3.is_on = i.has_bit_set(2);
            sc.sound_4.is_on = i.has_bit_set(3);
            sc.sound_enabled = i.has_bit_set(4);

            assert_eq!(sc.read_byte(0xFF26), expected);
        }
    }

    #[test]
    fn ff26_write() {
        let mut sc = SoundController::new();

        for i_large in 0usize..256 {
            let i = i_large as u8;

            sc.write_byte(0xFF26, i);

            assert_eq!(sc.sound_enabled, i.has_bit_set(7));
        }
    }
}
