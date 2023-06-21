use crate::ui::{GuiResult, Ui, Widget};
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
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Button {
        Button { label }
    }
}

impl Widget for Button<'_> {
    fn draw<
        DRAW: DrawTarget<Color = COL>,
        COL: PixelColor,
        CST: TextRenderer<Color = COL> + Clone,
    >(
        &self,
        ui: &mut Ui<DRAW, COL, CST>,
    ) -> GuiResult<()> {
        // get size
        let mut text = Text::new(
            self.label,
            Point::new(0, 0),
            ui.style().default_text_style.0.clone(),
        );

        let size = text.bounding_box();
        let spacing = ui.style().spacing.item_spacing;
        let padding = ui.style().spacing.button_padding;
        let border = ui.style().border_width;

        // allocate space
        let space = ui.allocate_space(Size::new(
            size.size.width + 2 * padding.width + 2 * border + spacing.width,
            size.size.height + 2 * padding.height + 2 * border,
        ))?;

        // move text
        text.translate_mut(space.top_left.add(Point::new(
            (padding.width + border) as i32,
            (padding.height + border) as i32,
        )));

        text.text_style.baseline = Baseline::Top;

        // styles
        let rect_style = PrimitiveStyleBuilder::new()
            .stroke_color(ui.style().border_color)
            .stroke_width(ui.style().border_width)
            .fill_color(ui.style().background_color)
            .build();

        // draw

        ui.draw_raw(&Rectangle::new(space.top_left, space.size).into_styled(rect_style))
            .ok();
        ui.draw_raw(&text).ok();

        Ok(())
    }
}
