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
use embedded_graphics::pixelcolor::{PixelColor, Rgb565, Rgb888};
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

/// WidgetStyleElements specifies custom styling for a Widget state
#[derive(Debug, Clone, Copy)]
pub struct WidgetStyleElements<COL: PixelColor> {
    pub border_width: u32,
    pub border_color: COL,
    pub background_color: COL,
    pub foreground_color: COL,
}
/// WidgetStyle specifies custom styling for widgets like buttons that have different interaction states
#[derive(Debug, Clone, Copy)]
pub struct WidgetStyle<COL: PixelColor> {
    pub normal: WidgetStyleElements<COL>,
    pub hover: WidgetStyleElements<COL>,
    pub active: WidgetStyleElements<COL>,
    pub disabled: WidgetStyleElements<COL>,
}

/// Debug-friendly dark theme with visible borders for development.
///
/// This theme uses high-contrast colors and visible borders to make UI layout
/// and component boundaries clear during development.
pub fn medsize_rgb565_debug_style() -> Style<Rgb565> {
    Style {
        background_color: Rgb565::BLACK,
        default_widget_height: 16,
        default_font: mono_font::iso_8859_10::FONT_9X15,
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(2, 2),
            default_padding: Size::new(3, 3),
            window_border_padding: Size::new(3, 3),
        },
        widget: WidgetStyle {
            normal: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_RED,
                background_color: Rgb565::CSS_GRAY,
                foreground_color: Rgb565::WHITE,
            },
            hover: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::WHITE,
                background_color: Rgb565::new(0x1, 0x2, 0x1),
                foreground_color: Rgb565::WHITE,
            },
            active: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::BLACK,
                background_color: Rgb565::CYAN,
                foreground_color: Rgb565::BLACK,
            },
            disabled: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::CSS_DARK_GRAY,
                background_color: Rgb565::BLACK,
                foreground_color: Rgb565::CSS_DARK_GRAY,
            },
        },
    }
}

/// Dark theme for RGB565 displays.
///
/// Features a dark gray background with cyan accents and white text.
pub fn medsize_rgb565_style() -> Style<Rgb565> {
    Style {
        background_color: Rgb565::new(0x4, 0x8, 0x4), // pretty dark gray
        default_widget_height: 16,
        default_font: mono_font::iso_8859_10::FONT_9X15,
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(5, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        widget: WidgetStyle {
            normal: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::WHITE,
                background_color: Rgb565::new(0x2, 0x4, 0x2), // darker gray
                foreground_color: Rgb565::WHITE,
            },
            hover: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::WHITE,
                background_color: Rgb565::new(0x1, 0x2, 0x1),
                foreground_color: Rgb565::WHITE,
            },
            active: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::WHITE,
                background_color: Rgb565::new(0x2, 0x4, 0x2), // darker gray
                foreground_color: Rgb565::CSS_DARK_CYAN,
            },
            disabled: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::CSS_DARK_GRAY,
                background_color: Rgb565::new(0x4, 0x8, 0x4), // pretty dark gray
                foreground_color: Rgb565::CSS_DARK_GRAY,
            },
        },
    }
}

/// Light theme for RGB565 displays.
///
/// Features a white background with orange accents and black text.
pub fn medsize_light_rgb565_style() -> Style<Rgb565> {
    Style {
        background_color: Rgb565::CSS_WHITE,
        default_widget_height: 16,
        default_font: mono_font::iso_8859_10::FONT_9X15,
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(5, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        widget: WidgetStyle {
            normal: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::CSS_WHITE,
                background_color: Rgb565::CSS_NAVAJO_WHITE,
                foreground_color: Rgb565::CSS_BLACK,
            },
            hover: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_BLACK,
                background_color: Rgb565::CSS_GAINSBORO,
                foreground_color: Rgb565::CSS_BLACK,
            },
            active: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::CSS_WHITE,
                background_color: Rgb565::CSS_DARK_ORANGE,
                foreground_color: Rgb565::CSS_BLACK,
            },
            disabled: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::CSS_LIGHT_GRAY,
                background_color: Rgb565::CSS_LIGHT_GRAY,
                foreground_color: Rgb565::CSS_DARK_GRAY,
            },
        },
    }
}

