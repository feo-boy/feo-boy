bitflags! {
    #[derive(Default)]
    pub struct SelectFlags: u8 {
        const DIRECTION = 0b00010000;
        const BUTTON = 0b00100000;
    }
}

#[derive(Debug)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    B,
    A,
    Start,
    Select,
}

#[derive(Debug, Default)]
pub struct ButtonState {
    pub select: SelectFlags,
    pressed: [bool; 8],
}

impl ButtonState {
    fn button_to_index(button: &Button) -> usize {
        use Button::*;

        match *button {
            Up => 0,
            Down => 1,
            Left => 2,
            Right => 3,
            B => 4,
            A => 5,
            Start => 6,
            Select => 7,
        }
    }

    pub fn is_pressed(&self, button: &Button) -> bool {
        let index = Self::button_to_index(button);

        let direction_pressed = self.select.contains(SelectFlags::DIRECTION) && index <= 3 &&
            self.pressed[index];
        let button_pressed = self.select.contains(SelectFlags::BUTTON) && index >= 4 &&
            self.pressed[index];

        direction_pressed || button_pressed
    }

    pub fn press(&mut self, button: &Button) {
        self.pressed[Self::button_to_index(button)] = true;
    }

    pub fn release(&mut self, button: &Button) {
        self.pressed[Self::button_to_index(button)] = false;
    }
}

#[cfg(test)]
mod tests {
    use super::{Button, ButtonState, SelectFlags};

    #[test]
    fn press() {
        let mut state = ButtonState::default();
        state.press(&Button::A);
        assert!(!state.is_pressed(&Button::A));

        state.select = SelectFlags::BUTTON;
        assert!(state.is_pressed(&Button::A));

        state.press(&Button::Up);
        assert!(!state.is_pressed(&Button::Up));

        state.select = SelectFlags::DIRECTION;
        assert!(state.is_pressed(&Button::Up));

        assert!(!state.is_pressed(&Button::Down));

        state.select = SelectFlags::BUTTON | SelectFlags::DIRECTION;
        state.press(&Button::B);
        state.press(&Button::Left);
        assert!(state.is_pressed(&Button::B));
        assert!(state.is_pressed(&Button::Left));
    }
}
