use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiError, GuiResult, Response, Ui, Widget};
use core::ops::Add;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::{Baseline, Text, TextStyleBuilder};

pub struct Label<'a> {
    text: &'a str,
    smartstate: Container<'a, Smartstate>,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str) -> Label {
        Label {
            text,
            smartstate: Container::empty(),
        }
    }

    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

impl Widget for Label<'_> {
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
            self.text,
            Point::new(0, 0),
            ui.style().default_text_style.0.clone(),
        );

        let size = text.bounding_box();

        // allocate space

        let space = ui.allocate_space(Size::new(size.size.width, size.size.height))?;

        // move text (center vertically)

        text.translate_mut(space.area.top_left.add(Point::new(
            0,
            (space.area.size.height - size.size.height) as i32 / 2,
        )));
        text.text_style.baseline = Baseline::Top;

        // check smartstate (a bool would work, but this is consistent with other widgets)
        let redraw = !self.smartstate.eq_option(&Some(Smartstate::state(0)));
        self.smartstate.modify(|st| *st = Smartstate::state(0));

        // draw

        if redraw {
            // clear background if necessary
            if !ui.cleared() {
                ui.clear_area(space.area)?;
            }

            ui.draw_raw(&mut text)
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw text")))?;
        }

        Ok(Response::new(space))
    }
}