/// Pink theme for RGB565 displays.
///
/// Features a peach background with pink accents and black text.
pub fn medsize_sakura_rgb565_style() -> Style<Rgb565> {
    Style {
        background_color: Rgb565::CSS_PEACH_PUFF,
        default_widget_height: 16,
        default_font: mono_font::ascii::FONT_9X15,
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(5, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        widget: WidgetStyle {
            normal: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::WHITE,
                background_color: Rgb565::CSS_LIGHT_PINK,
                foreground_color: Rgb565::WHITE,
            },
            hover: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_BLACK,
                background_color: Rgb565::CSS_HOT_PINK,
                foreground_color: Rgb565::WHITE,
            },
            active: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::WHITE,
                background_color: Rgb565::CSS_HOT_PINK,
                foreground_color: Rgb565::CSS_DARK_CYAN,
            },
            disabled: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::CSS_DARK_GRAY,
                background_color: Rgb565::CSS_PEACH_PUFF,
                foreground_color: Rgb565::CSS_DARK_GRAY,
            },
        },
    }
}

/// Blue theme for RGB565 displays.
///
/// Features a midnight blue background with violet accents and white text.
pub fn medsize_blue_rgb565_style() -> Style<Rgb565> {
    Style {
        background_color: Rgb565::CSS_MIDNIGHT_BLUE,
        default_widget_height: 16,
        default_font: mono_font::iso_8859_10::FONT_9X15,
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(5, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        widget: WidgetStyle {
            normal: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::CSS_WHITE,
                background_color: Rgb565::CSS_BLUE,
                foreground_color: Rgb565::CSS_WHITE,
            },
            hover: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_WHITE,
                background_color: Rgb565::CSS_LIGHT_BLUE,
                foreground_color: Rgb565::CSS_WHITE,
            },
            active: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::CSS_WHITE,
                background_color: Rgb565::CSS_PALE_VIOLET_RED,
                foreground_color: Rgb565::CSS_WHITE,
            },
            disabled: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::CSS_WHITE,
                background_color: Rgb565::CSS_DARK_GRAY,
                foreground_color: Rgb565::CSS_GRAY,
            },
        },
    }
}

/// Retro CRT monitor theme for RGB565 displays.
///
/// Features a black background with green text and borders, reminiscent of early CRT monitors.
pub fn medsize_crt_rgb565_style() -> Style<Rgb565> {
    Style {
        background_color: Rgb565::CSS_BLACK,
        default_widget_height: 16,
        default_font: mono_font::iso_8859_10::FONT_9X15,
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(5, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        widget: WidgetStyle {
            normal: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_GREEN,
                background_color: Rgb565::CSS_BLACK,
                foreground_color: Rgb565::CSS_GREEN,
            },
            hover: WidgetStyleElements {
                border_width: 3,
                border_color: Rgb565::CSS_GREEN,
                background_color: Rgb565::CSS_BLACK,
                foreground_color: Rgb565::CSS_GREEN,
            },
            active: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_GREEN,
                background_color: Rgb565::CSS_GREEN,
                foreground_color: Rgb565::CSS_BLACK,
            },
            disabled: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_GRAY,
                background_color: Rgb565::CSS_BLACK,
                foreground_color: Rgb565::CSS_GRAY,
            },
        },
    }
}

/// Minimalist black and white theme for RGB565 displays.
///
/// Features a white background with black borders and text, suitable for high contrast displays or e-ink screens.
pub fn medsize_retro_rgb565_style() -> Style<Rgb565> {
    Style {
        background_color: Rgb565::CSS_WHITE,
        default_widget_height: 16,
        default_font: mono_font::ascii::FONT_9X15,
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(5, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        widget: WidgetStyle {
            normal: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_BLACK,
                background_color: Rgb565::CSS_WHITE,
                foreground_color: Rgb565::CSS_BLACK,
            },
            hover: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_BLACK,
                background_color: Rgb565::CSS_WHITE,
                foreground_color: Rgb565::CSS_BLACK,
            },
            active: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_WHITE,
                background_color: Rgb565::CSS_BLACK,
                foreground_color: Rgb565::CSS_WHITE,
            },
            disabled: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_GRAY,
                background_color: Rgb565::CSS_LIGHT_GRAY,
                foreground_color: Rgb565::CSS_GRAY,
            },
        },
    }
}

