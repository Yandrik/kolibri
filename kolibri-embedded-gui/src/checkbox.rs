use crate::ui::{GuiError, GuiResult, Interaction, Response, Ui, Widget};
use core::cmp::{max, min};
use core::ops::{Add, Sub};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::{Baseline, Text, TextStyleBuilder};
use embedded_iconoir::prelude::IconoirNewIcon;
use embedded_iconoir::prelude::*;
use embedded_iconoir::{size12px, size18px, size24px, size32px, Icon};

pub struct Checkbox<'a> {
    checked: &'a mut bool,
}

impl Checkbox<'_> {
    pub fn new(checked: &mut bool) -> Checkbox {
        Checkbox { checked }
    }
}

impl Checkbox<'_> {
    fn draw_icon<
        DRAW: DrawTarget<Color = COL>,
        COL: PixelColor,
        CST: TextRenderer<Color = COL> + Clone,
    >(
        &mut self,
        painter: &mut Ui<DRAW, COL>,
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
        ui.draw_raw(&img)
            .map_err(|_| GuiError::DrawError(Some("Couldn't draw Checkbox")))
    }
}

impl<'a> Widget for Checkbox<'a> {
    fn draw<
        DRAW: DrawTarget<Color = COL>,
        COL: PixelColor,
        CST: TextRenderer<Color = COL> + Clone,
    >(
        &mut self,
        ui: &mut Ui<DRAW, COL, CST>,
    ) -> GuiResult<Response> {
        // allocate space

        let size = ui.style().default_widget_height;
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

        let style = match iresponse.interaction {
            Interaction::Click(_) | Interaction::Drag(_) | Interaction::Release(_) => {
                PrimitiveStyleBuilder::new()
                    .fill_color(ui.style().primary_color)
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
            }
            Interaction::Hover(_) => PrimitiveStyleBuilder::new()
                .fill_color(ui.style().highlight_item_background_color)
                .stroke_color(ui.style().highlight_border_color)
                .stroke_width(ui.style().highlight_border_width),
            _ => PrimitiveStyleBuilder::new()
                .fill_color(ui.style().item_background_color)
                .stroke_color(ui.style().border_color)
                .stroke_width(ui.style().border_width),
        };

        // clear background if needed

        if !ui.cleared() && size - padding.height < 12 {
            ui.clear_area(iresponse.area)?;
        }

        // draw

        let rect = Rectangle::new(
            iresponse
                .area
                .top_left
                .add(Point::new(padding.width as i32, padding.height as i32)),
            iresponse.area.size.saturating_sub(padding * 2),
        );

        ui.draw_raw(&rect.into_styled(style.build()))
            .map_err(|_| GuiError::DrawError(Some("Couldn't draw Checkbox")))?;

        if *self.checked {
            match size - padding.width {
                0..=18 => {
                    (self.draw_icon(
                        ui,
                        size12px::actions::Check::new(ui.style().text_color),
                        &iresponse.area,
                        Point::new(6, 6),
                    ))
                }
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

        Ok(Response::new(iresponse).set_changed(changed))
    }
}
