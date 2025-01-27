// File: kolibri-embedded-gui/src/toggle_switch.rs

use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiError, GuiResult, Interaction, Response, Ui, Widget};
use core::cmp::max;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, CornerRadii, PrimitiveStyleBuilder, Rectangle, RoundedRectangle,
};

pub struct ToggleSwitch<'a> {
    active: &'a mut bool,
    smartstate: Container<'a, Smartstate>,
    width: u32,
    height: u32,
}

impl<'a> ToggleSwitch<'a> {
    pub fn new(active: &'a mut bool) -> ToggleSwitch<'a> {
        ToggleSwitch {
            active,
            smartstate: Container::empty(),
            width: 50,  // Default width
            height: 25, // Default height
        }
    }

    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }

    /// Set the width of the toggle switch.
    /// Minimum width is enforced to ensure proper rendering.
    pub fn width(mut self, width: u32) -> Self {
        self.width = max(width, 30); // Enforce a minimum width
        self
    }

    /// Set the height of the toggle switch.
    /// Minimum height is enforced to ensure proper rendering.
    pub fn height(mut self, height: u32) -> Self {
        self.height = max(height, 15); // Enforce a minimum height
        self
    }
}

impl Widget for ToggleSwitch<'_> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
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

        // Colors for active and inactive states
        let switch_color = if *self.active {
            ui.style().primary_color
        } else {
            ui.style().item_background_color
        };

        let knob_color = match iresponse.interaction {
            Interaction::Click(_) | Interaction::Drag(_) => ui.style().primary_color,
            Interaction::Hover(_) => ui.style().highlight_item_background_color,
            _ => ui.style().item_background_color,
        };

        // Determine border color based on interaction
        let border_color = match iresponse.interaction {
            Interaction::Hover(_) => ui.style().highlight_border_color,
            _ => ui.style().border_color,
        };

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
        let redraw = !self.smartstate.eq_option(&prevstate) || changed;

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
                .stroke_width(ui.style().border_width)
                .build();

            ui.draw(&switch_rect.into_styled(switch_style))
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw ToggleSwitch background")))?;

            // Calculate knob position
            let knob_radius = (self.height / 2) - ui.style().border_width;
            let knob_x = if *self.active {
                // Positioned on the right
                iresponse.area.top_left.x + padding.width as i32 + self.width as i32
                    - knob_radius as i32
                    - ui.style().border_width as i32
            } else {
                // Positioned on the left
                iresponse.area.top_left.x
                    + padding.width as i32
                    + knob_radius as i32
                    + ui.style().border_width as i32
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

        let click = matches!(iresponse.interaction, Interaction::Release(_));
        let down = matches!(
            iresponse.interaction,
            Interaction::Click(_) | Interaction::Drag(_)
        );

        Ok(Response::new(iresponse)
            .set_clicked(click)
            .set_down(down)
            .set_changed(changed))
    }
}
