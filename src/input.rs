use bitflags::bitflags;

use crate::bytes::ByteExt;

bitflags! {
    #[derive(Default)]
    pub struct SelectFlags: u8 {
        const BUTTON    = 0x10;
        const DIRECTION = 0x20;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Button {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    B = 4,
    A = 5,
    Start = 6,
    Select = 7,
}

impl Button {
    fn is_direction(&self) -> bool {
        matches!(
            self,
            Button::Up | Button::Down | Button::Right | Button::Left
        )
    }
}

#[derive(Debug, Default)]
pub struct ButtonState {
    select: SelectFlags,
    pressed: [bool; 8],
}

impl ButtonState {
    pub fn is_pressed(&self, button: Button) -> bool {
        if (button.is_direction() && self.select.contains(SelectFlags::DIRECTION))
            || (!button.is_direction() && self.select.contains(SelectFlags::BUTTON))
        {
            self.pressed[button as usize]
        } else {
            false
        }
    }

    /// Select which buttons should be read.
    ///
    /// Set by I/O register 0xFF00.
    pub fn select(&mut self, register: u8) {
        self.select = SelectFlags::from_bits_truncate(register);
    }

    /// Conversion to I/O register 0xFF00.
    pub fn as_byte(&self) -> u8 {
        // The two high bits are unused, so they should be set high.
        let mut register = 0xc0;

        register.set_bit(
            0,
            !(self.is_pressed(Button::Right) || self.is_pressed(Button::A)),
        );

        register.set_bit(
            1,
            !(self.is_pressed(Button::Left) || self.is_pressed(Button::B)),
        );

        register.set_bit(
            2,
            !(self.is_pressed(Button::Up) || self.is_pressed(Button::Select)),
        );

        register.set_bit(
            3,
            !(self.is_pressed(Button::Down) || self.is_pressed(Button::Start)),
        );

        // Sets bits 4 and 5.
        register |= self.select.bits();

        register
    }

    pub fn press(&mut self, button: Button) {
        self.pressed[button as usize] = true;
    }

    pub fn release(&mut self, button: Button) {
        self.pressed[button as usize] = false;
    }
}

#[cfg(test)]
mod tests {
    use super::{Button, ButtonState, SelectFlags};

    #[test]
    fn press() {
        let mut state = ButtonState::default();
        state.press(Button::A);
        assert!(!state.is_pressed(Button::A));

        state.select = SelectFlags::BUTTON;
        assert!(state.is_pressed(Button::A));

        state.press(Button::Up);
        assert!(!state.is_pressed(Button::Up));

        state.select = SelectFlags::DIRECTION;
        assert!(state.is_pressed(Button::Up));

        assert!(!state.is_pressed(Button::Down));

        state.select = SelectFlags::BUTTON | SelectFlags::DIRECTION;
        state.press(Button::B);
        state.press(Button::Left);
        assert!(state.is_pressed(Button::B));
        assert!(state.is_pressed(Button::Left));
    }
}
