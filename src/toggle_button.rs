use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiError, GuiResult, Interaction, Response, Ui, Widget};
use core::cmp::max;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::{Baseline, Text};

pub struct ToggleButton<'a> {
    label: &'a str,
    active: &'a mut bool,
    smartstate: Container<'a, Smartstate>,
}

impl<'a> ToggleButton<'a> {
    pub fn new(label: &'a str, active: &'a mut bool) -> ToggleButton<'a> {
        ToggleButton {
            label,
            active,
            smartstate: Container::empty(),
        }
    }

    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

impl Widget for ToggleButton<'_> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // Prepare text
        let font = ui.style().default_font;
        let mut text = Text::new(
            self.label,
            Point::zero(),
            MonoTextStyle::new(&font, ui.style().text_color),
        );

        // Determine size
        let text_bounds = text.bounding_box();
        let padding = ui.style().spacing.button_padding;
        let border = ui.style().border_width;
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

        // Determine widget style
        let style = match (*self.active, iresponse.interaction) {
            (true, Interaction::Click(_) | Interaction::Drag(_) | Interaction::Release(_)) => {
                self.smartstate.modify(|st| *st = Smartstate::state(1));
                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
                    .fill_color(ui.style().primary_color)
                    .build()
            }
            (true, Interaction::Hover(_)) => {
                self.smartstate.modify(|st| *st = Smartstate::state(2));
                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
                    .fill_color(ui.style().primary_color)
                    .build()
            }
            (true, _) => {
                self.smartstate.modify(|st| *st = Smartstate::state(3));
                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().border_color)
                    .stroke_width(ui.style().border_width)
                    .fill_color(ui.style().primary_color)
                    .build()
            }
            (false, Interaction::Click(_) | Interaction::Drag(_) | Interaction::Release(_)) => {
                self.smartstate.modify(|st| *st = Smartstate::state(4));
                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
                    .fill_color(ui.style().primary_color)
                    .build()
            }
            (false, Interaction::Hover(_)) => {
                self.smartstate.modify(|st| *st = Smartstate::state(5));
                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
                    .fill_color(ui.style().highlight_item_background_color)
                    .build()
            }
            (false, _) => {
                self.smartstate.modify(|st| *st = Smartstate::state(6));
                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().border_color)
                    .stroke_width(ui.style().border_width)
                    .fill_color(ui.style().item_background_color)
                    .build()
            }
        };

        let redraw = !self.smartstate.eq_option(&prevstate) || changed;

        if redraw {
            ui.start_drawing(&iresponse.area);

            let rect = Rectangle::new(iresponse.area.top_left, iresponse.area.size);
            ui.draw(&rect.into_styled(style))
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw ToggleButton")))?;
            ui.draw(&text)
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw ToggleButton label")))?;

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
