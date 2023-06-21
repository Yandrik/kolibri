use crate::spacer::Spacer;
use crate::style::Style;
use core::cmp::max;
use core::fmt::Debug;
use core::ops::{Add, AddAssign, Sub};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::OriginDimensions;
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::{
    PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StyledDrawable,
};
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::Drawable;
use embedded_iconoir::prelude::IconoirNewIcon;
use embedded_iconoir::{make_icon_category, Icon};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GuiError {
    /// The widget is too large to fit in the bounds with the current constraints
    NoSpaceLeft,
    /// The Drawable returned an error while drawing
    // TODO: (maybe) add better error handling here
    // The rationale for the 'static str is that generics are annoying to implement,
    // and that generic would need to be everywhere, basically, as returning just () as an
    // error would make handling wierd and complicated.
    // The goal of this library is to be trivially easy, not to be 100% generic.
    // If you have a better idea, a PR is much appreciated.
    // (maybe a Box<dyn Error> with alloc feature gate? Or a 'String' (heapless / alloc) and format!()?)
    DrawError(Option<&'static str>),
}

pub type GuiResult<T> = Result<T, GuiError>;

pub struct InternalResponse {
    pub area: Rectangle,
    pub interaction: Interaction,
}

/// Response for UI interaction / space allocation and such
pub struct Response {
    pub internal: InternalResponse,
    /// Whether the widget was clicked (as in successfully interacted with)
    pub click: bool,
    /// Marker to tell the UI that this widget was redrawn this frame (if you don't have redraw
    /// / change detection, just set this to `true`, as you are redrawing every frame)
    ///
    /// **The default for this is `true`**.
    pub redraw: bool,

    /// What the underlying data changed?
    ///
    /// e.g. the slider was dragged, etc.
    /// Always `false` for something like a [`Button`](crate::button::Button).
    pub changed: bool,
}

// builder pattern
impl Response {
    pub fn new(raw: InternalResponse) -> Response {
        Response {
            internal: raw,
            click: false,
            redraw: true,
            changed: false,
        }
    }
    pub fn set_clicked(mut self, clicked: bool) -> Self {
        self.click = clicked;
        self
    }

    pub fn set_redraw(mut self, redraw: bool) -> Self {
        self.redraw = redraw;
        self
    }

    pub fn set_changed(mut self, changed: bool) -> Self {
        self.changed = changed;
        self
    }

    /// Check whether the widget was clicked (as in successfully interacted with)
    pub fn clicked(&self) -> bool {
        self.click
    }

    /// Check whether the widget was redrawn this frame
    pub fn redrawn(&self) -> bool {
        self.redraw
    }

    /// Check whether the underlying data changed (e.g. slider was moved)
    pub fn changed(&self) -> bool {
        self.changed
    }
}

pub trait Widget {
    fn draw<
        DRAW: DrawTarget<Color = COL>,
        COL: PixelColor,
        CST: TextRenderer<Color = COL> + Clone,
    >(
        &mut self,
        ui: &mut Ui<DRAW, COL, CST>,
    ) -> GuiResult<Response>;
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

        Ok(Rectangle::new(
            item_pos,
            Size::new(size.width, self.col_height),
        ))
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

/// Interaction with the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Interaction {
    /// A click event (mouse, touch, etc. down)
    Click(Point),
    /// A drag event (mouse, touch, etc. move while clicked)
    Drag(Point),
    /// A release event (mouse, touch, etc. up)
    Release(Point),
    /// A hover event (mouse, touch, etc. move while not clicked).
    /// Generally not applicable to touch screens.
    Hover(Point),
    /// No interaction
    None,
}

impl Interaction {
    fn get_point(&self) -> Option<Point> {
        match self {
            Interaction::Click(p) => Some(*p),
            Interaction::Drag(p) => Some(*p),
            Interaction::Release(p) => Some(*p),
            Interaction::Hover(p) => Some(*p),
            Interaction::None => None,
        }
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
    interact: Interaction,
    /// Whether the UI was background-cleared this frame
    cleared: bool,
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
        // set bounds to internal bounds (apply padding)
        let bounds = Rectangle::new(
            bounds.top_left.add(Point::new(
                style.spacing.window_border_padding.height as i32,
                style.spacing.window_border_padding.width as i32,
            )),
            bounds
                .size
                .saturating_sub(style.spacing.window_border_padding * 2),
        );

        // set up placer
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
            interact: Interaction::None,
            cleared: false,
        }
    }

    pub fn new_fullscreen(
        drawable: &mut DRAW,
        style: Style<COL, DefaultCharstyle>,
    ) -> Ui<DRAW, COL, DefaultCharstyle> {
        let bounds = drawable.bounding_box();
        Ui::new(drawable, bounds, style)
    }

    pub fn clear_background(&mut self) -> GuiResult<()> {
        self.cleared = true;

        // clear background
        let real_bg = Rectangle::new(
            self.bounds.top_left.sub(Point::new(
                self.style.spacing.window_border_padding.width as i32,
                self.style.spacing.window_border_padding.height as i32,
            )),
            self.bounds
                .size
                .saturating_add(self.style.spacing.window_border_padding * 2),
        );

        real_bg
            .draw_styled(
                &PrimitiveStyleBuilder::new()
                    .fill_color(self.style.background_color)
                    .build(),
                self.drawable,
            )
            .map_err(|_| GuiError::DrawError(Some("Couldn't clear GUI Background")))
    }

    /// Return whether the UI was background-cleared this frame
    pub fn cleared(&self) -> bool {
        self.cleared
    }

    pub fn interact(&mut self, interaction: Interaction) {
        self.interact = interaction;
    }

    pub fn add(&mut self, widget: impl Widget) -> Response {
        // draw widget. TODO: Add new auto ID
        let resp = self.add_raw(widget).expect("Couldn't add widget to UI");

        // create new row
        self.placer
            .new_row(self.style().spacing.item_spacing.height);

        self.placer.new_row(self.style().default_widget_height);

        resp
    }

    /// Add a widget horizontally to the layout to the current row
    pub fn add_horizontal(&mut self, height: Option<u32>, mut widget: impl Widget) -> Response {
        // set row height to the given
        self.expand_row_height(height.unwrap_or(0));

        let resp = self.add_raw(widget).expect("Couldn't add widget to UI");
        // ignore space alignment errors (those are "fine". If wrapping is enabled,
        // the next widget will be placed on the next row, without any space in between.)
        self.allocate_space_no_wrap(self.style().spacing.item_spacing)
            .ok();

        resp
    }

    pub fn add_raw(&mut self, mut widget: impl Widget) -> GuiResult<Response> {
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

    pub fn clear_area(&mut self, area: Rectangle) -> GuiResult<()> {
        area.draw_styled(
            &PrimitiveStyle::with_fill(self.style.background_color),
            self.drawable,
        )
        .map_err(|_| GuiError::DrawError(Some("Couldn't clear area")))
    }

    pub fn space_available(&self) -> Size {
        self.placer.space_available()
    }

    pub fn check_interact(&self, area: Rectangle) -> Interaction {
        if self
            .interact
            .get_point()
            .map(|pt| area.contains(pt))
            .unwrap_or(false)
        {
            self.interact
        } else {
            Interaction::None
        }
    }

    /// For now, only stub method.
    pub fn allocate_exact_size(&mut self, desired_size: Size) -> GuiResult<InternalResponse> {
        let allocated = self.allocate_space(desired_size);
        allocated
    }

    pub fn allocate_space(&mut self, desired_size: Size) -> GuiResult<InternalResponse> {
        let rect = self.placer.next(desired_size).map(|mut rect| {
            rect.top_left.add_assign(self.bounds.top_left);
            rect
        })?;
        let inter = self.check_interact(rect);

        Ok(InternalResponse {
            area: rect,
            interaction: inter,
        })
    }

    pub fn allocate_space_no_wrap(&mut self, desired_size: Size) -> GuiResult<InternalResponse> {
        let area = self.placer.next_no_wrap(desired_size).map(|mut rect| {
            rect.top_left.add_assign(self.bounds.top_left);
            rect
        })?;

        Ok(InternalResponse {
            area,
            interaction: Interaction::None,
        })
    }
}
