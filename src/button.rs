//! # Button Widget
//!
//! See [Button] for more info.

use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiResult, Interaction, Response, Ui, Widget};
use core::cmp::max;
use core::ops::Add;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::{Baseline, Text};

/// # Button Widget
///
/// A clickable button widget that displays text and responds to user interaction.
///
/// Buttons are one of the most fundamental widgets in Kolibri. They provide a simple way to trigger
/// actions in response to user input. Buttons can be created with just a text label and optionally
/// support smartstate-based incremental redrawing for better performance.
///
/// # Features
/// - Text label with customizable font and colors
/// - Visual feedback for different interaction states (normal, hover, pressed)
/// - Optional smartstate support for incremental redrawing
/// - Automatic sizing based on text content and style settings
///
/// # Example
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
/// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
/// # use kolibri_embedded_gui::button::Button;
///
/// // Create a basic button
/// if ui.add(Button::new("Click me!")).clicked() {
///     // Handle click
/// }
///
/// // Create a button with smartstate for incremental redrawing
/// let mut smartstateProvider = SmartstateProvider::<20>::new();
/// if ui.add(Button::new("Efficient!").smartstate(smartstateProvider.nxt())).clicked() {
///     // Handle click with improved performance
/// }
///
/// // Create a button in a horizontal layout
/// if ui.add_horizontal(Button::new("-")).clicked() {
///     // Handle click in horizontal layout
/// }
/// ```
///
/// # Visual States
/// Buttons have three visual states that provide user feedback:
/// 1. Normal - Default appearance with standard border and background
/// 2. Hover - Enhanced appearance when mouse/pointer is over the button
/// 3. Pressed - Highlighted appearance when clicked/pressed
///
/// # Styling
/// Buttons follow the [UI]'s current style settings including:
/// - Border colors and widths (normal and highlighted)
/// - Background colors (normal, highlighted, and pressed)
/// - Text color and font
/// - Padding and spacing
pub struct Button<'a> {
    label: &'a str,
    smartstate: Container<'a, Smartstate>,
}

impl<'a> Button<'a> {
    /// Creates a new button with the given text label.
    ///
    /// # Arguments
    /// * `label` - The text to display on the button
    ///
    /// # Returns
    /// A new Button instance with the specified label and no smartstate
    pub fn new(label: &'a str) -> Button<'a> {
        Button {
            label,
            smartstate: Container::empty(),
        }
    }

    /// Adds smartstate support to the button for incremental redrawing.
    ///
    /// When a smartstate is provided, the button will only redraw when its visual state changes,
    /// significantly improving performance especially on slower displays.
    ///
    /// # Arguments
    /// * `smartstate` - The smartstate to use for tracking the button's state
    ///
    /// # Returns
    /// Self with smartstate configured
    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

impl Widget for Button<'_> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // get size
        let font = ui.style().default_font;

        let mut text = Text::new(
            self.label,
            Point::new(0, 0),
            MonoTextStyle::new(&font, ui.style().text_color),
        );

        let height = ui.style().default_widget_height;
        let size = text.bounding_box();
        let padding = ui.style().spacing.button_padding;
        let border = ui.style().border_width;

        // allocate space
        let iresponse = ui.allocate_space(Size::new(
            size.size.width + 2 * padding.width + 2 * border,
            max(size.size.height + 2 * padding.height + 2 * border, height),
        ))?;

        // move text
        text.translate_mut(iresponse.area.top_left.add(Point::new(
            (padding.width + border) as i32,
            (padding.height + border) as i32,
        )));

        text.text_style.baseline = Baseline::Top;

        // check for click
        let click = matches!(iresponse.interaction, Interaction::Release(_));
        let down = matches!(
            iresponse.interaction,
            Interaction::Click(_) | Interaction::Drag(_)
        );

        // styles and smartstate
        let prevstate = self.smartstate.clone_inner();

        let rect_style = match iresponse.interaction {
            Interaction::None => {
                self.smartstate.modify(|st| *st = Smartstate::state(1));

                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().border_color)
                    .stroke_width(ui.style().border_width)
                    .fill_color(ui.style().item_background_color)
                    .build()
            }
            Interaction::Hover(_) => {
                self.smartstate.modify(|st| *st = Smartstate::state(2));

                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
                    .fill_color(ui.style().highlight_item_background_color)
                    .build()
            }

            _ => {
                self.smartstate.modify(|st| *st = Smartstate::state(3));

                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
                    .fill_color(ui.style().primary_color)
                    .build()
            }
        };

        if !self.smartstate.eq_option(&prevstate) {
            ui.start_drawing(&iresponse.area);

            ui.draw(
                &Rectangle::new(iresponse.area.top_left, iresponse.area.size)
                    .into_styled(rect_style),
            )
            .ok();
            ui.draw(&text).ok();

            ui.finalize()?;
        }

        Ok(Response::new(iresponse).set_clicked(click).set_down(down))
    }
}
