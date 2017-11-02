//! Audio-related functionality.
//!
//! Contains an implmentation of the GameBoy sound hardware.

use bytes::ByteExt;

use memory::Addressable;

/// A single GameBoy sound channel.
#[derive(Debug, Default)]
pub struct Sound {
    /// Whether or not the sound is enabled.
    pub is_on: bool,

    /// Whether to output this sound to SO1 terminal.
    pub so1_enabled: bool,

    /// Whether to output this sound to SO2 terminal.
    pub so2_enabled: bool,
}

/// The controller for the four sound channels output by the GameBoy.
#[derive(Debug, Default)]
pub struct SoundController {
    /// Sound 1: Rectangle waveform with sweep and envelope functions.
    pub sound_1: Sound,

    /// Sound 2: Rectangle waveform with envelope function.
    pub sound_2: Sound,

    /// Sound 3: A waveform specificed by the waveform RAM.
    pub sound_3: Sound,

    /// Sound 4: White noise with an envelope function.
    pub sound_4: Sound,

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
    /// Reads a byte of audio memory.
    ///
    /// # Panics
    ///
    /// Panics if reading memory that is not managed by the sound controller.
    fn read_byte(&self, address: u16) -> u8 {
        // Access to sound registers, aside from 0xFF26, is disabled unless the sound is on.
        if !self.sound_enabled && address != 0xFF26 {
            // TODO: Currently assumes that unreadable addresses will be read as 0, but that might
            // not be the case.
            return 0;
        }

        match address {
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

            _ => panic!("read out-of-range address in the sound controller: {:#0x}", address),
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

            _ => panic!("write out-of-range address in the sound controller: {:#0x}", address),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::u8;

    use bytes::ByteExt;

    use memory::Addressable;

    use super::SoundController;

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
            assert_eq!(sc.read_byte(0xFF25), 0x0);

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
