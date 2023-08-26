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
        default_font: mono_font::iso_8859_10::FONT_9X15,
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
        default_font: mono_font::iso_8859_10::FONT_9X15,
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

pub fn medsize_light_rgb565_style() -> Style<Rgb565, MonoTextStyle<'static, Rgb565>> {
    Style {
        background_color: Rgb565::CSS_WHITE,
        item_background_color: Rgb565::CSS_NAVAJO_WHITE,
        highlight_item_background_color: Rgb565::CSS_GAINSBORO,
        border_color: Rgb565::CSS_WHITE,
        highlight_border_color: Rgb565::CSS_BLACK,
        primary_color: Rgb565::CSS_DARK_ORANGE,
        secondary_color: Rgb565::YELLOW,
        icon_color: Rgb565::CSS_BLACK,
        text_color: Rgb565::CSS_BLACK,
        default_widget_height: 16,
        border_width: 0,
        highlight_border_width: 1,
        default_font: mono_font::iso_8859_10::FONT_9X15,
        default_text_style: (
            MonoTextStyle::new(&mono_font::iso_8859_10::FONT_9X15, Rgb565::BLACK),
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

pub fn medsize_sakura_rgb565_style() -> Style<Rgb565, MonoTextStyle<'static, Rgb565>> {
    Style {
        background_color: Rgb565::CSS_PEACH_PUFF,
        item_background_color: Rgb565::CSS_LIGHT_PINK,
        highlight_item_background_color: Rgb565::CSS_HOT_PINK,
        border_color: Rgb565::CSS_WHITE,
        highlight_border_color: Rgb565::CSS_BLACK,
        primary_color: Rgb565::CSS_DEEP_PINK,
        secondary_color: Rgb565::YELLOW,
        icon_color: Rgb565::CSS_BLACK,
        text_color: Rgb565::CSS_BLACK,
        default_widget_height: 16,
        border_width: 0,
        highlight_border_width: 1,
        default_font: mono_font::ascii::FONT_9X15,
        default_text_style: (
            MonoTextStyle::new(&mono_font::ascii::FONT_9X15, Rgb565::BLACK),
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

pub fn medsize_blue_rgb565_style() -> Style<Rgb565, MonoTextStyle<'static, Rgb565>> {
    Style {
        background_color: Rgb565::CSS_MIDNIGHT_BLUE,
        item_background_color: Rgb565::CSS_BLUE,
        highlight_item_background_color: Rgb565::CSS_BLUE_VIOLET,
        border_color: Rgb565::CSS_WHITE,
        highlight_border_color: Rgb565::CSS_WHITE,
        primary_color: Rgb565::CSS_PALE_VIOLET_RED,
        secondary_color: Rgb565::YELLOW,
        icon_color: Rgb565::CSS_WHITE,
        text_color: Rgb565::CSS_WHITE,
        default_widget_height: 16,
        border_width: 0,
        highlight_border_width: 1,
        default_font: mono_font::iso_8859_10::FONT_9X15,
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

pub fn medsize_crt_rgb565_style() -> Style<Rgb565, MonoTextStyle<'static, Rgb565>> {
    Style {
        background_color: Rgb565::CSS_BLACK,
        item_background_color: Rgb565::CSS_BLACK,
        highlight_item_background_color: Rgb565::CSS_BLACK,
        border_color: Rgb565::CSS_GREEN,
        highlight_border_color: Rgb565::CSS_GREEN,
        primary_color: Rgb565::CSS_GREEN,
        secondary_color: Rgb565::YELLOW,
        icon_color: Rgb565::CSS_GREEN,
        text_color: Rgb565::CSS_GREEN,
        default_widget_height: 16,
        border_width: 1,
        highlight_border_width: 3,
        default_font: mono_font::iso_8859_10::FONT_9X15,
        default_text_style: (
            MonoTextStyle::new(&mono_font::iso_8859_10::FONT_9X15, Rgb565::GREEN),
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

pub fn medsize_retro_rgb565_style() -> Style<Rgb565, MonoTextStyle<'static, Rgb565>> {
    Style {
        background_color: Rgb565::CSS_WHITE,
        item_background_color: Rgb565::CSS_WHITE,
        highlight_item_background_color: Rgb565::CSS_WHITE,
        border_color: Rgb565::CSS_BLACK,
        highlight_border_color: Rgb565::CSS_BLACK,
        primary_color: Rgb565::CSS_BLACK,
        secondary_color: Rgb565::YELLOW,
        icon_color: Rgb565::CSS_BLACK,
        text_color: Rgb565::CSS_BLACK,
        default_widget_height: 16,
        border_width: 1,
        highlight_border_width: 1,
        default_font: mono_font::ascii::FONT_9X15,
        default_text_style: (
            MonoTextStyle::new(&mono_font::ascii::FONT_9X15, Rgb565::BLACK),
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
    pub default_font: MonoFont<'static>,
    #[deprecated(note = "use default_font and text_color instead")]
    pub default_text_style: (DefaultCharstyle, TextStyle),
    pub spacing: Spacing,
    pub item_background_color: COL,
    pub highlight_item_background_color: COL,
    pub highlight_border_color: COL,
    pub highlight_border_width: u32,
    pub text_color: COL,
}
