//! # Slider Widget
//!
//! A widget for selecting numeric values within an inclusive range.
//!
//! # Core Features
//!
//! - Interactive value selection within inclusive ranges using fixed-point arithmetic
//! - Step-based value adjustments for precise control
//! - Optional text labels for clear identification
//! - Customizable width to fit various layouts
//! - Visual feedback for different interaction states (normal, hover, active)
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
//! # let mut smartstateProvider = SmartstateProvider::<20>::new();
//! # use kolibri_embedded_gui::slider::*;
//! // Basic slider with range
//! let mut value = 0i16;
//! ui.add(Slider::new(&mut value, -100..=100));
//!
//! // Slider with label and custom width
//! ui.add(Slider::new(&mut value, 0..=100)
//!     .label("Volume")
//!     .width(150));
//!
//! // Slider with step size and smartstate
//! ui.add(Slider::new(&mut value, 0..=100)
//!     .step_size(5)
//!     .smartstate(smartstateProvider.nxt()));
//! ```
//!
//! # Visual Components
//!
//! The slider consists of several visual elements:
//! - A horizontal track line representing the full value range
//! - A movable knob indicating the current value position
//! - Optional label text displayed above the slider
//! - Visual states (normal, hover, active) with appropriate color changes
//!
//! # Interaction Details
//!
//! - Click and drag functionality for intuitive value selection
//! - Step-based value snapping for precise control
//! - Range constraints to keep values within bounds
//! - Visual feedback through dynamic styling
//! - Efficient fixed-point arithmetic for smooth value interpolation
//!
use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiResult, Interaction, Response, Ui, Widget};
use core::cmp::max;
use core::ops::RangeInclusive;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyleBuilder};
use embedded_graphics::text::{Alignment, Baseline, Text};

/// Performs linear interpolation using fixed-point arithmetic for embedded systems.
///
/// This function interpolates between `start` and `end` values based on a given position `t`
/// within the range `[min_t, max_t]`. It uses only integer arithmetic to maintain performance
/// and predictability on embedded systems.
///
/// Returns the interpolated value clamped within the valid range.
fn lerp_fixed(start: i16, end: i16, t: i16, min_t: i16, max_t: i16) -> i16 {
    // Convert to i32 to prevent overflow during calculations
    let (start, end, t, min_t, max_t) = (
        start as i32,
        end as i32,
        t as i32,
        min_t as i32,
        max_t as i32,
    );

    // Clamp `t` between `min_t` and `max_t`
    let clamped_t = if t < min_t {
        min_t
    } else if t > max_t {
        max_t
    } else {
        t
    };

    // Calculate the range
    let range = max_t - min_t;
    if range == 0 {
        return start as i16;
    }

    // Perform linear interpolation using only integer arithmetic
    let interpolated = start + ((end - start) * (clamped_t - min_t) + (range / 2)) / range;

    interpolated as i16
}

/// An interactive slider widget for selecting numeric values.
///
/// The Slider widget provides a graphical way to select values within an inclusive range.
/// It supports features like step-based value adjustments, optional labels, and visual
/// feedback for user interactions.
///
/// The widget uses fixed-point arithmetic for smooth value interpolation while maintaining
/// performance on embedded systems.
pub struct Slider<'a> {
    value: &'a mut i16,
    range: RangeInclusive<i16>,
    step_size: u16,
    label: Option<&'a str>,
    width: u32,
    smartstate: Container<'a, Smartstate>,
}

impl<'a> Slider<'a> {
    /// Creates a new slider with the given value and range.
    ///
    /// # Arguments
    ///
    /// * `value` - A mutable reference to the value to be controlled
    /// * `range` - An inclusive range specifying the minimum and maximum allowed values
    ///
    /// By default, the slider uses a step size of 1, no label, and a width of 200 pixels.
    /// These can be customized using the builder methods.
    pub fn new(value: &'a mut i16, range: RangeInclusive<i16>) -> Self {
        Self {
            value,
            range,
            step_size: 1,
            smartstate: Container::empty(),
            label: None,
            width: 200,
        }
    }

    /// Sets the label text for the slider.
    ///
    /// The label is displayed above the slider track and helps identify its purpose.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Adds a smartstate to the slider for incremental redrawing.
    ///
    /// The smartstate tracks the slider's value and interaction state to minimize
    /// unnecessary redraws.
    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }

    /// Sets the width of the slider in pixels.
    ///
    /// This affects only the track and interaction area width. The total width
    /// may be larger if the label text exceeds this value.
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Sets the step size for value adjustments.
    ///
    /// The step size determines how the value snaps during interaction:
    /// - Values will snap to the nearest multiple of the step size
    /// - Must be at least 1 and at most the range span
    ///
    /// # Arguments
    ///
    /// * `step_size` - The size of each discrete step
    pub fn step_size(mut self, step_size: u16) -> Self {
        let range_span = (*self.range.end() - *self.range.start()).abs();
        self.step_size = step_size.clamp(1, range_span as u16);
        self
    }
}

