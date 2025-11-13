//! Label widgets for displaying text in the UI.
//!
//! Labels are basic building blocks for displaying text content. They support both static
//! and dynamic text with features like:
//!
//! - Custom fonts and styling
//! - Automatic vertical centering
//! - Integration with the smartstate system for efficient redraws
//! - HashLabel variant for auto-refreshing on content changes
//!
//! # Examples
//!
//! ```no_run
//! # use embedded_graphics::pixelcolor::Rgb565;
//! # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
//! # use kolibri_embedded_gui::style::medsize_rgb565_style;
//! # use kolibri_embedded_gui::ui::Ui;
//! # use embedded_graphics::prelude::*;
//! # use embedded_graphics::primitives::Rectangle;
//! # use embedded_iconoir::prelude::*;
//! # use embedded_iconoir::size12px;
//! # use kolibri_embedded_gui::ui::*;
//! # use embedded_graphics::mono_font::ascii;
//! # use kolibri_embedded_gui::label::*;
//! # use kolibri_embedded_gui::smartstate::*;
//! # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
//! # let output_settings = OutputSettingsBuilder::new().build();
//! # let mut window = Window::new("Kolibri Example", &output_settings);
//! # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
//! # use kolibri_embedded_gui::label::*;
//!
//! // Create a basic label
//! ui.add(Label::new("Hello World"));
//!
//! let hasher = Hasher::new();
//! let mut smartstate = SmartstateProvider::<20>::new();
//!
//! // Create a HashLabel
//! ui.add(HashLabel::new("Dynamic content", smartstate.nxt(), &hasher));
//! ```

use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiError, GuiResult, Response, Ui, Widget};
use core::hash::BuildHasher;
use core::hash::Hash;
use core::ops::Add;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::renderer::TextRenderer as _;
use embedded_graphics::text::{Alignment, Baseline, Text};
use foldhash::fast::RandomState;

/// A widget for displaying text in the UI.
///
/// Labels are the primary way to display text content. They support static text display
/// with optional font customization and smartstate integration for efficient redraws.
///
/// # Features
///
/// - Basic text display with customizable fonts
/// - Smartstate integration for incremental redrawing
/// - Automatic vertical centering in allocated space
///
/// # Examples
///
/// ```no_run
/// # use embedded_graphics::pixelcolor::Rgb565;
/// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
/// # use kolibri_embedded_gui::style::medsize_rgb565_style;
/// # use kolibri_embedded_gui::ui::Ui;
/// # use embedded_graphics::prelude::*;
/// # use embedded_graphics::primitives::Rectangle;
/// # use embedded_iconoir::prelude::*;
/// # use embedded_iconoir::size12px;
/// # use kolibri_embedded_gui::ui::*;
/// # use embedded_graphics::mono_font::ascii;
/// # use kolibri_embedded_gui::label::*;
/// # use kolibri_embedded_gui::smartstate::*;
/// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
/// # let output_settings = OutputSettingsBuilder::new().build();
/// # let mut window = Window::new("Kolibri Example", &output_settings);
/// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
/// # let mut smartstateProvider = SmartstateProvider::<20>::new();
/// # use kolibri_embedded_gui::label::*;
///
/// // Basic label
/// ui.add(Label::new("Basic text"));
///
/// // Label with custom font and smartstate
/// ui.add(Label::new("Custom font").with_font(ascii::FONT_10X20).smartstate(smartstateProvider.nxt()));
/// ```
pub struct Label<'a> {
    text: &'a str,
    font: Option<MonoFont<'a>>,
    smartstate: Container<'a, Smartstate>,
    /// Width to auto-truncate to (if too large, screen width is used)
    auto_truncate_width: Option<u32>,
}

