use image::Rgba;

/// The colors that can be displayed by the DMG.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Shade {
    White = 0,
    LightGray = 1,
    DarkGray = 2,
    Black = 3,
}

impl Shade {
    /// Returns a pixel that represents the color of a `Shade`.
    pub fn to_rgba(&self) -> Rgba<u8> {
        use self::Shade::*;

        // This uses the GameBoy Pocket palette.
        // TODO: Support more palettes.
        match *self {
            White => Rgba([0xFF, 0xFF, 0xFF, 0xFF]),
            LightGray => Rgba([0xA9, 0xA9, 0xA9, 0xFF]),
            DarkGray => Rgba([0x54, 0x54, 0x54, 0xFF]),
            Black => Rgba([0x00, 0x00, 0x00, 0xFF]),
        }
    }
}

impl Default for Shade {
    fn default() -> Shade {
        Shade::White
    }
}

impl From<u8> for Shade {
    fn from(val: u8) -> Shade {
        use self::Shade::*;

        match val {
            0 => White,
            1 => LightGray,
            2 => DarkGray,
            3 => Black,
            _ => panic!("only 0-3 correspond to valid shades"),
        }
    }
}

/// Maps background and window tile colors to shades.
///
/// This struct can be thought of as a map from color number to shade, where the color numbers
/// are those used by the Background and Window tiles.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BackgroundPalette([Shade; 4]);

impl BackgroundPalette {
    pub fn new(shades: [Shade; 4]) -> Self {
        BackgroundPalette(shades)
    }

    pub fn get(&self, index: u8) -> Shade {
        self.0[index as usize]
    }

    pub fn as_byte(&self) -> u8 {
        shades_to_register(&self.0)
    }
}

impl From<u8> for BackgroundPalette {
    fn from(byte: u8) -> Self {
        BackgroundPalette(shades_from_register(byte))
    }
}

impl Into<u8> for BackgroundPalette {
    fn into(self) -> u8 {
        shades_to_register(&self.0)
    }
}

/// Maps sprite colors to shades.
///
/// This struct can be thought of as a map from color number to shade, where the color numbers
/// are those used by the sprite tiles. Note that color 0 is always transparent for sprites.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SpritePalette([Shade; 4]);

impl SpritePalette {
    pub fn new(shades: [Shade; 4]) -> Self {
        SpritePalette(shades)
    }

    pub fn get(&self, index: u8) -> Option<Shade> {
        match index {
            0 => None,
            _ => Some(self.0[index as usize]),
        }
    }

    pub fn as_byte(&self) -> u8 {
        shades_to_register(&self.0)
    }
}

impl From<u8> for SpritePalette {
    fn from(byte: u8) -> Self {
        SpritePalette(shades_from_register(byte))
    }
}

impl Into<u8> for SpritePalette {
    fn into(self) -> u8 {
        shades_to_register(&self.0)
    }
}

fn shades_from_register(reg: u8) -> [Shade; 4] {
    let mut shades = [Shade::default(); 4];

    for (i, shade) in shades.iter_mut().enumerate() {
        let number = (reg >> (i * 2)) & 0b11;
        *shade = number.into();
    }

    shades
}

fn shades_to_register(shades: &[Shade]) -> u8 {
    let mut register = 0;

    for (i, shade) in shades.iter().enumerate() {
        register |= (*shade as u8) << (i * 2);
    }

    register
}
