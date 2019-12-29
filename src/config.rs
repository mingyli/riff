use std::io;

use crate::color::ColorConfig;

pub struct Config<'a> {
    pub color_config: ColorConfig,
    pub left_file: &'a str,
    pub right_file: &'a str,
}

pub struct ConfigBuilder<'a> {
    color_config: Option<ColorConfig>,
    left_file: &'a str,
    right_file: &'a str,
}

impl<'a> ConfigBuilder<'a> {
    pub fn new() -> ConfigBuilder<'a> {
        ConfigBuilder {
            color_config: None,
            left_file: "",
            right_file: "",
        }
    }

    pub fn with_left_file(mut self, left_file: &'a str) -> ConfigBuilder<'a> {
        self.left_file = left_file;
        self
    }

    pub fn with_right_file(mut self, right_file: &'a str) -> ConfigBuilder<'a> {
        self.right_file = right_file;
        self
    }

    pub fn with_plain_colors(mut self) -> ConfigBuilder<'a> {
        self.color_config = Some(ColorConfig::plain());
        self
    }

    pub fn with_colors(mut self) -> ConfigBuilder<'a> {
        self.color_config = Some(ColorConfig::colored());
        self
    }

    pub fn build(self) -> io::Result<Config<'a>> {
        Ok(Config {
            color_config: self
                .color_config
                .ok_or_else(|| io::ErrorKind::InvalidInput)?,
            left_file: self.left_file,
            right_file: self.right_file,
        })
    }
}
