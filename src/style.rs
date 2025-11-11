//! GUI styling and theming system
//!
//! The style module provides a flexible theming system for Kolibri GUIs. It controls the visual
//! appearance of the UI, including colors, spacing, fonts, and other visual aspects of the
//! interface. This allows compatibility and abstraction over [embedded_graphics::pixelcolor]
//! color types, making it easy to switch between color depths or display technologies.
//! Several predefined themes are included for [Rgb565] displays (e.g. ILI9341).
//!
//! # Examples
//!
//! Using a predefined theme:
//! ```no_run
//! # use embedded_graphics::pixelcolor::Rgb565;
//! # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
//! # use embedded_graphics::prelude::*;
//! # use embedded_graphics::primitives::Rectangle;
//! # use embedded_iconoir::prelude::*;
//! # use embedded_iconoir::size12px;
//! # use kolibri_embedded_gui::ui::*;
//! # use kolibri_embedded_gui::label::*;
//! # use kolibri_embedded_gui::smartstate::*;
//! # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
//! # let output_settings = OutputSettingsBuilder::new().build();
//! # let mut window = Window::new("Kolibri Example", &output_settings);
//! use kolibri_embedded_gui::style::medsize_rgb565_style;
//! use kolibri_embedded_gui::ui::Ui;
//!
//! // Create UI with dark theme
//! let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
//! ```
//!
//! Switching themes at runtime:
//! ```no_run
//! # use embedded_graphics::pixelcolor::Rgb565;
//! # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
//! # use embedded_graphics::prelude::*;
//! # use embedded_graphics::primitives::Rectangle;
//! # use embedded_iconoir::prelude::*;
//! # use embedded_iconoir::size12px;
//! # use kolibri_embedded_gui::ui::*;
//! # use kolibri_embedded_gui::label::*;
//! # use kolibri_embedded_gui::smartstate::*;
//! # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
//! # let output_settings = OutputSettingsBuilder::new().build();
//! # let mut window = Window::new("Kolibri Example", &output_settings);
//! use kolibri_embedded_gui::style::{medsize_rgb565_style, medsize_light_rgb565_style};
//!
//! let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
//! // Later...
//! *ui.style_mut() = medsize_light_rgb565_style(); // Switch to light theme
//! ```

use embedded_graphics::mono_font::{self, MonoFont};
use embedded_graphics::pixelcolor::{PixelColor, Rgb565};
use embedded_graphics::prelude::*;

/// Controls spacing between UI elements.
#[derive(Debug, Clone, Copy)]
pub struct Spacing {
    /// Space between adjacent items in the UI
    pub item_spacing: Size,
    /// Internal padding within buttons
    pub button_padding: Size,
    /// Padding around the border of a widget (e.g. a checkbox)
    pub default_padding: Size,
    /// Padding inside window borders
    pub window_border_padding: Size,
}

/// Debug-friendly dark theme with visible borders for development.
///
/// This theme uses high-contrast colors and visible borders to make UI layout
/// and component boundaries clear during development.
pub fn medsize_rgb565_debug_style() -> Style<Rgb565> {
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
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(2, 2),
            default_padding: Size::new(3, 3),
            window_border_padding: Size::new(3, 3),
        },
        corner_radius: 8,
    }
}

/// Dark theme for RGB565 displays.
///
/// Features a dark gray background with cyan accents and white text.
pub fn medsize_rgb565_style() -> Style<Rgb565> {
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
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(6, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        corner_radius: 8,
    }
}

/// Light theme for RGB565 displays.
///
/// Features a white background with orange accents and black text.
pub fn medsize_light_rgb565_style() -> Style<Rgb565> {
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
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(6, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        corner_radius: 8,
    }
}

/// Pink theme for RGB565 displays.
///
/// Features a peach background with pink accents and black text.
pub fn medsize_sakura_rgb565_style() -> Style<Rgb565> {
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
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(6, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        corner_radius: 8,
    }
}

/// Blue theme for RGB565 displays.
///
/// Features a midnight blue background with violet accents and white text.
pub fn medsize_blue_rgb565_style() -> Style<Rgb565> {
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
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(6, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        corner_radius: 8,
    }
}

/// Retro CRT monitor theme for RGB565 displays.
///
/// Features a black background with green text and borders, reminiscent of early CRT monitors.
pub fn medsize_crt_rgb565_style() -> Style<Rgb565> {
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
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(5, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        corner_radius: 0,
    }
}

/// Minimalist black and white theme for RGB565 displays.
///
/// Features a white background with black borders and text, suitable for high contrast displays or e-ink screens.
pub fn medsize_retro_rgb565_style() -> Style<Rgb565> {
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
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(5, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        corner_radius: 0,
    }
}

/// Defines the visual appearance of a Kolibri UI.
///
/// The [Style] struct controls all visual aspects of the UI, including colors,
/// spacing, fonts, and dimensions. It is generic over the color type to support
/// different color depths (e.g., RGB565, grayscale, or monochrome).
///
/// # Examples
///
/// Creating a custom style:
/// ```rust
/// use embedded_graphics::pixelcolor::Rgb565;
/// use embedded_graphics::mono_font;
/// use kolibri_embedded_gui::style::{Style, Spacing};
/// use embedded_graphics::prelude::*;
///
/// let custom_style = Style {
///     background_color: Rgb565::BLACK,
///     text_color: Rgb565::WHITE,
///     primary_color: Rgb565::BLUE,
///     spacing: Spacing {
///         item_spacing: Size::new(10, 5),
///         button_padding: Size::new(4, 4),
///         default_padding: Size::new(2, 2),
///         window_border_padding: Size::new(3, 3),
///     },
///     default_font: mono_font::ascii::FONT_6X13,
///     border_color: Rgb565::BLACK,
///     border_width: 1,
///     default_widget_height: 16,
///     icon_color: Rgb565::BLACK,
///     secondary_color: Rgb565::YELLOW,
///     highlight_border_color: Rgb565::WHITE,
///     highlight_border_width: 2,
///     highlight_item_background_color: Rgb565::BLUE,
///     item_background_color: Rgb565::BLACK,
///     corner_radius: 8,
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Style<COL: PixelColor> {
    /// Background color for the entire UI
    pub background_color: COL,
    /// Color used for borders around widgets
    pub border_color: COL,
    /// Primary accent color for interactive elements
    pub primary_color: COL,
    /// Secondary accent color for additional highlighting;
    pub secondary_color: COL,
    /// Color used for icons
    pub icon_color: COL,
    /// Default height for widgets like buttons
    pub default_widget_height: u32,
    /// Width of borders around widgets
    pub border_width: u32,
    /// Default font used for text rendering
    pub default_font: MonoFont<'static>,
    /// Spacing configuration for UI elements
    pub spacing: Spacing,
    /// Background color for items like buttons
    pub item_background_color: COL,
    /// Background color for highlighted items
    pub highlight_item_background_color: COL,
    /// Border color for highlighted elements
    pub highlight_border_color: COL,
    /// Border width for highlighted elements
    pub highlight_border_width: u32,
    /// Color used for text
    pub text_color: COL,
    /// Corner radius for rounded corners on widgets
    pub corner_radius: u32,
}
