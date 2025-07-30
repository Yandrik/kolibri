//! # Toggle Switch
//!
//! A customizable toggle switch widget that provides a simple on/off control.
//!
//! The toggle switch provides a slider-style control similar to those found in mobile applications,
//! with a background track and sliding knob that moves between on/off positions.
//! The widget supports customizable dimensions, colors based on theme, and hover/interaction states.
//!
//! This widget is part of the Kolibri embedded GUI framework's core widget set and integrates
//! with the framework's [Smartstate] system for efficient rendering.

use crate::smartstate::{Container, Smartstate};
use crate::style::WidgetStyle;
use crate::ui::{GuiError, GuiResult, Interaction, Response, Ui, Widget};
use core::cmp::max;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, CornerRadii, PrimitiveStyleBuilder, Rectangle, RoundedRectangle,
};

/// A toggle switch widget that provides an animated on/off control with a sliding knob.
///
/// The [ToggleSwitch] widget creates a visual control that allows users to toggle between
/// two states (on/off). It features a sliding knob that moves horizontally across a track
/// to indicate the current state.
///
/// The widget supports:
/// - Customizable width and height
/// - Theme-based colors for active/inactive states
/// - Interactive hover and click effects
/// - Integration with Kolibri's smartstate system for efficient rendering
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
/// # use kolibri_embedded_gui::toggle_switch::ToggleSwitch;
/// let mut state = false;
///
/// // Create a basic toggle switch
/// ui.add(ToggleSwitch::new(&mut state));
///
/// // Create a custom-sized toggle switch
/// ui.add(ToggleSwitch::new(&mut state)
///     .width(60)
///     .height(30));
/// ```
pub struct ToggleSwitch<'a, COL: PixelColor> {
    active: &'a mut bool,
    smartstate: Container<'a, Smartstate>,
    width: u32,
    height: u32,
    is_enabled: bool,
    is_modified: bool,
    custom_style: Option<WidgetStyle<COL>>,
}

impl<'a, COL: PixelColor> ToggleSwitch<'a, COL> {
    /// Creates a new [ToggleSwitch] instance with the provided mutable reference to the active state.
    ///
    /// The new [ToggleSwitch] will have a default width of 50 pixels and a height of 25 pixels.
    pub fn new(active: &'a mut bool) -> ToggleSwitch<'a, COL> {
        ToggleSwitch {
            active,
            smartstate: Container::empty(),
            width: 50,
            height: 25,
            is_enabled: true,
            is_modified: false,
            custom_style: None,
        }
    }

    /// Adds a [Smartstate] to the toggle switch for incremental redrawing.
    ///
    /// The smartstate is used to efficiently manage the rendering of the toggle switch.
    /// Through this [Smartstate], the toggle switch can leverage
    /// the smartstate system to avoid unnecessary redraws and improve performance.
    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }

    /// Sets the width of the toggle switch.
    ///
    /// The width determines the horizontal size of the switch's track. A minimum
    /// width of 30 pixels is enforced to ensure proper rendering and usability.
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
    /// # use kolibri_embedded_gui::toggle_switch::ToggleSwitch;
    /// let mut state = false;
    /// ui.add(ToggleSwitch::new(&mut state).width(60));
    /// ```
    pub fn width(mut self, width: u32) -> Self {
        self.width = max(width, 30); // Enforce a minimum width
        self
    }