impl<'a> Label<'a> {
    /// Creates a new label with the given text.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use embedded_iconoir::size12px;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use embedded_graphics::mono_font::ascii;
    /// # use kolibri_embedded_gui::label::*;
    /// # use kolibri_embedded_gui::smartstate::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// ui.add(Label::new("Hello World"));
    /// ```
    pub fn new(text: &'a str) -> Label<'a> {
        Label {
            text,
            font: None,
            smartstate: Container::empty(),
            auto_truncate_width: None,
        }
    }

    /// Sets a custom font for the label.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use embedded_iconoir::size12px;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use embedded_graphics::mono_font::ascii;
    /// # use kolibri_embedded_gui::label::*;
    /// # use kolibri_embedded_gui::smartstate::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # use kolibri_embedded_gui::label::*;
    /// # let hasher = Hasher::new();
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// ui.add(Label::new("Custom Font").with_font(ascii::FONT_10X20));
    /// ```
    pub fn with_font(mut self, font: MonoFont<'a>) -> Self {
        self.font = Some(font);
        self
    }

    /// Adds a [Smartstate] to the label for incremental redrawing.
    ///
    /// When using smartstate, the label will only redraw when the smartstate is
    /// **forced to redraw** using [Smartstate::force_redraw()].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use embedded_iconoir::size12px;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use embedded_graphics::mono_font::ascii;
    /// # use kolibri_embedded_gui::label::*;
    /// # use kolibri_embedded_gui::smartstate::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # use kolibri_embedded_gui::label::*;
    /// # let hasher = Hasher::new();
    /// # let mut smartstateProvider = SmartstateProvider::<20>::new();
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// ui.add(Label::new("Efficient").smartstate(smartstateProvider.nxt()));
    /// ```
    ///
    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }

    pub fn auto_truncate(self) -> Self {
        self.auto_truncate_to(u32::MAX)
    }

    /// if the text is too long to fit within the available width calling this will reduce the length of the text and append an ellipsis (...)
    pub fn auto_truncate_to(mut self, width: u32) -> Self {
        self.auto_truncate_width = Some(width);
        self
    }
}

impl Widget for Label<'_> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // get size of text
        let font = self.font.unwrap_or(ui.style().default_font);
        let text_style = MonoTextStyle::new(&font, ui.style().text_color);
        let mut is_ellipsis = false;
        let ellipsis_width = measure_string_width("...", text_style);
        let mut text = self.text;

        if let Some(width) = self.auto_truncate_width {
            let (truncated_text, ellipsis_needed) =
                truncate_text_singleline(text, text_style, width.min(ui.space_available().width));
            text = truncated_text;
            is_ellipsis = ellipsis_needed;
        }

        let mut text = Text::new(
            text,
            Point::new(0, 0),
            MonoTextStyle::new(&font, ui.style().text_color),
        );

        let mut size = text.bounding_box();

        if is_ellipsis {
            size.size.width += ellipsis_width;
        }
        // allocate space
        let iresponse = ui.allocate_space(Size::new(size.size.width, size.size.height))?;

        // check smartstate (a bool would work, but this is consistent with other widgets)
        let redraw = !self.smartstate.eq_option(&Some(Smartstate::state(0)));
        self.smartstate.modify(|st| *st = Smartstate::state(0));

        // move text (center vertically)
        text.translate_mut(iresponse.area.top_left.add(Point::new(
            0,
            (iresponse.area.size.height - size.size.height) as i32 / 2,
        )));
        text.text_style.baseline = Baseline::Top;

        // draw

        if redraw {
            ui.start_drawing(&iresponse.area);
            // clear background if necessary
            if !ui.cleared() {
                ui.clear_area(iresponse.area)?;
            }

            ui.draw(&text)
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw text")))?;

            if is_ellipsis {
                let mut ellipsis = Text::new(
                    "...",
                    text.bounding_box().bottom_right().unwrap(),
                    text_style,
                );
                ellipsis.text_style.baseline = Baseline::Bottom;
                ellipsis.text_style.alignment = Alignment::Left;

                ui.draw(&ellipsis)
                    .map_err(|_| GuiError::DrawError(Some("Couldn't draw ellipsis")))?;
            }
            ui.finalize()?;
        }

        Ok(Response::new(iresponse))
    }
}

/// A hasher for widgets that require hashing of data.
///
/// Make sure to create your hasher outside of the drawing loop, just like you would with a
/// [crate::smartstate::SmartstateProvider].
pub struct Hasher {
    random_state: RandomState,
}