impl Widget for Slider<'_> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // get size

        let padding = ui.style().spacing.button_padding;

        let slider_thickness = 2;
        let slider_knob_diameter = 10;

        let mut height = max(
            max(ui.style().default_widget_height, ui.get_row_height()),
            slider_knob_diameter + padding.height * 2,
        );

        let mut width = self.width + 2 * padding.width;

        let font = ui.style().default_font;
        let mut text = if let Some(label) = self.label {
            let mut text = Text::new(
                label,
                Point::new(0, 0),
                MonoTextStyle::new(&font, ui.style().text_color),
            );
            text.text_style.alignment = Alignment::Center;
            text.text_style.baseline = Baseline::Top;
            height += padding.height + text.bounding_box().size.height;
            width = width.max(text.bounding_box().size.width + 2 * padding.width);
            Some(text)
        } else {
            None
        };

        let size = Size::new(width, height);

        /*
        let icon = match size.width - 2 * padding.width {
            0..=17 => 12,
            18..=24 => 18,
            24..=32 => 24,
            _ => 32,
        };
         */

        // allocate space
        let iresponse = ui.allocate_space(Size::new(size.width, max(size.height, height)))?;

        // slider main line
        let slider_line = Line::new(
            Point::new(
                iresponse.area.top_left.x + padding.width as i32 + slider_knob_diameter as i32 / 2,
                iresponse.area.top_left.y + (padding.height + slider_knob_diameter / 2) as i32,
            ),
            Point::new(
                iresponse.area.top_left.x
                    + (size.width - padding.width - slider_knob_diameter / 2) as i32,
                iresponse.area.top_left.y + (padding.height + slider_knob_diameter / 2) as i32,
            ),
        );

        let style = ui.style();
        let line_style = PrimitiveStyleBuilder::new()
            .stroke_color(style.border_color)
            .stroke_width(slider_thickness)
            .fill_color(style.primary_color)
            .build();
        let mut slider_knob_style = PrimitiveStyleBuilder::new()
            .stroke_color(style.border_color)
            .stroke_width(1.max(style.border_width))
            .fill_color(style.background_color)
            .build();
        let old_slider_knob_style = PrimitiveStyleBuilder::new()
            .stroke_color(style.background_color)
            .stroke_width(0)
            .fill_color(style.background_color)
            .build();

        // previous slider knob circle for clearing it

        // center text (if it exists)
        if let Some(text) = text.as_mut() {
            let center_offset = iresponse.area.top_left
                + Point::new(
                    (iresponse.area.size.width / 2) as i32,
                    (iresponse.area.size.height - text.bounding_box().size.height - padding.height)
                        as i32,
                );
            text.translate_mut(center_offset);
        }

        // check for click
        // let click = matches!(iresponse.interaction, Interaction::Release(_));
        // let down = matches!(
        //     iresponse.interaction,
        //     Interaction::Click(_) | Interaction::Drag(_)
        // );

        // find user input
        // TODO
        let old_val = *self.value;
        match iresponse.interaction {
            Interaction::Click(point) | Interaction::Drag(point) => {
                let slider_val = lerp_fixed(
                    *self.range.start(),
                    *self.range.end(),
                    point.x as i16 - iresponse.area.top_left.x as i16,
                    // + (slider_knob_diameter / 2) as i16,
                    padding.width as i16 + slider_knob_diameter as i16 / 2,
                    width as i16 - padding.width as i16 - slider_knob_diameter as i16 / 2,
                );
                let range_span = (*self.range.end() - *self.range.start()).abs();
                let step_size = self.step_size.clamp(1, range_span as u16) as i16;
                let to_next = slider_val.rem_euclid(step_size);
                let to_prev = step_size - to_next;
                if to_next < to_prev {
                    *self.value = (slider_val - to_next).max(*self.range.start());
                } else {
                    *self.value = (slider_val + to_prev).min(*self.range.end());
                }
            }
            _ => {}
        }

        let slider_knob_pos = lerp_fixed(
            // padding.width as i16,
            padding.width as i16 + slider_knob_diameter as i16 / 2,
            width as i16 - padding.width as i16 - slider_knob_diameter as i16 / 2,
            *self.value,
            *self.range.start(),
            *self.range.end(),
        );

        let slider_knob = Circle::with_center(
            Point::new(
                iresponse.area.top_left.x + slider_knob_pos as i32,
                iresponse.area.top_left.y
                    + padding.height as i32
                    + (slider_knob_diameter / 2) as i32,
            ),
            slider_knob_diameter,
        );

        // old slider knob (for clearing)
        let old_slider_knob_pos = lerp_fixed(
            // padding.width as i16,
            padding.width as i16 + slider_knob_diameter as i16 / 2,
            width as i16 - padding.width as i16 - slider_knob_diameter as i16 / 2,
            old_val,
            *self.range.start(),
            *self.range.end(),
        );

        let old_slider_knob = Circle::with_center(
            Point::new(
                iresponse.area.top_left.x + old_slider_knob_pos as i32,
                iresponse.area.top_left.y
                    + padding.height as i32
                    + (slider_knob_diameter / 2) as i32,
            ),
            slider_knob_diameter + 4,
        );

        // styles and smartstate

        let interact_val: u16 = match iresponse.interaction {
            Interaction::Click(_) | Interaction::Drag(_) => {
                slider_knob_style.fill_color = Some(style.primary_color);
                2
            }
            Interaction::Hover(_) => {
                slider_knob_style.fill_color = Some(style.highlight_item_background_color);
                1
            }
            _ => {
                slider_knob_style.fill_color = Some(style.item_background_color);
                0
            }
        };
        let state_val = (*self.value as u16) as u32 | ((interact_val as u32) << 16);

        if !self.smartstate.eq_inner(&Smartstate::state(state_val)) {
            ui.start_drawing(&iresponse.area);

            if old_slider_knob_pos != slider_knob_pos {
                ui.draw(&old_slider_knob.into_styled(old_slider_knob_style))
                    .ok();
            }
            ui.draw(&slider_line.into_styled(line_style)).ok();
            ui.draw(&slider_knob.into_styled(slider_knob_style)).ok();
            // ui.draw(&icon_img).ok();
            if let Some(text) = text.as_mut() {
                ui.draw(text).unwrap();
            }

            ui.finalize()?;
        }

        self.smartstate
            .modify(|s| *s = Smartstate::state(state_val));

        Ok(Response::new(iresponse).set_changed(old_val != *self.value)) //.set_clicked(click).set_down(down))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_fixed_basic() {
        let start = 0;
        let end = 100;
        let t = 50;
        let min_t = 0;
        let max_t = 100;
        assert_eq!(lerp_fixed(start, end, t, min_t, max_t), 50);
    }

    #[test]
    fn test_lerp_fixed_clamp_low() {
        let start = 0;
        let end = 100;
        let t = -10;
        let min_t = 0;
        let max_t = 100;
        assert_eq!(lerp_fixed(start, end, t, min_t, max_t), 0);
    }

    #[test]
    fn test_lerp_fixed_clamp_high() {
        let start = 0;
        let end = 100;
        let t = 110;
        let min_t = 0;
        let max_t = 100;
        assert_eq!(lerp_fixed(start, end, t, min_t, max_t), 100);
    }

    #[test]
    fn test_lerp_fixed_negative_range() {
        let start = -50;
        let end = 50;
        let t = 0;
        let min_t = -50;
        let max_t = 50;
        assert_eq!(lerp_fixed(start, end, t, min_t, max_t), 0);
    }

    #[test]
    fn test_lerp_fixed_negative_t() {
        let start = 0;
        let end = 200;
        let t = -25;
        let min_t = -50;
        let max_t = 50;
        assert_eq!(lerp_fixed(start, end, t, min_t, max_t), 50);
    }

    #[test]
    fn test_lerp_fixed_max_t_equals_min_t() {
        let start = 100;
        let end = 200;
        let t = 100;
        let min_t = 100;
        let max_t = 100;
        assert_eq!(lerp_fixed(start, end, t, min_t, max_t), 100);
    }

    #[test]
    fn test_lerp_fixed_halfway_negative_range() {
        let start = -100;
        let end = 100;
        let t = 0;
        let min_t = -100;
        let max_t = 100;
        assert_eq!(lerp_fixed(start, end, t, min_t, max_t), 0);
    }

    #[test]
    fn test_lerp_fixed_rounding() {
        let start = 0;
        let end = 100;
        let t = 33;
        let min_t = 0;
        let max_t = 100;
        // Expected: 0 + ((100 - 0) * 33 + 50) / 100 = (3300 + 50)/100 = 3350/100 = 33
        assert_eq!(lerp_fixed(start, end, t, min_t, max_t), 33);

        let t = 34;
        // Expected: (3400 + 50)/100 = 3450/100 = 34
        assert_eq!(lerp_fixed(start, end, t, min_t, max_t), 34);
    }

    #[test]
    fn test_lerp_fixed_full_range() {
        let start = -32768;
        let end = 32767;
        let t = 0;
        let min_t = -32768;
        let max_t = 32767;
        assert_eq!(lerp_fixed(start, end, t, min_t, max_t), 0);
    }
}
