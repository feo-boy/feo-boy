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
            return 0;
        }

        match address {
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