impl Hasher {
    pub fn new() -> Self {
        Self {
            random_state: RandomState::default(),
        }
    }
    pub fn hash<T: Hash + ?Sized>(&self, to_hash: &T) -> u64 {
        self.random_state.hash_one(to_hash)
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

/// A [Label] variant that automatically refreshes when its content changes.
///
/// HashLabel maintains a hash of its text content and automatically redraws
/// when the content changes. This is particularly useful for displaying
/// dynamic content that updates frequently.
///
/// # Features
///
/// - Automatic content change detection via hashing
/// - Custom font support
/// - Efficient redrawing only when content changes
///
/// # Examples
///
/// ```no_run
/// # use embedded_graphics::pixelcolor::Rgb565;
/// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
/// # use kolibri_embedded_gui::style::medsize_rgb565_style;
/// # use kolibri_embedded_gui::ui::Ui;
/// # use embedded_graphics::prelude::*;
/// # use embedded_graphics::primitives::Rectangle;
/// # use embedded_iconoir::prelude::*;
/// # use embedded_iconoir::size12px;
/// # use kolibri_embedded_gui::ui::*;
/// # use kolibri_embedded_gui::label::*;
/// # use kolibri_embedded_gui::smartstate::*;
/// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
/// # let output_settings = OutputSettingsBuilder::new().build();
/// # let mut window = Window::new("Kolibri Example", &output_settings);
/// # use kolibri_embedded_gui::label::*;
/// # let hasher = Hasher::new();
/// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
/// # let mut smartstateProvider = SmartstateProvider::<20>::new();
/// let mut count = 42;
///
/// // Create a hasher (do this outside the draw loop)
/// let hasher = Hasher::new();
///
/// // Create a HashLabel that updates when content changes
/// ui.add(HashLabel::new(
///     format!("Count: {}", count).as_ref(),
///     smartstateProvider.nxt(),
///     &hasher
/// ));
/// ```
pub struct HashLabel<'a> {
    text: &'a str,
    font: Option<MonoFont<'a>>,
    smartstate: Container<'a, Smartstate>,
    hasher: &'a Hasher,
    /// Width to auto-truncate to (if too large, screen width is used)
    auto_truncate_width: Option<u32>,
}

impl<'a> HashLabel<'a> {
    /// Creates a new HashLabel with the given text, smartstate, and hasher.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use embedded_iconoir::size12px;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # use kolibri_embedded_gui::smartstate::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # use kolibri_embedded_gui::label::*;
    /// # let hasher = Hasher::new();
    /// # let mut smartstateProvider = SmartstateProvider::<20>::new();
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// let hasher = Hasher::new();
    /// let mut value = 42;
    /// let text = format!("Dynamic: {}", value);
    ///
    /// ui.add(HashLabel::new(
    ///     text.as_ref(),
    ///     smartstateProvider.nxt(),
    ///     &hasher
    /// ));
    /// ```
    pub fn new(text: &'a str, smartstate: &'a mut Smartstate, hasher: &'a Hasher) -> Self {
        Self {
            text,
            font: None,
            smartstate: Container::new(smartstate),
            hasher,
            auto_truncate_width: None,
        }
    }

    /// Sets a custom font for the HashLabel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_graphics::mono_font::ascii;
    /// # use embedded_iconoir::prelude::*;
    /// # use embedded_iconoir::size12px;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # use kolibri_embedded_gui::smartstate::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # use kolibri_embedded_gui::label::*;
    /// # let hasher = Hasher::new();
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut smartstateProvider = SmartstateProvider::<20>::new();
    /// let mut text = "Some dynamically changed text";
    ///
    /// ui.add(HashLabel::new(text, smartstateProvider.nxt(), &hasher).with_font(ascii::FONT_10X20));
    /// ```
    pub fn with_font(mut self, font: MonoFont<'a>) -> Self {
        self.font = Some(font);
        self
    }

    pub fn auto_truncate(self) -> Self {
        self.auto_truncate_to(u32::MAX)
    }

    /// if the text is too long to fit within the available width calling this will reduce the length of the text and append an ellipsis (...)
    pub fn auto_truncate_to(mut self, width: u32) -> Self {
        self.auto_truncate_width = Some(width);
        self
    }
}

impl Widget for HashLabel<'_> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // get size of text
        let font = self.font.unwrap_or(ui.style().default_font);
        let text_style = MonoTextStyle::new(&font, ui.style().text_color);
        let mut is_ellipsis = false;
        let ellipsis_width = measure_string_width("...", text_style);
        let mut text = self.text;

