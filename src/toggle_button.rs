//! # Toggle Button Widget
//!
//! A customizable toggle button widget that provides a clickable on/off control.
//!
//! The toggle button provides a traditional button-style control that maintains its state,
//! featuring different visual styles for active and inactive states. It supports text labels
//! and integrates with the framework's theming system for consistent appearance.
//!
//! This widget is part of the Kolibri embedded GUI framework's core widget set and integrates
//! with the framework's [Smartstate] system for efficient rendering.
//!
use crate::smartstate::{Container, Smartstate};
use crate::style::WidgetStyle;
use crate::ui::{GuiError, GuiResult, Interaction, Response, Ui, Widget};
use core::cmp::max;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::{Baseline, Text};

/// A button widget that can be toggled on and off.
///
/// [ToggleButton] provides a clickable button that maintains an on/off state. When clicked,
/// it toggles between these states and displays different visual styles accordingly.
/// The button includes a text label and supports various interaction states like hover and click.
///
/// ## Examples
///
/// ```no_run
/// # use embedded_graphics::pixelcolor::Rgb565;
/// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
/// # use kolibri_embedded_gui::style::medsize_rgb565_style;
/// # use kolibri_embedded_gui::ui::Ui;
/// # use embedded_graphics::prelude::*;
/// # use embedded_graphics::primitives::Rectangle;
/// # use embedded_iconoir::prelude::*;
/// # use kolibri_embedded_gui::ui::*;
/// # use kolibri_embedded_gui::label::*;
/// # use kolibri_embedded_gui::smartstate::*;
/// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
/// # let output_settings = OutputSettingsBuilder::new().build();
/// # let mut window = Window::new("Kolibri Example", &output_settings);
/// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
/// # use kolibri_embedded_gui::toggle_button::ToggleButton;
/// let mut state = false;
///
/// loop {
///     // [...]
///     ui.add(ToggleButton::new("Toggle Me", &mut state));
/// }
/// ```
pub struct ToggleButton<'a, COL: PixelColor> {
    label: &'a str,
    active: &'a mut bool,
    smartstate: Container<'a, Smartstate>,
    is_enabled: bool,
    is_modified: bool,
    custom_style: Option<WidgetStyle<COL>>,
}

impl<'a, COL: PixelColor> ToggleButton<'a, COL> {
    /// Creates a new [ToggleButton] with the given label and active state.
    ///
    /// The `label` parameter is the text to display on the button, and the `active`
    /// parameter is a mutable reference to a boolean that tracks the on/off state
    /// of the button.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # use kolibri_embedded_gui::smartstate::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # use kolibri_embedded_gui::toggle_button::ToggleButton;
    /// let mut state = false;
    /// let mut smartstateProvider = SmartstateProvider::<20>::new();
    ///
    /// loop {
    ///     // [...]
    ///     if ui.add(ToggleButton::new("Toggle Me", &mut state)).changed() {
    ///         // handle toggle
    ///     }
    ///     // or with smartstate:
    ///    if ui.add(ToggleButton::new("Toggle Me", &mut state).smartstate(smartstateProvider.nxt())).changed() {
    ///        // handle toggle
    ///    }
    ///
    /// }
    pub fn new(label: &'a str, active: &'a mut bool) -> ToggleButton<'a, COL> {
        ToggleButton {
            label,
            active,
            smartstate: Container::empty(),
            is_enabled: true,
            is_modified: false,
            custom_style: None,
        }
    }

    /// Attaches a [Smartstate] to the toggle button for incremental redrawing.
    ///
    /// Smartstates enable efficient rendering by tracking the button's visual state
    /// and only redrawing when necessary.
    ///
    /// Returns self for method chaining.
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

