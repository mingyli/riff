use ansi_term::{Colour, Style};

#[derive(Default)]
pub struct ColorConfig {
    pub removed: Style,
    pub added: Style,
}

impl ColorConfig {
    pub fn plain() -> ColorConfig {
        ColorConfig::default()
    }

    pub fn colored() -> ColorConfig {
        ColorConfig {
            removed: Colour::Black.on(Colour::Red),
            added: Colour::Black.on(Colour::Green),
        }
    }
}
