use crate::framebuf::WidgetFramebuf;
use crate::style::Style;
use core::cell::UnsafeCell;
use core::cmp::{max, min};
use core::fmt::Debug;
use core::ops::{Add, AddAssign, Sub};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StyledDrawable,
};
use embedded_graphics::{Drawable, Pixel};

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

    /// The requested operation would cause the bounds to be different from the expected size
    BoundsError,
}

impl GuiError {
    pub fn draw_error(msg: &'static str) -> Self {
        GuiError::DrawError(Some(msg))
    }
}

pub type GuiResult<T> = Result<T, GuiError>;

pub struct InternalResponse {
    pub area: Rectangle,
    pub interaction: Interaction,
}

impl InternalResponse {
    pub fn new(area: Rectangle, interaction: Interaction) -> Self {
        Self { area, interaction }
    }

    pub fn empty() -> Self {
        Self {
            area: Rectangle::new(Point::zero(), Size::zero()),
            interaction: Interaction::None,
        }
    }
}

/// Response for UI interaction / space allocation and such
pub struct Response {
    pub internal: InternalResponse,
    /// Whether the widget was clicked (as in successfully interacted with)
    pub click: bool,

    /// Whether the widget is in a "down" state (e.g. a button is pressed, but not yet released)
    ///
    /// Can be used to do things while a button is held down
    pub down: bool,

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

    /// Whether the widget had an error while drawing
    pub error: Option<GuiError>,
}

// builder pattern
impl Response {
    pub fn new(raw: InternalResponse) -> Response {
        Response {
            internal: raw,
            click: false,
            redraw: true,
            changed: false,
            down: false,
            error: None,
        }
    }

    pub fn from_error(error: GuiError) -> Response {
        Response::new(InternalResponse::empty()).set_error(error)
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

    pub fn set_error(mut self, error: GuiError) -> Self {
        self.error = Some(error);
        self
    }

    pub fn set_down(mut self, down: bool) -> Self {
        self.down = down;
        self
    }

    /// Check whether the widget was clicked (as in successfully interacted with)
    pub fn clicked(&self) -> bool {
        self.click
    }

    /// Check whether the widget is in a "down" state (e.g. a button is pressed, but not yet released)
    ///
    /// Can be used to do things while a button is held down
    pub fn down(&self) -> bool {
        self.down
    }

    /// Check whether the widget was redrawn this frame
    pub fn redrawn(&self) -> bool {
        self.redraw
    }

    /// Check whether the underlying data changed (e.g. slider was moved)
    pub fn changed(&self) -> bool {
        self.changed
    }

    /// Check whether the widget had an error while drawing
    /// (e.g. the underlying draw target returned an error), no space was left, ...
    pub fn error(&self) -> Option<GuiError> {
        self.error
    }
}

pub trait Widget {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response>;
}

#[derive(Clone, Copy, Debug)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy, Debug)]
pub struct Align(pub HorizontalAlign, pub VerticalAlign);

impl Default for Align {
    fn default() -> Self {
        Align(HorizontalAlign::Left, VerticalAlign::Top)
    }
}

#[derive(Clone, Debug)]
struct Placer {
    row: u32,
    col: u32,
    pos: Point,
    row_height: u32,
    bounds: Size,
    wrap: bool,
    #[allow(unused)] // TODO: use in the future
    align: Align,
}

impl Placer {
    pub fn new(bounds: Size, wrap: bool, align: Align) -> Self {
        Placer {
            row: 0,
            col: 0,
            pos: Point::zero(),
            row_height: 0,
            bounds,
            wrap,
            align,
        }
    }

    #[allow(unused)] // TODO: use in the future
    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }

    #[allow(unused)] // TODO: use in the future
    pub fn set_align(&mut self, align: Align) {
        self.align = align;
    }

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

        // set bounds (temporary) TODO: do this PROPERLY!
        if let Align(HorizontalAlign::Center, _) = self.align {
            if self.pos.x as u32 + size.width > self.bounds.width {
                return Err(GuiError::NoSpaceLeft);
            }
            // Calculate the right x-coordinate to center the widget between self.pos.x and self.bounds.width
            // (self.bounds.width + self.pos.x as u32 - size.width) / 2
            self.pos.x = ((self.bounds.width + self.pos.x as u32 - size.width) / 2) as i32;
        };
        let right = size.width + self.pos.x as u32;
        let mut bottom = max(self.row_height, size.height) + self.pos.y as u32;
        if !self.check_bounds(Size::new(right, bottom)) {
            if self.wrap {
                bottom = self.pos.y as u32 + max(self.row_height, size.height);
                // check that wrap fits
                if !self.check_bounds(Size::new(0, bottom)) {
                    return Err(GuiError::NoSpaceLeft);
                }

                // perform wrap
                self.new_row(size.height); // TODO: better / proper wrap impl
            } else {
                return Err(GuiError::NoSpaceLeft);
            }
        }

