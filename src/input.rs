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
        match *self {
            Button::Up | Button::Down | Button::Right | Button::Left => true,
            _ => false,
        }
    }
}

#[derive(Debug, Default)]
pub struct ButtonState {
    pub select: SelectFlags,
    pressed: [bool; 8],
}

impl ButtonState {
    pub fn is_pressed(&self, button: Button) -> bool {
        if (button.is_direction() && self.select.contains(SelectFlags::DIRECTION)) ||
            (!button.is_direction() && self.select.contains(SelectFlags::BUTTON))
        {
            self.pressed[button as usize]
        } else {
            false
        }
    }

    pub fn press(&mut self, button: Button) {
        trace!("pressed {:?}", button);
        self.pressed[button as usize] = true;
    }

    pub fn release(&mut self, button: Button) {
        trace!("released {:?}", button);
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
