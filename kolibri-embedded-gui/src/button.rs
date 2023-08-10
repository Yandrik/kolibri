use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiResult, Interaction, Response, Ui, Widget};
use core::cmp::max;
use core::ops::Add;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::{Baseline, Text, TextStyleBuilder};

pub struct Button<'a> {
    label: &'a str,
    smartstate: Container<'a, Smartstate>,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Button<'a> {
        Button {
            label,
            smartstate: Container::empty(),
        }
    }

    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

impl Widget for Button<'_> {
    fn draw<
        DRAW: DrawTarget<Color = COL>,
        COL: PixelColor,
        CST: TextRenderer<Color = COL> + Clone,
    >(
        &mut self,
        ui: &mut Ui<DRAW, COL, CST>,
    ) -> GuiResult<Response> {
        // get size
        let mut text = Text::new(
            self.label,
            Point::new(0, 0),
            ui.style().default_text_style.0.clone(),
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
            (Interaction::None) => {
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