        if let Some(width) = self.auto_truncate_width {
            let (truncated_text, ellipsis_needed) =
                truncate_text_singleline(text, text_style, width.min(ui.space_available().width));
            text = truncated_text;
            is_ellipsis = ellipsis_needed;
        }

        let mut text = Text::new(
            text,
            Point::new(0, 0),
            MonoTextStyle::new(&font, ui.style().text_color),
        );

        let mut size = text.bounding_box();

        if is_ellipsis {
            size.size.width += ellipsis_width;
        }
        // allocate space
        let iresponse = ui.allocate_space(Size::new(size.size.width, size.size.height))?;

        let hash = self.hasher.hash(self.text) as u32;

        let redraw = !self.smartstate.eq_option(&Some(Smartstate::state(hash)));
        self.smartstate.modify(|st| *st = Smartstate::state(hash));

        // move text (center vertically)
        text.translate_mut(iresponse.area.top_left.add(Point::new(
            0,
            (iresponse.area.size.height - size.size.height) as i32 / 2,
        )));
        text.text_style.baseline = Baseline::Top;

        // draw

        if redraw {
            ui.start_drawing(&iresponse.area);
            // clear background if necessary
            if !ui.cleared() {
                ui.clear_area(iresponse.area)?;
            }

            ui.draw(&text)
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw text")))?;

            if is_ellipsis {
                let mut ellipsis = Text::new(
                    "...",
                    text.bounding_box().bottom_right().unwrap(),
                    text_style,
                );
                ellipsis.text_style.baseline = Baseline::Bottom;
                ellipsis.text_style.alignment = Alignment::Left;

                ui.draw(&ellipsis)
                    .map_err(|_| GuiError::DrawError(Some("Couldn't draw ellipsis")))?;
            }
            ui.finalize()?;
        }

        Ok(Response::new(iresponse))
    }
}

/// Measures the width of a string using the specified font style.
///
/// # Arguments
/// * `text` - The text to measure.
/// * `font` - The font style that will be used for drawing the text.
fn measure_string_width<Color: PixelColor>(text: &str, font: MonoTextStyle<'_, Color>) -> u32 {
    font.measure_string(text, Point::zero(), Baseline::Top)
        .bounding_box
        .size
        .width
}

/// Truncates a string by removing the specified number of characters from the end.
fn truncate_string_by(text: &str, num_chars: usize) -> Option<&str> {
    if let Some((idx, _char)) = text.char_indices().nth_back(num_chars) {
        Some(&text[..idx])
    } else {
        None
    }
}

/// Truncates text to fit within max_width once drawn, returning the truncated text and a flag indicating if an ellipsis is needed.
///
/// # Arguments
///
/// * `text` - The text to truncate.
/// * `font` - The font style used for measuring text width.
/// * `max_width` - The maximum allowed width for the text.
///
/// # Returns
///
/// A tuple containing the truncated text and a boolean indicating if an ellipsis should be drawn.
/// This is true if:
/// - Truncation has occurred.
/// - The original text was longer than the truncated text.
/// - The truncated text (at least 1 char of it) plus ellipsis fits within max_width. (otherwise the text is just cut off.)
fn truncate_text_singleline<'a, Color: PixelColor>(
    text: &'a str,
    font: MonoTextStyle<'_, Color>,
    max_width: u32,
) -> (&'a str, bool) {
    // Placeholder for text truncation logic
    let mut text = text.lines().next().unwrap_or("");
    let mut ellipsis_needed = false;
    let mut width = measure_string_width(text, font);
    if width > max_width {
        let mut ellipsis_width = measure_string_width("...", font) + font.font.character_spacing;
        ellipsis_needed = true;
        if ellipsis_width + font.font.character_size.width > max_width {
            // not enough space even for ellipsis
            ellipsis_needed = false;
            ellipsis_width = 0;
        }

        let per_char_width_estimate = font.font.character_size.width + font.font.character_spacing;
        let chars_to_remove = (width - max_width - ellipsis_width) / per_char_width_estimate;
        text = truncate_string_by(text, chars_to_remove as usize).unwrap_or("");
        width = measure_string_width(text, font);
        while width + ellipsis_width > max_width {
            // keep trimming until it fits
            text = truncate_string_by(text, 1).unwrap_or("");
            width = measure_string_width(text, font);
        }
    }
    (text, ellipsis_needed)
}
