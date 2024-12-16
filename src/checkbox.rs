use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiError, GuiResult, Interaction, Response, Ui, Widget};
use core::cmp::max;
use core::ops::{Add, Sub};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_iconoir::prelude::*;
use embedded_iconoir::{size12px, size18px, size24px, size32px};

pub struct Checkbox<'a> {
    checked: &'a mut bool,
    smartstate: Container<'a, Smartstate>,
}

impl<'a> Checkbox<'a> {
    pub fn new(checked: &mut bool) -> Checkbox {
        Checkbox {
            checked,
            smartstate: Container::empty(),
        }
    }

    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

impl<'a> Checkbox<'a> {
    fn draw_icon<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
        icon: impl ImageDrawable<Color = COL>,
        area: &Rectangle,
        center_offset: Point,
    ) -> GuiResult<()> {
        let img = Image::new(
            &icon,
            area.top_left.add(
                Point::new(area.size.width as i32 / 2, area.size.height as i32 / 2)
                    .sub(center_offset),
            ),
        );
        ui.draw(&img)
            .map_err(|_| GuiError::DrawError(Some("Couldn't draw Checkbox")))
    }
}

impl<'a> Widget for Checkbox<'a> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // allocate space

        let size = ui.style().default_widget_height.max(ui.get_row_height());
        let padding = {
            // make square padding
            let pad = ui.style().spacing.default_padding;
            let biggest_pad = max(pad.width, pad.height);
            Size::new(biggest_pad, biggest_pad)
        };
        let iresponse = ui.allocate_space(Size::new(size, size))?;

        // check interaction

        let mut changed = false;
        if let Interaction::Release(_) = iresponse.interaction {
            *self.checked = !*self.checked;
            changed = true;
        }

        // styles

        // smartstate
        let prevstate = self.smartstate.clone_inner();

        let style = match iresponse.interaction {
            Interaction::Click(_) | Interaction::Drag(_) | Interaction::Release(_) => {
                self.smartstate.modify(|st| *st = Smartstate::state(1));
                PrimitiveStyleBuilder::new()
                    .fill_color(ui.style().primary_color)
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
            }
            Interaction::Hover(_) => {
                self.smartstate.modify(|st| *st = Smartstate::state(2));
                PrimitiveStyleBuilder::new()
                    .fill_color(ui.style().highlight_item_background_color)
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
            }
            _ => {
                self.smartstate.modify(|st| *st = Smartstate::state(3));
                PrimitiveStyleBuilder::new()
                    .fill_color(ui.style().item_background_color)
                    .stroke_color(ui.style().border_color)
                    .stroke_width(ui.style().border_width)
            }
        };

        let redraw = !self.smartstate.eq_option(&prevstate) || changed;

        if redraw {
            ui.start_drawing(&iresponse.area);

            // clear background if needed

            if !ui.cleared() && size - padding.height < 12 {
                ui.clear_area(iresponse.area)?;
            }

            // draw

            let rect = Rectangle::new(iresponse.area.top_left, iresponse.area.size);

            ui.draw(&rect.into_styled(style.build()))
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw Checkbox")))?;

            if *self.checked {
                match size - padding.width {
                    0..=18 => self.draw_icon(
                        ui,
                        size12px::actions::Check::new(ui.style().text_color),
                        &iresponse.area,
                        Point::new(6, 6),
                    ),
                    19..=23 => self.draw_icon(
                        ui,
                        size18px::actions::Check::new(ui.style().text_color),
                        &iresponse.area,
                        Point::new(9, 9),
                    ),
                    24..=32 => self.draw_icon(
                        ui,
                        size24px::actions::Check::new(ui.style().text_color),
                        &iresponse.area,
                        Point::new(12, 12),
                    ),
                    _ => self.draw_icon(
                        ui,
                        size32px::actions::Check::new(ui.style().text_color),
                        &iresponse.area,
                        Point::new(16, 16),
                    ),
                }?;
            }

            ui.finalize()?;
        }

        Ok(Response::new(iresponse).set_changed(changed))
    }
}