        // set new col height (expand if necessary)
        self.row_height = max(self.row_height, size.height);

        // set new position
        let item_pos = self.pos;
        self.pos = Point::new(right as i32, self.pos.y);

        Ok(Rectangle::new(
            item_pos,
            Size::new(size.width, self.row_height),
        ))
    }

    #[allow(unused)]
    fn row_size(&self) -> Size {
        Size::new(self.bounds.width, self.row_height)
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
        self.pos = Point::new(0, self.pos.y + self.row_height as i32);
        self.row_height = height;
    }

    fn row_height(&self) -> u32 {
        self.row_height
    }

    fn expand_row_height(&mut self, height: u32) {
        self.row_height = max(self.row_height, height);
    }

    /// Check whether a size is in bounds of the widget (<= widget_size)
    fn check_bounds(&self, pos: Size) -> bool {
        pos.width as u32 <= self.bounds.width && pos.height <= self.bounds.height
    }
}

struct Painter<'a, COL: PixelColor, DRAW: DrawTarget<Color = COL>> {
    target: &'a mut DRAW,
    buffer_raw: Option<UnsafeCell<&'a mut [COL]>>,
    framebuf: Option<WidgetFramebuf<'a, COL>>,
}

impl<'a, COL: PixelColor, DRAW: DrawTarget<Color = COL>> Painter<'a, COL, DRAW> {
    fn new(target: &'a mut DRAW) -> Self {
        Self {
            target,
            buffer_raw: None,
            framebuf: None,
        }
    }

    fn set_buffer(&mut self, buffer: &'a mut [COL]) {
        self.buffer_raw = Some(UnsafeCell::new(buffer));
    }

    fn start_drawing(&mut self, area: &Rectangle) {
        if let Some(_) = self.framebuf {
            panic!("Framebuffer is already in use!");
        }

        if let Some(buf) = &mut self.buffer_raw {
            let buf = WidgetFramebuf::try_new(unsafe { *buf.get() }, area.size, area.top_left);
            if let Some(framebuf) = buf {
                self.framebuf = Some(framebuf);
            }
        }
    }

    /// Clear the buffer, if it's available.
    ///
    /// ## Returns
    ///
    /// `true` if the buffer was cleared, `false` if there's no buffer to clear.
    fn clear_buffer(&mut self, color: COL) -> bool {
        if let Some(framebuf) = &mut self.framebuf {
            framebuf.clear(color)
                .ok()  /* cannot fail */;
            true
        } else {
            false
        }
    }

    fn finalize(&mut self) -> GuiResult<()> {
        if let Some(buf) = &mut self.framebuf {
            buf.draw(self.target)
                .map_err(|_| GuiError::draw_error("Failed to draw framebuf"))?;
            self.framebuf = None;
        }
        Ok(())
    }

    fn draw(&mut self, item: &impl Drawable<Color = COL>) -> GuiResult<()> {
        if let Some(buffer) = &mut self.framebuf {
            item.draw(buffer)
                .ok() /* cannot fail */;
        } else {
            item.draw(self.target)
                .map_err(|_| GuiError::draw_error("Failed to draw item"))?;
        }
        Ok(())
    }

    fn with_subpainter<'b, F>(&'b mut self, f: F) -> GuiResult<()>
    where
        F: FnOnce(Painter<'b, COL, DRAW>) -> GuiResult<()>,
    {
        let target: &'b mut DRAW = self.target;
        let mut subpainter = Painter::new(target);

        if matches!(self.framebuf, Some(_)) {
            panic!("Cannot create subpainter when framebuf is in use!");
        }

        if let Some(buf) = &mut self.buffer_raw {
            subpainter.set_buffer(unsafe { *buf.get() });
        }
        (f)(subpainter)?;
        Ok(())
    }
}

// Basic Implementations for DrawTarget and Dimensions to allow Painter to be used as its inner DrawTarget
impl<COL: PixelColor, DRAW: DrawTarget<Color = COL, Error = ERR>, ERR> Dimensions
    for Painter<'_, COL, DRAW>
{
    fn bounding_box(&self) -> Rectangle {
        self.target.bounding_box()
    }
}