/// Bootstrap-inspired theme for RGB565 displays.
///
/// Features a dark background with white text.
// custom colors defined as from(Rgb888) to allow direct comparison with standard web/rgb colors and color pickers and easy conversion to other PixelColors
pub fn medsize_bootstrap_rgb565_style() -> Style<Rgb565> {
    Style {
        background_color: Rgb565::CSS_BLACK,
        widget: WidgetStyle {
            normal: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::WHITE,
                background_color: Rgb565::CSS_BLACK,
                foreground_color: Rgb565::WHITE,
            },
            hover: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::WHITE,
                background_color: Rgb565::CSS_LIGHT_GRAY,
                foreground_color: Rgb565::CSS_BLACK,
            },
            active: WidgetStyleElements {
                border_width: 0,
                border_color: Rgb565::WHITE,
                background_color: Rgb565::WHITE,
                foreground_color: Rgb565::CSS_BLACK,
            },
            disabled: WidgetStyleElements {
                border_width: 1,
                border_color: Rgb565::CSS_DARK_GRAY,
                background_color: Rgb565::CSS_BLACK,
                foreground_color: Rgb565::CSS_DARK_GRAY,
            },
        },
        default_widget_height: 16,
        default_font: mono_font::ascii::FONT_9X15,
        spacing: Spacing {
            item_spacing: Size::new(8, 4),
            button_padding: Size::new(5, 5),
            default_padding: Size::new(1, 1),
            window_border_padding: Size::new(3, 3),
        },
        //corner_radius: 5,
    }
}

/// Bootstrap inspired custom Widget Style - for Primary Buttons and Widgets
pub fn medsize_bootstrap_rgb565_primary_widget_style() -> WidgetStyle<Rgb565> {
    WidgetStyle {
        normal: WidgetStyleElements {
            border_width: 0,
            border_color: Rgb565::from(Rgb888::new(13, 110, 253)), // rgb(13,110,253)
            background_color: Rgb565::from(Rgb888::new(13, 110, 253)), // rgb(13,110,253)
            foreground_color: Rgb565::WHITE,
        },
        hover: WidgetStyleElements {
            border_width: 0,
            border_color: Rgb565::from(Rgb888::new(0x0b, 0x5e, 0xd7)), // #0B5ED7
            background_color: Rgb565::from(Rgb888::new(0x0b, 0x5e, 0xd7)), // rgba(11, 94, 215, 1)
            foreground_color: Rgb565::WHITE,
        },
        active: WidgetStyleElements {
            border_width: 0,
            border_color: Rgb565::from(Rgb888::new(10, 88, 202)), // rgb(10,88,202)
            background_color: Rgb565::from(Rgb888::new(10, 88, 202)), // rgb(10,88,202)
            foreground_color: Rgb565::WHITE,
        },
        disabled: WidgetStyleElements {
            border_width: 0,
            border_color: Rgb565::from(Rgb888::new(0x13, 0x54, 0xb3)), // rgba(19, 84, 179, 1)
            background_color: Rgb565::from(Rgb888::new(0x13, 0x54, 0xb3)), // rgba(19, 84, 179, 1)
            foreground_color: Rgb565::CSS_LIGHT_GRAY,
        },
    }
}

/// Bootstrap inspired custom Widget Style - for Primary Buttons and Widgets
pub fn medsize_bootstrap_rgb565_secondary_widget_style() -> WidgetStyle<Rgb565> {
    WidgetStyle {
        normal: WidgetStyleElements {
            border_width: 0,
            border_color: Rgb565::from(Rgb888::new(108, 117, 125)), // rgb(108,117,125)
            background_color: Rgb565::from(Rgb888::new(108, 117, 125)), // rgb(108,117,125)
            foreground_color: Rgb565::WHITE,
        },
        hover: WidgetStyleElements {
            border_width: 0,
            border_color: Rgb565::from(Rgb888::new(92, 99, 106)), //  rgb(92, 99, 106)
            background_color: Rgb565::from(Rgb888::new(92, 99, 106)), //  rgb(92, 99, 106)
            foreground_color: Rgb565::WHITE,
        },
        active: WidgetStyleElements {
            border_width: 0,
            border_color: Rgb565::from(Rgb888::new(0x0a, 0x58, 0xca)), // rgb(76, 81, 91)
            background_color: Rgb565::from(Rgb888::new(0x0a, 0x58, 0xca)), // rgb(76, 81, 91)
            foreground_color: Rgb565::WHITE,
        },
        disabled: WidgetStyleElements {
            border_width: 0,
            border_color: Rgb565::from(Rgb888::new(81, 89, 95)), // rgb(81, 89, 95)
            background_color: Rgb565::from(Rgb888::new(81, 89, 95)), // rgb(81, 89, 95)
            foreground_color: Rgb565::from(Rgb888::new(177, 179, 180)), // rgb(177, 179, 180)
        },
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
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Style<COL: PixelColor> {
    /// Background color for the entire UI
    pub background_color: COL,
    /// Default height for widgets like buttons
    pub default_widget_height: u32,
    /// Default font used for text rendering
    pub default_font: MonoFont<'static>,
    /// Spacing configuration for UI elements
    pub spacing: Spacing,
    /// default styling for widgets
    pub widget: WidgetStyle<COL>,
}
