use crate::style::Style;
use crate::ui::{GuiError, GuiResult, Response, Ui, Widget};
use core::marker::PhantomData;
use core::ops::Add;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::renderer::{CharacterStyle, TextRenderer};
use embedded_graphics::text::{Baseline, Text, TextStyleBuilder};
use embedded_iconoir::prelude::*;
use embedded_iconoir::Icon;

pub struct IconWidget<Ico: IconoirIcon> {
    marker: PhantomData<Ico>,
}

impl<Ico: IconoirIcon> IconWidget<Ico> {
    /// Create a new IconWidget. The icon color will be ignored, if it's set.
    pub fn new(icon: Ico) -> Self {
        Self {
            marker: PhantomData,
        }
    }

    pub fn new_from_type() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<Ico: IconoirIcon> Widget for IconWidget<Ico> {
    fn draw<
        DRAW: DrawTarget<Color = COL>,
        COL: PixelColor,
        CST: TextRenderer<Color = COL> + Clone,
    >(
        &mut self,
        ui: &mut Ui<DRAW, COL, CST>,
    ) -> GuiResult<Response> {
        // find size && allocate space
        let icon = Ico::new(ui.style().icon_color);
        let space = ui.allocate_space(icon.size())?;

        // draw icon
        let img = Image::new(
            &icon,
            space.area.top_left.add(Point::new(
                0, // center vertically
                (space.area.size.height - icon.size().height) as i32 / 2,
            )),
        );
        ui.draw_raw(&img)
            .map_err(|_| GuiError::DrawError(Some("Couldn't draw Icon")))?;

        Ok(Response::new(space))
    }
}
