use embedded_graphics::mono_font;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::{PixelColor, Rgb565};
use embedded_graphics::prelude::*;
use embedded_graphics::text::renderer::{CharacterStyle, TextRenderer};
use embedded_graphics::text::{Baseline, TextStyle, TextStyleBuilder};

#[derive(Debug, Clone, Copy)]
pub struct Spacing {
    pub item_spacing: Size,
    pub button_padding: Size,
    /// Padding around the border of a widget (e.g. a checkbox)
    pub default_padding: Size,
    pub window_border_padding: Size,
}

pub fn medsize_rgb565_debug_style() -> Style<Rgb565, MonoTextStyle<'static, Rgb565>> {
    Style {
        background_color: Rgb565::BLACK,
        item_background_color: Rgb565::CSS_GRAY,
        highlight_item_background_color: Rgb565::new(0x1, 0x2, 0x1),
        border_color: Rgb565::RED,
        highlight_border_color: Rgb565::WHITE,
        primary_color: Rgb565::CYAN,
        secondary_color: Rgb565::YELLOW,
        icon_color: Rgb565::WHITE,
        text_color: Rgb565::WHITE,
        default_widget_height: 16,
        border_width: 1,
        highlight_border_width: 1,
        default_text_style: (
            MonoTextStyle::new(&mono_font::iso_8859_10::FONT_9X15, Rgb565::WHITE),
            TextStyleBuilder::new().baseline(Baseline::Bottom).build(),
        ),
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(2, 2),
            default_padding: Size::new(3, 3),
            window_border_padding: Size::new(3, 3),
        },
    }
}

pub fn medsize_rgb565_style() -> Style<Rgb565, MonoTextStyle<'static, Rgb565>> {
    Style {
        background_color: Rgb565::new(0x4, 0x8, 0x4), // pretty dark gray
        item_background_color: Rgb565::new(0x2, 0x4, 0x2), // darker gray
        highlight_item_background_color: Rgb565::new(0x1, 0x2, 0x1),
        border_color: Rgb565::WHITE,
        highlight_border_color: Rgb565::WHITE,
        primary_color: Rgb565::CSS_DARK_CYAN,
        secondary_color: Rgb565::YELLOW,
        icon_color: Rgb565::WHITE,
        text_color: Rgb565::WHITE,
        default_widget_height: 16,
        border_width: 0,
        highlight_border_width: 1,
        default_text_style: (
            MonoTextStyle::new(&mono_font::iso_8859_10::FONT_9X15, Rgb565::WHITE),
            TextStyleBuilder::new().baseline(Baseline::Bottom).build(),
        ),
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(5, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style<COL: PixelColor, DefaultCharstyle: TextRenderer<Color = COL>> {
    pub background_color: COL,
    pub border_color: COL,
    pub primary_color: COL,
    pub secondary_color: COL,
    pub icon_color: COL,
    pub default_widget_height: u32,
    pub border_width: u32,
    pub default_text_style: (DefaultCharstyle, TextStyle),
    pub spacing: Spacing,
    pub item_background_color: COL,
    pub highlight_item_background_color: COL,
    pub highlight_border_color: COL,
    pub highlight_border_width: u32,
    /// The color of text. **CAUTION!** This is not necessarily supported. Most text renderers
    /// don't actually give you control over the color of the text.
    // FIXME: correct me if i'm wrong here
    pub text_color: COL,
}