impl<COL: PixelColor, DRAW: DrawTarget<Color = COL, Error = ERR>, ERR> DrawTarget
    for Painter<'_, COL, DRAW>
{
    type Color = COL;
    type Error = ERR;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.target.draw_iter(pixels)
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

impl Default for Interaction {
    fn default() -> Self {
        Interaction::None
    }
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

pub struct Ui<'a, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    bounds: Rectangle,
    painter: Painter<'a, COL, DRAW>,
    style: Style<COL>,
    placer: Placer,
    interact: Interaction,
    /// Whether the UI was background-cleared this frame
    cleared: bool,
    debug_color: Option<COL>,
}

// getters for Ui things
impl<'a, DRAW, COL> Ui<'a, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    /// Get the width of the UI's placer. Note that this **isn't the entire screen width**.
    /// To get the screen width, use `get_screen_width()`.
    pub fn get_width(&self) -> u32 {
        self.placer.bounds.width
    }

    /// Get the width of the screen
    pub fn get_screen_width(&self) -> u32 {
        self.bounds.size.width + self.style.spacing.window_border_padding.width * 2
    }
}

impl<'a, COL, DRAW> Ui<'a, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    pub fn new(drawable: &'a mut DRAW, bounds: Rectangle, style: Style<COL>) -> Self {
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

        let placer = Placer::new(
            bounds.size,
            true,
            Align(HorizontalAlign::Left, VerticalAlign::Top),
        );

        Self {
            bounds,
            painter: Painter::new(drawable),
            style,
            placer,
            interact: Interaction::None,
            cleared: false,
            debug_color: None,
        }
    }

    pub fn new_fullscreen(drawable: &'a mut DRAW, style: Style<COL>) -> Self {
        let bounds = drawable.bounding_box();
        Ui::new(drawable, bounds, style)
    }

    pub fn interact(&mut self, interaction: Interaction) {
        self.interact = interaction;
    }

    pub fn add_and_clear_col_remainder(&mut self, widget: impl Widget, clear: bool) -> Response {
        let resp = self.add_raw(widget).unwrap_or_else(|e| {
            // panic!("Failed to add widget to UI: {:?}", e);
            Response::from_error(e)
        });
        if clear {
            self.clear_row_to_end().ok();
        }

        self.new_row();

        resp
    }

    pub fn add(&mut self, widget: impl Widget) -> Response {
        // draw widget. TODO: Add new auto ID
        let resp = self.add_raw(widget).unwrap_or_else(|e| {
            // panic!("Failed to add widget to UI: {:?}", e);
            Response::from_error(e)
        });

        // create new row
        self.new_row();

        resp
    }

    pub fn add_centered(&mut self, widget: impl Widget) -> Response {
        // draw widget. TODO: Add new auto ID
        let align = self.placer.align;
        self.placer.align = Align(HorizontalAlign::Center, align.1);
        let resp = self.add_raw(widget).unwrap_or_else(|e| {
            // panic!("Failed to add widget to UI: {:?}", e);
            Response::from_error(e)
        });
        self.placer.align = align;

        self.new_row();
        resp
    }

    /// Add a widget horizontally to the layout to the current row
    pub fn add_horizontal(&mut self, widget: impl Widget) -> Response {
        // add widget (auto-expands row height potentially
        let resp = self.add_raw(widget).unwrap_or_else(|e| {
            // panic!("Failed to add widget to UI: {:?}", e);
            Response::from_error(e)
        });
        // ignore space alignment errors (those are "fine". If wrapping is enabled,
        // the next widget will be placed on the next row, without any space in between.)
        self.allocate_space_no_wrap(self.style().spacing.item_spacing)
            .ok();

        resp
    }

    pub fn add_raw(&mut self, mut widget: impl Widget) -> GuiResult<Response> {
        let res = widget.draw(self);
        if let (Ok(res), Some(debug_color)) = (&res, self.debug_color) {
            res.internal
                .area
                .draw_styled(
                    &PrimitiveStyleBuilder::new()
                        .stroke_color(debug_color)
                        .stroke_width(1)
                        .build(),
                    &mut self.painter,
                )
                .ok();
        }
        res
    }

    pub fn style(&self) -> &Style<COL> {
        &self.style
    }

    pub fn style_mut(&mut self) -> &mut Style<COL> {
        &mut self.style
    }

    pub fn new_row(&mut self) {
        self.new_row_raw(self.style().spacing.item_spacing.height);

        self.new_row_raw(self.style().default_widget_height);
    }

    pub fn new_row_raw(&mut self, height: u32) {
        self.placer.new_row(height);
    }

    /// Increase the height of the current row to the given height, if it is
    /// larger than the current height
    pub fn expand_row_height(&mut self, height: u32) {
        self.placer.expand_row_height(height);
    }

    pub fn draw_raw<OUT>(
        &mut self,
        to_draw: &impl Drawable<Color = COL, Output = OUT>,
    ) -> Result<OUT, DRAW::Error> {
        to_draw.draw(self.painter.target)
    }

    // painter functions

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

        let inter = self.check_interact(area);

        Ok(InternalResponse {
            area,
            interaction: inter,
        })
    }

    pub fn get_row_height(&self) -> u32 {
        self.placer.row_height()
    }
}

