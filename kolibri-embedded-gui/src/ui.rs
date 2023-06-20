use crate::spacer::Spacer;
use crate::style::Style;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::Drawable;
use embedded_layout::align::vertical;
use embedded_layout::layout::linear::{spacing, Horizontal, LinearLayout, Vertical};
use embedded_layout::prelude::Chain;
use embedded_layout::view_group::ViewGroup;
use embedded_layout::View;
use std::cmp::max;
use std::ops::AddAssign;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GuiError {
    /// The widget is too large to fit in the bounds with the current constraints
    NoSpaceLeft,
}

pub type GuiResult<T> = Result<T, GuiError>;

pub trait Widget {
    fn draw<
        DRAW: DrawTarget<Color = COL>,
        COL: PixelColor,
        CST: TextRenderer<Color = COL> + Clone,
    >(
        &self,
        ui: &mut Ui<DRAW, COL, CST>,
    ) -> GuiResult<() /* TODO: Add Response Type */>;
}

struct Placer {
    row: u32,
    col: u32,
    pos: Point,
    col_height: u32,
    bounds: Size,
    wrap: bool,
}

impl Placer {
    fn next_no_wrap(&mut self, size: Size) -> GuiResult<Rectangle> {
        let wrap = self.wrap;
        self.wrap = false;
        let res = self.next(size);
        self.wrap = wrap;
        res
    }

    fn next(&mut self, size: Size) -> GuiResult<Rectangle> {
        // check that it's in bounds (size < bounds)
        if !self.check_bounds(size) {
            return Err(GuiError::NoSpaceLeft);
        }

        // check bounds
        let right = size.width + self.pos.x as u32;
        let mut bottom = max(self.col_height, size.height) + self.pos.y as u32;
        if !self.check_bounds(Size::new(right, bottom)) {
            if self.wrap {
                bottom = self.pos.y as u32 + max(self.col_height, size.height);
                // check that wrap fits
                if !self.check_bounds(Size::new(0, bottom)) {
                    return Err(GuiError::NoSpaceLeft);
                }

                // perform wrap
                self.new_row(size.height);
            } else {
                return Err(GuiError::NoSpaceLeft);
            }
        }

        // set new col height (expand if necessary)
        self.col_height = max(self.col_height, size.height);

        // set new position
        let item_pos = self.pos;
        self.pos = Point::new(right as i32, self.pos.y);

        Ok(Rectangle::new(item_pos, size))
    }

    fn row_size(&self) -> Size {
        Size::new(self.bounds.width, self.col_height)
    }

    fn space_available(&self) -> Size {
        Size::new(
            self.bounds.width - self.pos.x as u32,
            self.bounds.height - self.pos.y as u32,
        )
    }

    fn new_row(&mut self, height: u32) {
        self.row += 1;
        self.col = 0;
        self.pos = Point::new(0, self.pos.y + self.col_height as i32);
        self.col_height = height;
    }

    fn col_height(&self) -> u32 {
        self.col_height
    }

    /// Check whether a size is in bounds of the widget (<= widget_size)
    fn check_bounds(&self, pos: Size) -> bool {
        pos.width as u32 <= self.bounds.width && pos.height <= self.bounds.height
    }
}

pub struct Ui<'a, DRAW, COL, DefaultCharstyle>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
    DefaultCharstyle: TextRenderer<Color = COL> + Clone,
{
    bounds: Rectangle,
    drawable: &'a mut DRAW,
    style: Style<COL, DefaultCharstyle>,
    next_auto_id_source: u16,
    placer: Placer,
}

impl<COL, DefaultCharstyle, DRAW> Ui<'_, DRAW, COL, DefaultCharstyle>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
    DefaultCharstyle: TextRenderer<Color = COL> + Clone,
{
    pub fn new(
        drawable: &mut DRAW,
        bounds: Rectangle,
        style: Style<COL, DefaultCharstyle>,
    ) -> Ui<DRAW, COL, DefaultCharstyle> {
        let placer = Placer {
            row: 0,
            col: 0,
            pos: Point::zero(),
            col_height: 0,
            bounds: bounds.size,
            wrap: true,
        };

        Ui {
            bounds,
            drawable,
            style,
            next_auto_id_source: 0,
            placer,
        }
    }

    pub fn new_fullscreen(
        drawable: &mut DRAW,
        style: Style<COL, DefaultCharstyle>,
    ) -> Ui<DRAW, COL, DefaultCharstyle> {
        let bounds = drawable.bounding_box();
        Ui::new(drawable, bounds, style)
    }

    pub fn add(&mut self, widget: impl Widget) -> GuiResult<()> {
        // draw widget. TODO: Add new auto ID
        self.add_raw(widget)?;

        // create new row
        self.placer
            .new_row(self.style().spacing.item_spacing.height);

        self.placer.new_row(0);

        Ok(())
    }

    /// Add a widget horizontally to the layout to the current row
    pub fn add_horizontal(&mut self, height: Option<u32>, widget: impl Widget) -> GuiResult<()> {
        // set row height to the given
        self.expand_row_height(height.unwrap_or(0));

        widget.draw(self)?;
        // ignore space alignment errors (those are "fine". If wrapping is enabled,
        // the next widget will be placed on the next row, without any space in between.)
        self.allocate_space_no_wrap(self.style().spacing.item_spacing)
            .ok();

        Ok(())
    }

    pub fn add_raw(&mut self, widget: impl Widget) -> GuiResult<()> {
        widget.draw(self)
    }

    pub fn style(&self) -> &Style<COL, DefaultCharstyle> {
        &self.style
    }

    pub fn new_row(&mut self, height: u32) {
        self.placer.new_row(height);
    }

    /// Increase the height of the current row to the given height, if it is
    /// larger than the current height
    pub fn expand_row_height(&mut self, height: u32) {
        self.placer.col_height = height;
    }

    pub fn draw_raw<OUT>(
        &mut self,
        to_draw: &impl Drawable<Color = COL, Output = OUT>,
    ) -> Result<OUT, DRAW::Error> {
        to_draw.draw(self.drawable)
    }

    pub fn space_available(&self) -> Size {
        self.placer.space_available()
    }

    pub fn allocate_exact_size(&mut self, desired_size: Size) -> GuiResult<()> {
        let allocated = self.allocate_space(desired_size);
        allocated.map(|_| ())
    }

    pub fn allocate_space(&mut self, desired_size: Size) -> GuiResult<Rectangle> {
        self.placer.next(desired_size).map(|mut rect| {
            rect.top_left.add_assign(self.bounds.top_left);
            rect
        })
    }

    pub fn allocate_space_no_wrap(&mut self, desired_size: Size) -> GuiResult<Rectangle> {
        self.placer.next_no_wrap(desired_size).map(|mut rect| {
            rect.top_left.add_assign(self.bounds.top_left);
            rect
        })
    }
}
