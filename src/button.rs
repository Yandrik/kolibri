//! # Button Widget
//!
//! See [Button] for more info.

use crate::smartstate::{Container, Smartstate};
use crate::style::WidgetStyle;
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
/// - Visual feedback for different interaction states (normal, hover, pressed, disabled)
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
/// Buttons have four visual states that provide user feedback:
/// 1. Normal - Default appearance with standard border and background
/// 2. Hover - Enhanced appearance when mouse/pointer is over the button
/// 3. Pressed - Highlighted appearance when clicked/pressed
/// 4. Disabled - diminished appearance when disabled
///
/// # Styling
/// Buttons follow the [UI]'s current style settings including:
/// - Padding, spacing and font from style
/// - Border color, border width, text color and background color from style.widget
/// - An optional custom widget style may be provided to override the defaults
pub struct Button<'a, COL: PixelColor> {
    label: &'a str,
    smartstate: Container<'a, Smartstate>,
    is_enabled: bool,
    is_modified: bool,
    custom_style: Option<WidgetStyle<COL>>,
}

impl<'a, COL: PixelColor> Button<'a, COL> {
    /// Creates a new button with the given text label.
    ///
    /// # Arguments
    /// * `label` - The text to display on the button
    ///
    /// # Returns
    /// A new Button instance with the specified label and no smartstate
    pub fn new(label: &'a str) -> Button<'a, COL> {
        Button {
            label,
            smartstate: Container::empty(),
            is_enabled: true,
            is_modified: false,
            custom_style: None,
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

    /// Enables or disables the widget - will not respond to interaction
    ///
    /// # Arguments
    /// * `enabled` - if the widget should be enabled (true) or disabled(false)
    ///
    /// # Returns
    /// Self with is_enabled set
    pub fn enable(mut self, enabled: &bool) -> Self {
        self.is_modified = true;
        self.is_enabled = *enabled;
        self
    }

    /// Specifies the context for the widget to determine how it is styled
    ///
    /// # Arguments
    /// * `context` - Context::Normal, Context::Primary, Context::Secondary
    ///
    /// # Returns
    /// Self with context set
    pub fn with_widget_style(mut self, style: WidgetStyle<COL>) -> Self {
        self.is_modified = true;
        self.custom_style = Some(style);
        self
    }
}

impl<COL: PixelColor> Widget<COL> for Button<'_, COL> {
    fn draw<DRAW: DrawTarget<Color = COL>>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // get size
        let font = ui.style().default_font;

        let widget_style = self.custom_style.unwrap_or_else(|| ui.style().widget);

        let mut text = Text::new(
            self.label,
            Point::new(0, 0),
            MonoTextStyle::new(&font, widget_style.normal.foreground_color),
        );

        let height = ui.style().default_widget_height;
        let size = text.bounding_box();
        let padding = ui.style().spacing.button_padding;
        let border = widget_style.normal.border_width;

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
        let rect_style: embedded_graphics::primitives::PrimitiveStyle<COL>;

        if self.is_enabled {
            rect_style = match iresponse.interaction {
                Interaction::None => {
                    self.smartstate.modify(|st| *st = Smartstate::state(1));

                    PrimitiveStyleBuilder::new()
                        .stroke_color(widget_style.normal.border_color)
                        .stroke_width(widget_style.normal.border_width)
                        .fill_color(widget_style.normal.background_color)
                        .build()
                }
                Interaction::Hover(_) => {
                    self.smartstate.modify(|st| *st = Smartstate::state(2));

                    PrimitiveStyleBuilder::new()
                        .stroke_color(widget_style.hover.border_color)
                        .stroke_width(widget_style.hover.border_width)
                        .fill_color(widget_style.hover.background_color)
                        .build()
                }

                _ => {
                    self.smartstate.modify(|st| *st = Smartstate::state(3));

                    PrimitiveStyleBuilder::new()
                        .stroke_color(widget_style.active.border_color)
                        .stroke_width(widget_style.active.border_width)
                        .fill_color(widget_style.active.background_color)
                        .build()
                }
            };
            text.character_style.text_color = match iresponse.interaction {
                Interaction::None => Some(widget_style.normal.foreground_color),
                Interaction::Hover(_) => Some(widget_style.hover.foreground_color),

                _ => Some(widget_style.active.foreground_color),
            };
        } else {
            rect_style = PrimitiveStyleBuilder::new()
                .stroke_color(widget_style.disabled.border_color)
                .stroke_width(widget_style.disabled.border_width)
                .fill_color(widget_style.disabled.background_color)
                .build();
            text.character_style.text_color = Some(widget_style.disabled.foreground_color);
        }
        if !self.smartstate.eq_option(&prevstate) || self.is_modified {
            ui.start_drawing(&iresponse.area);

            ui.draw(
                &Rectangle::new(iresponse.area.top_left, iresponse.area.size)
                    .into_styled(rect_style),
            )
            .ok();
            ui.draw(&text).ok();

            ui.finalize()?;
        }
        self.is_modified = false;

        if self.is_enabled {
            Ok(Response::new(iresponse).set_clicked(click).set_down(down))
        } else {
            Ok(Response::new(iresponse).set_clicked(false).set_down(false))
        }
    }
}