    /// Sets the height of the toggle switch.
    ///
    /// The height determines the vertical size of the switch's track and knob.
    /// A minimum height of 15 pixels is enforced to ensure proper rendering and usability.
    ///
    /// ## Examples
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
    /// # use kolibri_embedded_gui::toggle_switch::ToggleSwitch;
    /// let mut state = false;
    ///
    /// loop {
    ///     // [...]
    ///     ui.add(ToggleSwitch::new(&mut state).height(30).width(60));
    /// }
    /// ```
    pub fn height(mut self, height: u32) -> Self {
        self.height = max(height, 15); // Enforce a minimum height
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

impl<COL: PixelColor> Widget<COL> for ToggleSwitch<'_, COL> {
    fn draw<DRAW: DrawTarget<Color = COL>>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        let widget_style = self.custom_style.unwrap_or_else(|| ui.style().widget);

        // Calculate total size including padding
        let padding = ui.style().spacing.button_padding;
        let total_size = Size::new(
            self.width + 2 * padding.width,
            self.height + 2 * padding.height,
        );

        // Allocate space in the UI
        let iresponse = ui.allocate_space(total_size)?;

        // Handle interaction
        let mut changed = false;
        if matches!(iresponse.interaction, Interaction::Release(_)) {
            *self.active = !*self.active;
            changed = true;
        }

        // Determine colors based on state
        let switch_color: COL;
        let knob_color: COL;
        let border_color: COL;
        let border_width: u32;

        if self.is_enabled {
            if *self.active {
                switch_color = widget_style.active.background_color;
            } else {
                switch_color = widget_style.normal.background_color
            };

            match iresponse.interaction {
                Interaction::Click(_) | Interaction::Drag(_) => {
                    knob_color = widget_style.active.background_color;
                    border_color = widget_style.active.border_color;
                    border_width = widget_style.active.border_width;
                }

                Interaction::Hover(_) => {
                    knob_color = widget_style.hover.background_color;
                    border_color = widget_style.hover.border_color;
                    border_width = widget_style.hover.border_width;
                }
                _ => {
                    knob_color = widget_style.normal.background_color;
                    border_color = widget_style.normal.border_color;
                    border_width = widget_style.normal.border_width;
                }
            };
        } else {
            switch_color = widget_style.disabled.background_color;
            knob_color = widget_style.disabled.background_color;
            border_color = widget_style.disabled.border_color;
            border_width = widget_style.disabled.border_width;
        }

        // Inside the draw method, replace the current smartstate handling with:

        let prevstate = self.smartstate.clone_inner();

        // Determine state based on both toggle state and interaction
        let state = match (iresponse.interaction, *self.active) {
            (Interaction::Click(_) | Interaction::Drag(_), true) => 1,
            (Interaction::Click(_) | Interaction::Drag(_), false) => 2,
            (Interaction::Hover(_), true) => 3,
            (Interaction::Hover(_), false) => 4,
            (_, true) => 5,
            (_, false) => 6,
        };

        self.smartstate.modify(|st| *st = Smartstate::state(state));

        // Determine if redraw is needed based on state change or active state change
        let redraw = !self.smartstate.eq_option(&prevstate) || changed || self.is_modified;

        if redraw {
            ui.start_drawing(&iresponse.area);

            // Define the switch background (rounded rectangle)
            let switch_rect = RoundedRectangle::new(
                Rectangle::new(
                    iresponse.area.top_left
                        + Point::new(padding.width as i32, padding.height as i32),
                    Size::new(self.width, self.height),
                ),
                CornerRadii::new(Size::new(self.height / 2, self.height / 2)),
            );

            let switch_style = PrimitiveStyleBuilder::new()
                .fill_color(switch_color)
                .stroke_color(border_color)
                .stroke_width(border_width)
                .build();

            ui.draw(&switch_rect.into_styled(switch_style))
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw ToggleSwitch background")))?;

            // Calculate knob position
            let knob_radius = (self.height / 2) - border_width;
            let knob_x = if *self.active {
                // Positioned on the right
                iresponse.area.top_left.x + padding.width as i32 + self.width as i32
                    - knob_radius as i32
                    - border_width as i32
            } else {
                // Positioned on the left
                iresponse.area.top_left.x
                    + padding.width as i32
                    + knob_radius as i32
                    + border_width as i32
            };

            let knob_center = Point::new(
                knob_x,
                iresponse.area.top_left.y + padding.height as i32 + (self.height / 2) as i32,
            );

            let knob = Circle::with_center(knob_center, knob_radius * 2 - 3);

            let knob_style = PrimitiveStyleBuilder::new()
                .fill_color(knob_color)
                .stroke_color(border_color)
                .stroke_width(2)
                .build();

            ui.draw(&knob.into_styled(knob_style))
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw ToggleSwitch knob")))?;

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
