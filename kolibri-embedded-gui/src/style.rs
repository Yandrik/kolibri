use embedded_graphics::mono_font;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::{PixelColor, Rgb565};
use embedded_graphics::prelude::*;
use embedded_graphics::text::renderer::{CharacterStyle, TextRenderer};
use embedded_graphics::text::{Baseline, TextStyle, TextStyleBuilder};

pub struct Spacing {
    pub item_spacing: Size,
    pub button_padding: Size,
}

pub fn medsize_rgb565_style() -> Style<Rgb565, MonoTextStyle<'static, Rgb565>> {
    Style {
        background_color: Rgb565::BLACK,
        border_color: Rgb565::RED,
        primary_color: Rgb565::CYAN,
        secondary_color: Rgb565::YELLOW,
        border_width: 1,
        default_text_style: (
            MonoTextStyle::new(&mono_font::iso_8859_10::FONT_9X15, Rgb565::WHITE),
            TextStyleBuilder::new().baseline(Baseline::Bottom).build(),
        ),
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(2, 2),
        },
    }
}

pub struct Style<COL: PixelColor, DefaultCharstyle: TextRenderer<Color = COL>> {
    pub background_color: COL,
    pub border_color: COL,
    pub primary_color: COL,
    pub secondary_color: COL,
    pub border_width: u32,
    pub default_text_style: (DefaultCharstyle, TextStyle),
    pub spacing: Spacing,
}