// Clearing impls
impl<'a, COL, DRAW> Ui<'a, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    /// Return whether the UI was background-cleared this frame
    pub fn cleared(&self) -> bool {
        self.cleared
    }

    pub fn clear_area(&mut self, area: Rectangle) -> GuiResult<()> {
        self.draw(&area.into_styled(PrimitiveStyle::with_fill(self.style.background_color)))
            .map_err(|_| GuiError::DrawError(Some("Couldn't clear area")))
    }

    /// Clear the current row with the background color
    pub fn clear_row(&mut self) -> GuiResult<()> {
        let row_height = self.placer.row_height();
        let row_rect = Rectangle::new(
            Point::new(0, self.placer.pos.y),
            Size::new(self.placer.bounds.width, row_height),
        );
        self.clear_area(row_rect)
    }

    /// Clear the row to the end of the screen. This is useful for clearing the rendering
    /// remains of partially drawn widgets and such (e.g. clearing after a label's width went down)
    ///
    /// As this is fairly expensive, it should only be used when necessary.
    ///
    /// Sidenote: This clears the entire row to the end, taking the row_height into account.
    /// What that means is that - in general - you shoud use this **after** adding a widget,
    /// as the row height will be increased to the widget's height.
    pub fn clear_row_to_end(&mut self) -> GuiResult<()> {
        let col_height = self.placer.row_height;
        let col_rect = Rectangle::new(
            // clear right to widget bounds
            Point::new(
                self.placer.pos.x + self.style.spacing.window_border_padding.width as i32,
                self.placer.pos.y,
            ),
            Size::new(
                (self.placer.bounds.width as i32 - self.placer.pos.x).max(0) as u32,
                col_height,
            ),
        );
        self.clear_area(col_rect)
    }

    /// Clear the screen down to the bottom. This is useful for clearing the rendering
    /// remains of partially drawn widgets and such (e.g. clearing after a label's width went down),
    /// especially for multiple rows at the same time.
    ///
    /// As this is fairly expensive, it should only be used when necessary.
    ///
    /// Note that this clears **the entire screen** down from the current placer position,
    /// so call this at the start of a new row *before drawing on it* if you want to draw on the
    /// cleared area, otherwise it will erase any widgets you already drew.
    pub fn clear_to_bottom(&mut self) -> GuiResult<()> {
        self.clear_area(Rectangle::new(
            Point::new(0, self.placer.pos.y),
            Size::new(
                self.placer.bounds.width,
                self.placer.bounds.height - self.placer.pos.y as u32,
            ),
        ))
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
                self.painter.target,
            )
            .map_err(|_| GuiError::DrawError(Some("Couldn't clear GUI Background")))
    }
}

// Drawing Impl
impl<'a, COL, DRAW> Ui<'a, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    pub fn set_buffer(&mut self, buffer: &'a mut [COL]) {
        self.painter.set_buffer(buffer);
    }

    pub fn start_drawing(&mut self, area: &Rectangle) {
        self.painter.start_drawing(area);
        self.painter.clear_buffer(self.style.background_color);
    }

    pub fn clear_buffer_raw(&mut self, color: COL) -> bool {
        self.painter.clear_buffer(color)
    }

    pub fn finalize(&mut self) -> GuiResult<()> {
        self.painter.finalize()
    }

    pub fn draw(&mut self, item: &impl Drawable<Color = COL>) -> GuiResult<()> {
        self.painter.draw(item)
    }
}

// SubUI impl