impl<COL: PixelColor> Widget<COL> for ToggleButton<'_, COL> {
    fn draw<DRAW: DrawTarget<Color = COL>>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // Prepare text
        let font = ui.style().default_font;
        let widget_style = self.custom_style.unwrap_or_else(|| ui.style().widget);

        let mut text = Text::new(
            self.label,
            Point::zero(),
            MonoTextStyle::new(&font, widget_style.normal.foreground_color),
        );

        // Determine size
        let text_bounds = text.bounding_box();
        let padding = ui.style().spacing.button_padding;
        let border = widget_style.normal.border_width;
        let height = ui.style().default_widget_height;

        let size = Size::new(
            text_bounds.size.width + 2 * padding.width + 2 * border,
            max(
                text_bounds.size.height + 2 * padding.height + 2 * border,
                height,
            ),
        );

        // Allocate space
        let iresponse = ui.allocate_space(size)?;

        // Position text
        text.translate_mut(
            iresponse.area.top_left
                + Point::new(
                    (padding.width + border) as i32,
                    (padding.height + border) as i32,
                ),
        );
        text.text_style.baseline = Baseline::Top;

        // Handle interaction
        let mut changed = false;
        if let Interaction::Release(_) = iresponse.interaction {
            *self.active = !*self.active;
            changed = true;
        }

        // Determine styles based on state and interaction
        let prevstate = self.smartstate.clone_inner();

        // Determine text color
        text.character_style.text_color = match (*self.active, iresponse.interaction) {
            (true, Interaction::Click(_) | Interaction::Drag(_) | Interaction::Release(_)) => {
                Some(widget_style.active.foreground_color)
            }
            (true, Interaction::Hover(_)) => Some(widget_style.hover.foreground_color),
            (true, _) => Some(widget_style.active.foreground_color),
            (false, Interaction::Click(_) | Interaction::Drag(_) | Interaction::Release(_)) => {
                Some(widget_style.hover.foreground_color)
            }
            (false, Interaction::Hover(_)) => Some(widget_style.hover.foreground_color),
            (false, _) => Some(widget_style.normal.foreground_color),
        };

        // Determine widget style
        let style = match (*self.active, iresponse.interaction) {
            (true, Interaction::Click(_) | Interaction::Drag(_) | Interaction::Release(_)) => {
                self.smartstate.modify(|st| *st = Smartstate::state(1));
                PrimitiveStyleBuilder::new()
                    .stroke_color(widget_style.active.border_color)
                    .stroke_width(widget_style.active.border_width)
                    .fill_color(widget_style.active.background_color)
                    .build()
            }
            (true, Interaction::Hover(_)) => {
                self.smartstate.modify(|st| *st = Smartstate::state(2));
                PrimitiveStyleBuilder::new()
                    .stroke_color(widget_style.hover.border_color)
                    .stroke_width(widget_style.hover.border_width)
                    .fill_color(widget_style.hover.background_color)
                    .build()
            }
            (true, _) => {
                self.smartstate.modify(|st| *st = Smartstate::state(3));
                PrimitiveStyleBuilder::new()
                    .stroke_color(widget_style.active.border_color)
                    .stroke_width(widget_style.active.border_width)
                    .fill_color(widget_style.active.background_color)
                    .build()
            }
            (false, Interaction::Click(_) | Interaction::Drag(_) | Interaction::Release(_)) => {
                self.smartstate.modify(|st| *st = Smartstate::state(4));
                PrimitiveStyleBuilder::new()
                    .stroke_color(widget_style.hover.border_color)
                    .stroke_width(widget_style.hover.border_width)
                    .fill_color(widget_style.hover.background_color)
                    .build()
            }
            (false, Interaction::Hover(_)) => {
                self.smartstate.modify(|st| *st = Smartstate::state(5));
                PrimitiveStyleBuilder::new()
                    .stroke_color(widget_style.hover.border_color)
                    .stroke_width(widget_style.hover.border_width)
                    .fill_color(widget_style.hover.background_color)
                    .build()
            }
            (false, _) => {
                self.smartstate.modify(|st| *st = Smartstate::state(6));
                PrimitiveStyleBuilder::new()
                    .stroke_color(widget_style.normal.border_color)
                    .stroke_width(widget_style.normal.border_width)
                    .fill_color(widget_style.normal.background_color)
                    .build()
            }
        };

        let redraw = !self.smartstate.eq_option(&prevstate) || changed || self.is_modified;

        if redraw {
            ui.start_drawing(&iresponse.area);

            let rect = Rectangle::new(iresponse.area.top_left, iresponse.area.size);
            ui.draw(&rect.into_styled(style))
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw ToggleButton")))?;
            ui.draw(&text)
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw ToggleButton label")))?;

            ui.finalize()?;
        }
        self.is_modified = false;

        if self.is_enabled {
            let click = matches!(iresponse.interaction, Interaction::Release(_));
            let down = matches!(
                iresponse.interaction,
                Interaction::Click(_) | Interaction::Drag(_)
            );

            Ok(Response::new(iresponse)
                .set_clicked(click)
                .set_down(down)
                .set_changed(changed))
        } else {
            Ok(Response::new(iresponse)
                .set_clicked(false)
                .set_down(false)
                .set_changed(false))
        }
    }
}
