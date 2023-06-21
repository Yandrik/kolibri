use crate::ui::{GuiResult, Ui, Widget};
use core::ops::Add;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::{Baseline, Text, TextStyleBuilder};

pub struct Spacer {
    space: Size,
}

impl Spacer {
    pub fn new(space: Size) -> Spacer {
        Spacer { space }
    }
}

impl Widget for Spacer {
    fn draw<
        DRAW: DrawTarget<Color = COL>,
        COL: PixelColor,
        CST: TextRenderer<Color = COL> + Clone,
    >(
        &self,
        ui: &mut Ui<DRAW, COL, CST>,
    ) -> GuiResult<()> {
        // allocate space
        let space = ui.allocate_space(self.space)?;

        Ok(())
    }
}
