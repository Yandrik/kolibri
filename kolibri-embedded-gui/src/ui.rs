use crate::framebuf::WidgetFramebuf;
use crate::spacer::Spacer;
use crate::style::Style;
use alloc::rc::Rc;
use core::cell::{RefCell, RefMut};
use core::cmp::max;
use core::fmt::Debug;
use core::ops::{Add, AddAssign, Sub};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::OriginDimensions;
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::{Dimensions, Point, Primitive, Size};
use embedded_graphics::primitives::{
    PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StyledDrawable,
};
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::Drawable;
use embedded_graphics_framebuf::FrameBuf;
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

pub struct Painter<'a, COL, DRAW, DefaultCharstyle>
where
    COL: PixelColor,
    DRAW: DrawTarget<Color = COL>,
    DefaultCharstyle: TextRenderer<Color = COL>,
{
    drawable: &'a mut DRAW,

    full_bounds: Rectangle,

    cleared: bool,

    /// Framebuffer for drawing widgets into (if available)
    framebuf: RefCell<Option<&'a mut [COL]>>,

    cur_fb: Option<WidgetFramebuf<'a, COL>>,

    style: Style<COL, DefaultCharstyle>,
}

impl<'a, COL: PixelColor, DRAW: DrawTarget<Color = COL>, CST: TextRenderer<Color = COL>>
    Painter<'a, COL, DRAW, CST>
{
    pub fn new(drawable: &'a mut DRAW, style: Style<COL, CST>, bounds: Rectangle) -> Self {
        Self {
            drawable,
            framebuf: RefCell::new(None),
            cur_fb: None,
            style,
            full_bounds: bounds,
            cleared: false,
        }
    }

    pub fn with_framebuf(mut self, framebuf: &'a mut [COL]) -> Self {
        self.framebuf = RefCell::from(Some(framebuf));
        self
    }

    /// Start drawing a new widget, creating a new framebuffer if possible.
    ///
    /// Returns true if a framebuffer was created, and false otherwise.
    /// If `true` is returned, the widget should draw into the framebuffer
    /// (and therefore clear its own background).
    pub fn alloc_framebuf(&'a mut self, area: &Rectangle) -> bool {
        if self.framebuf.borrow().is_some() {
            self.cur_fb = Some(WidgetFramebuf::new(
                &mut *self.framebuf.borrow_mut().as_mut().unwrap(),
                area.size,
                area.top_left,
            ));
            true
        } else {
            false
        }
    }

    pub fn draw(&mut self, drawable: &impl Drawable<Color = COL>) -> GuiResult<()> {
        if let Some(fb) = &mut self.cur_fb {
            drawable.draw(fb).ok();
        } else {
            drawable
                .draw(self.drawable)
                .map_err(|_| GuiError::DrawError(Some("Painter couldn't draw to drawable")))?;
        }
        Ok(())
    }

    pub fn clear(&mut self) -> GuiResult<()> {
        self.cleared = true;
        self.clear_area(self.full_bounds)
    }

    pub fn cleared(&self) -> bool {
        self.cleared
    }

    pub fn clear_area(&mut self, area: Rectangle) -> GuiResult<()> {
        let styled = area.into_styled(PrimitiveStyle::with_fill(self.style.background_color));
        self.draw(&styled)
    }

    pub fn finalize(&mut self) -> Result<(), DRAW::Error> {
        if let Some(buf) = self.cur_fb.take() {
            buf.draw(self.drawable)
        } else {
            Ok(())
        }
    }

    pub fn style(&self) -> &Style<COL, CST> {
        &self.style
    }

    pub fn style_mut(&mut self) -> &mut Style<COL, CST> {
        &mut self.style
    }
}

pub struct Ui<'a, DRAW, COL, DefaultCharstyle>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
    DefaultCharstyle: TextRenderer<Color = COL> + Clone,
{
    next_auto_id_source: u16,
    placer: Placer,
    painter: Painter<'a, COL, DRAW, DefaultCharstyle>,
    interact: Interaction,
    /// Whether the UI was background-cleared this frame
    cleared: bool,
}

impl<'a, COL, DefaultCharstyle, DRAW> Ui<'a, DRAW, COL, DefaultCharstyle>
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
        let padded_bounds = Rectangle::new(
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
            bounds: padded_bounds.size,
            wrap: true,
        };

        Ui {
            next_auto_id_source: 0,
            placer,
            painter: Painter::new(drawable, style, bounds),
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

    /// Add a Framebuffer Array to the UI.
    ///
    /// This array is used to draw widgets into, if it's available and large enough,
    /// and the widget supports it. (Widgets can get a framebuffer using `ui.draw_framebuf()`)
    /// Generally, it should have as many spaces as the biggest widget you want to draw.
    /// All widgets that are too small to fit into the array will be drawn directly into the drawable.
    ///
    ///
    pub fn with_framebuf(&mut self, framebuf: &'a mut [COL]) -> &mut Self {
        self.painter = self.painter.with_framebuf(framebuf);
        self
    }

    pub fn clear_background(&mut self) -> GuiResult<()> {
        self.painter.clear()
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
        &self.painter.style()
    }

    pub fn new_row(&mut self, height: u32) {
        self.placer.new_row(height);
    }

    /// Increase the height of the current row to the given height, if it is
    /// larger than the current height
    pub fn expand_row_height(&mut self, height: u32) {
        self.placer.col_height = height;
    }

    pub fn painter(&mut self) -> &mut Painter<'a, COL, DRAW, DefaultCharstyle> {
        &mut self.painter
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
            rect.top_left.add_assign(self.placer.pos);
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
            rect.top_left.add_assign(self.placer.pos);
            rect
        })?;

        Ok(InternalResponse {
            area,
            interaction: Interaction::None,
        })
    }
}
