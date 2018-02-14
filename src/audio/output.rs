//! Audio playback functionality.
//!
//! Plays the audio based on the state of the sound hardware.

use audio::SoundController;

/// Outputs the GameBoy audio for a given sound controller.
#[derive(Debug, Default)]
pub struct AudioOutput {
    /// The sound controller which determines the audio to output.
    sound_controller: SoundController,
}