impl<'a, COL, DRAW> Ui<'a, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    /// Create a sub-UI with the given bounds, where you can modify all values. This is useful for
    /// creating a sub-UI with a different style, or drawing to a screen area outside (or on top)
    /// of the normal UI.
    pub fn unchecked_sub_ui<F>(&mut self, bounds: Rectangle, f: F) -> GuiResult<()>
    where
        F: FnOnce(&mut Ui<DRAW, COL>) -> GuiResult<()>,
    {
        let bounds = Rectangle::new(
            bounds.top_left.add(Point::new(
                self.style.spacing.window_border_padding.height as i32,
                self.style.spacing.window_border_padding.width as i32,
            )),
            bounds
                .size
                .saturating_sub(self.style.spacing.window_border_padding * 2),
        );

        // set up placer
        let placer = Placer::new(
            bounds.size,
            true,
            Align(HorizontalAlign::Left, VerticalAlign::Top),
        );

        self.painter.with_subpainter(|painter| {
            let mut sub_ui = Ui {
                painter,
                bounds,
                style: self.style.clone(),
                interact: self.interact,
                placer,
                cleared: false,
                debug_color: self.debug_color,
            };
            (f)(&mut sub_ui)
        })?;

        Ok(())
    }

    pub fn sub_ui<F>(&mut self, f: F) -> GuiResult<()>
    where
        F: FnOnce(&mut Ui<DRAW, COL>) -> GuiResult<()>,
    {
        self.painter.with_subpainter(|painter| {
            let mut sub_ui = Ui {
                painter,
                bounds: self.bounds,
                style: self.style.clone(),
                interact: self.interact,
                placer: self.placer.clone(),
                cleared: false,
                debug_color: self.debug_color,
            };
            let res = (f)(&mut sub_ui);

            self.placer = sub_ui.placer;

            res
        })?;

        Ok(())
    }

    pub fn right_panel_ui<F>(&mut self, width: u32, allow_smaller: bool, f: F) -> GuiResult<()>
    where
        F: FnOnce(&mut Ui<DRAW, COL>) -> GuiResult<()>,
    {
        // check bounds and remaining space of placer
        let bounds = self.placer.bounds;

        let y = self.placer.pos.y as u32;

        let max_width = bounds.width - self.placer.pos.x as u32;
        let max_height = bounds.height - y;

        if width > max_width && !allow_smaller {
            return Err(GuiError::BoundsError);
        }

        self.placer.bounds.width -= min(width, max_width);

        let area = Rectangle::new(
            Point::new((bounds.width - min(width, max_width)) as i32, y as i32),
            Size::new(
                bounds.width - (bounds.width - min(width, max_width)),
                max_height,
            ),
        );

        self.unchecked_sub_ui(area, f)
    }

    /// Create a sub-UI with the given bounds in the center of the screen.
    /// This is very useful to draw OVER other UI elements. In other words:
    /// When using this, make sure that you don't update the UI behind it if your display allows it,
    /// or you will get flickering.
    pub fn central_centered_panel_ui<F>(&mut self, width: u32, height: u32, f: F) -> GuiResult<()>
    where
        F: FnOnce(&mut Ui<DRAW, COL>) -> GuiResult<()>,
    {
        let bounds = self.placer.bounds;

        let max_width = bounds.width;
        let max_height = bounds.height;

        if width > max_width {
            return Err(GuiError::BoundsError);
        }

        if height > max_height {
            return Err(GuiError::BoundsError);
        }

        self.placer.bounds.width -= min(width, max_width);
        self.placer.bounds.height -= min(height, max_height);

        let area = Rectangle::new(
            Point::new(
                ((bounds.width - width) / 2) as i32,
                ((bounds.height - height) / 2) as i32,
            ),
            Size::new(width, height),
        );

        self.unchecked_sub_ui(area, f)
    }
}

// debug drawing impl

impl<'a, COL, DRAW> Ui<'a, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    pub fn draw_bounds_debug(&mut self, color: COL) -> GuiResult<()> {
        let bounds = self.bounds;
        bounds
            .draw_styled(
                &PrimitiveStyleBuilder::new()
                    .stroke_color(color)
                    .stroke_width(1)
                    .build(),
                &mut self.painter,
            )
            .map_err(|_| GuiError::DrawError(Some("Couldn't draw bounds")))
    }

    pub fn draw_widget_bounds_debug(&mut self, color: COL) {
        self.debug_color = Some(color);
    }
}
