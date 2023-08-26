use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiError, GuiResult, Response, Ui, Widget};
use core::marker::PhantomData;
use core::ops::Add;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_iconoir::prelude::*;

pub struct IconWidget<'a, Ico: IconoirIcon> {
    marker: PhantomData<Ico>,
    smartstate: Container<'a, Smartstate>,
}

impl<'a, Ico: IconoirIcon> IconWidget<'a, Ico> {
    /// Create a new IconWidget. The icon color will be ignored, if it's set.
    pub fn new(_icon: Ico) -> Self {
        Self {
            marker: PhantomData,
            smartstate: Container::empty(),
        }
    }

    pub fn new_from_type() -> Self {
        Self {
            marker: PhantomData,
            smartstate: Container::empty(),
        }
    }

    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

impl<Ico: IconoirIcon> Widget for IconWidget<'_, Ico> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // find size && allocate space
        let icon = Ico::new(ui.style().icon_color);
        let iresponse = ui.allocate_space(icon.size())?;

        let prevstate = self.smartstate.clone_inner();
        self.smartstate.modify(|sm| *sm = Smartstate::state(1));

        // draw icon

        if !self.smartstate.eq_option(&prevstate) {
            ui.start_drawing(&iresponse.area);

            if !ui.cleared() {
                ui.clear_area(iresponse.area)?;
            }

            let img = Image::new(
                &icon,
                iresponse.area.top_left.add(Point::new(
                    0, // center vertically
                    (iresponse.area.size.height - icon.size().height) as i32 / 2,
                )),
            );
            ui.draw(&img)
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw Icon")))?;

            ui.finalize()?;
        }

        Ok(Response::new(iresponse))
    }
}
