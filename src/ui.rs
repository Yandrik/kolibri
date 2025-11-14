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
/// Struct for managing placing of widgets in the [Ui]
///
/// Placement is deterministic (meaning widgets placed with the same settings in the same order will always be placed at the same position)
///
/// ## Placement Rules
///
/// - Widgets are placed in rows, from left to right, from top to bottom (for now)
/// - Placement is deterministic and repeatable
/// - Placement cannot happen outside of the bounds of the placer
struct Placer {
    /// Current row
    row: u32,
    /// Current column
    col: u32,
    /// Position of the top left corner of the placer
    pos: Point,
    /// Height of the current row
    row_height: u32,
    /// Bounds of the placer
    bounds: Size,
    /// Whether to wrap to the next row if the widget doesn't fit
    wrap: bool,
    #[allow(unused)] // TODO: use in the future
    align: Align,
}

impl Placer {
    /// Create a new placer with the given bounds, wrapping and alignment
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

    /// **STUB / NOT YET USEFUL**: Set the wrap setting of the placer
    #[allow(unused)] // TODO: use in the future
    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }

    /// **STUB / NOT YET USEFUL**: Set the alignment of the placer
    #[allow(unused)] // TODO: use in the future
    pub fn set_align(&mut self, align: Align) {
        self.align = align;
    }

    /// Allocate the next widget with the given `size`, explicitly disabling wrapping for this operation
    ///
    /// ## Returns
    ///
    /// Returns the allocated rectangle, or an error if the widget doesn't fit
    fn next_no_wrap(&mut self, size: Size) -> GuiResult<Rectangle> {
        let wrap = self.wrap;
        self.wrap = false;
        let res = self.next(size);
        self.wrap = wrap;
        res
    }

    /// Allocate the next widget with the given `size`.
    ///
    /// ## Returns
    ///
    /// Returns the allocated rectangle, or an error if the widget doesn't fit
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
        self.col += 1;

        Ok(Rectangle::new(
            item_pos,
            Size::new(size.width, self.row_height),
        ))
    }

    #[allow(unused)]
    /// Returns the full size of the current row, which is the full width of the bounds and the current row height.
    fn row_size(&self) -> Size {
        Size::new(self.bounds.width, self.row_height)
    }

    /// Returns the remaining available space within the bounds of the [Placer] for placing widgets.
    /// The remaining space hearby is a rectangle from the current plcer X and Y position to the bottom right corner of the bounds.
    fn space_available(&self) -> Size {
        Size::new(
            self.bounds.width - self.pos.x as u32,
            self.bounds.height - self.pos.y as u32,
        )
    }

    /// Advances to the next row, setting the initial row height to the provided `height` parameter.
    fn new_row(&mut self, height: u32) {
        self.row += 1;
        self.col = 0;
        self.pos = Point::new(0, self.pos.y + self.row_height as i32);
        self.row_height = height;
    }

    /// Returns the current row height.
    fn row_height(&self) -> u32 {
        self.row_height
    }

    /// Expands the current row height to the maximum of the current height and the provided `height` parameter.
    fn expand_row_height(&mut self, height: u32) {
        self.row_height = max(self.row_height, height);
    }

    /// Check whether a size is in bounds of the widget (<= widget_size)
    fn check_bounds(&self, pos: Size) -> bool {
        pos.width <= self.bounds.width && pos.height <= self.bounds.height
    }
}

/// Struct that manages drawing to a [DrawTarget], with optional [WidgetFramebuf] for more efficient drawing.
///
///
/// It provides methods for setting the buffer, starting and finalizing the drawing process, and clearing the buffer.
struct Painter<'a, COL: PixelColor, DRAW: DrawTarget<Color = COL>> {
    target: &'a mut DRAW,
    buffer_raw: Option<UnsafeCell<&'a mut [COL]>>,
    framebuf: Option<WidgetFramebuf<'a, COL>>,
}

impl<'a, COL: PixelColor, DRAW: DrawTarget<Color = COL>> Painter<'a, COL, DRAW> {
    /// Creates a new [Painter] instance based on the provided [DrawTarget].
    fn new(target: &'a mut DRAW) -> Self {
        Self {
            target,
            buffer_raw: None,
            framebuf: None,
        }
    }

    /// Sets the internal buffer used for drawing operations.
    ///
    /// This method allows the caller to provide a mutable slice of the pixel color type `COL` that the `Painter` will use
    /// as its internal buffer. This can be used to optimize drawing by avoiding unnecessary memory allocations.
    ///
    /// This buffer is entirely optional, but can increase drawing performance significantly, especially for layered widgets such as
    /// [crate::button::Button] or [crate::slider::Slider]
    fn set_buffer(&mut self, buffer: &'a mut [COL]) {
        self.buffer_raw = Some(UnsafeCell::new(buffer));
    }

    /// Begin the drawing process in the given area.
    ///
    /// If a framebuffer is provided and of sufficient size, it gets used for subsequent drawing operations.
    /// If not, the drawing operations are performed directly on the [DrawTarget].
    fn start_drawing(&mut self, area: &Rectangle) {
        if self.framebuf.is_some() {
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

    /// Finalize the drawing process.
    ///
    /// This flushes the framebuffer to the draw target, if it was used in this drawing process.
    ///
    /// If a framebuffer is provided and of sufficient size, it gets used for subsequent drawing operations.
    /// If not, the drawing operations are performed directly on the [DrawTarget].
    fn finalize(&mut self) -> GuiResult<()> {
        if let Some(buf) = &mut self.framebuf {
            buf.draw(self.target)
                .map_err(|_| GuiError::draw_error("Failed to draw framebuf"))?;
            self.framebuf = None;
        }
        Ok(())
    }

    /// Draws the given [Drawable] to the [DrawTarget].
    ///
    /// If a framebuffer is available, the item is drawn to the framebuffer, and flushed to the target when [Painter::finalize()] is called.
    /// Otherwise, it is drawn directly to the target.
    ///
    /// ## Returns
    ///
    /// Returns a `GuiResult` indicating whether the drawing was successful.
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

    /// Creates a `Subpainter`, a new [Painter] instance, executes the provided closure with the sub-painter, and returns the result.
    ///
    /// This method is useful for creating a temporary [Painter] instance that can be modified for a subset of drawing operations on the main [DrawTarget].
    /// The sub-painter's [DrawTarget] is a reference to the main [DrawTarget], so any drawing operations performed on the sub-painter will
    /// be reflected in the main [DrawTarget].
    ///
    /// If the main [Painter] instance has a raw buffer set, the sub-painter will inherit that buffer.
    ///
    /// ## Panics
    ///
    /// Panics if the main [Painter] instance is currently using its framebuffer, as sub-painters cannot be created when the framebuffer is in use.
    /// Make sure to call [Painter::finalize()] before creating a sub-painter to prevent this.
    fn with_subpainter<'b, F>(&'b mut self, f: F) -> GuiResult<()>
    where
        F: FnOnce(Painter<'b, COL, DRAW>) -> GuiResult<()>,
    {
        let target: &'b mut DRAW = self.target;
        let mut subpainter = Painter::new(target);

        if self.framebuf.is_some() {
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

    // TODO: optimize by implementing the other methods too

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.target.draw_iter(pixels)
    }
}

/// Interaction with the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
    #[default]
    None,
}

impl Interaction {
    /// Gets the point associated with the current interaction, if any.
    ///
    /// This method returns the point associated with the current interaction, such as the click, drag, release, or hover point. If the interaction is [Interaction::None], this method returns [None`.
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

#[derive(Debug, Default, PartialEq)]
enum PopupStage {
    #[default]
    Hide,
    Drawing,
    Show,
    Handled,
}

/// Represents the state of a popup widget.
///
/// The [PopupState] struct tracks the current stage of the popup widget, as well as its position and bounds.
///
/// # Fields
///
/// * `stage` - The current stage of the popup widget (hide, drawing, show)
/// * `col` - The column index of the popup widget in the grid layout
/// * `row` - The row index of the popup widget in the grid layout
/// * `bounds` - The optional bounds of the popup widget, if it has been positioned
///
/// # Notice
///
/// - The [PopupState] parameter must be declared outside the UI loop to ensure that the popup widget's state is not lost when redraw the main UI.
#[derive(Debug, Default)]
pub struct PopupState {
    stage: PopupStage,
    col: u32,
    row: u32,
    bounds: Option<Rectangle>,
    offset_y: i32,
}

/// Struct that manages the state and interaction of a popup widget.
///
/// The [Popup] struct is responsible for tracking the state of a popup widget, such as its position, bounds, and interaction status.
/// It is used internally by the framework to manage popup widgets and handle user interactions.
///
/// # Fields
///
/// * `state` - A mutable reference to the [PopupState] struct that tracks the popup widget's state
/// * `buffer` - A mutable reference to the buffer used for rendering the popup widget
/// * `interact` - The current interaction state of the popup widget (click, drag, release, hover, none)
pub(crate) struct Popup<'a, COL: PixelColor> {
    state: &'a mut PopupState,
    buffer: &'a mut [COL],
    interact: Interaction,
}

impl<'a, COL: PixelColor> Popup<'a, COL> {
    pub fn new(state: &'a mut PopupState, buffer: &'a mut [COL]) -> Self {
        Self {
            state,
            buffer,
            interact: Interaction::None,
        }
    }
}

/// The main UI struct, responsible for managing the layout and rendering of the user interface.
///
/// The [Ui] struct is the core of the Kolibri GUI framework. It manages the following:
/// - A [Painter] for rendering widgets to a [DrawTarget]
/// - A [Placer] for managing widget layout and positioning
/// - A [Style] for configuring the appearance of widgets
/// - Interaction state tracking (clicks, drags, hovers, etc.)
/// - Debug settings (e.g., debug color for visualizing widget bounds)
///
/// # Example
///
/// ```no_run
/// use embedded_graphics::geometry::Size;
/// use embedded_graphics::pixelcolor::Rgb565;
/// use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
/// use kolibri_embedded_gui::style::medsize_rgb565_style;
/// use kolibri_embedded_gui::ui::Ui;
///
/// let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
/// let output_settings = OutputSettingsBuilder::new().build();
/// let mut window = Window::new("Kolibri Example", &output_settings);
///
/// let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
/// ui.clear_background().unwrap();
/// // ... add widgets etc.
/// ```
///
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
    popup: Option<Popup<'a, COL>>,
}

// -- Getter methods for [Ui] --
impl<DRAW, COL> Ui<'_, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    /// Returns the width of the [Ui]'s placer.
    ///
    /// Note that this is not the entire screen width.
    ///
    /// ## Returns
    ///
    /// The width of the placer as a `u32`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let width = ui.get_width();
    /// println!("Placer width: {}", width);
    /// ```
    pub fn get_width(&self) -> u32 {
        self.placer.bounds.width
    }

    /// Returns the width of the screen.
    ///
    /// This includes the UI's window border padding.
    ///
    /// ## Returns
    ///
    /// The screen width as a `u32`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::geometry::Size;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// let screen_width = ui.get_screen_width();
    /// println!("Screen width: {}", screen_width);
    /// ```
    pub fn get_screen_width(&self) -> u32 {
        self.bounds.size.width + self.style.spacing.window_border_padding.width * 2
    }

    /// Returns the height of the screen
    ///
    /// This includes the UI's window border padding.
    ///
    /// ## Returns
    ///
    /// The screen width as a `u32`.
    ///
    /// # Example
    /// ```no_run
    /// # use embedded_graphics::geometry::Size;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// let screen_height = ui.get_screen_height();
    /// println!("Screen height: {}", screen_height);
    /// ```
    pub fn get_screen_height(&self) -> u32 {
        self.bounds.size.height + self.style.spacing.window_border_padding.height * 2
    }

    /// Return the position of the placer.
    /// ## Returns
    ///
    /// The position of the placer as a [Point].
    /// ## Example
    ///
    /// ```no_run
    /// # use embedded_graphics::geometry::Size;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// let pos = ui.get_placer_top_left();
    /// println!("Placer position: {}", pos);
    /// ```
    pub fn get_placer_top_left(&self) -> Point {
        self.placer.pos
    }
}

// -- Construction and widget addition methods --
impl<'a, COL, DRAW> Ui<'a, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    /// Creates a new [Ui] instance with the given drawable, bounds and style.
    ///
    /// The provided bounds are adjusted by the style's window border padding.
    ///
    /// ## Returns
    ///
    /// A new instance of [Ui].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::geometry::Size;
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// use embedded_graphics::geometry::Dimensions;
    /// let bounds = display.bounding_box();
    /// let ui = Ui::new(&mut display, bounds, medsize_rgb565_style());
    /// ```
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
            popup: None,
        }
    }

    /// Creates a new fullscreen [Ui] instance using the entire bounding box of the drawable.
    ///
    /// This is equivalent to calling [Ui::new] with the drawable's bounding box.
    ///
    /// ## Returns
    ///
    /// A new fullscreen instance of [Ui].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// let ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// ```
    pub fn new_fullscreen(drawable: &'a mut DRAW, style: Style<COL>) -> Self {
        let bounds = drawable.bounding_box();
        Ui::new(drawable, bounds, style)
    }

    /// Sets the current interaction for the [Ui].
    ///
    /// This interaction is used to update the state of widgets.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// ui.interact(Interaction::Click(Point::new(10, 10)));
    /// ```
    pub fn interact(&mut self, interaction: Interaction) {
        self.interact = interaction;
    }

    /// Adds a widget to the [Ui] and, if requested, clears the remaining horizontal space in the current row.
    ///
    /// After adding the widget, a new row is started.
    ///
    /// ## Returns
    ///
    /// A [Response] indicating the result of adding the widget.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let response = ui.add_and_clear_col_remainder(widget, true);
    /// ```
    pub fn add_and_clear_col_remainder(&mut self, widget: impl Widget, clear: bool) -> Response {
        let resp = self.add_raw(widget).unwrap_or_else(Response::from_error);
        if clear {
            self.clear_row_to_end().ok();
        }
        self.new_row();
        resp
    }

    /// Adds a widget to the [Ui] and then starts a new row.
    ///
    /// The widget is drawn and its response is returned.
    ///
    /// ## Returns
    ///
    /// A [Response] indicating the result of adding the widget.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let response = ui.add(widget);
    /// ```
    pub fn add(&mut self, widget: impl Widget) -> Response {
        let resp = self.add_raw(widget).unwrap_or_else(Response::from_error);
        self.new_row();
        resp
    }

    /// Adds a widget centered horizontally in the current row of the [Ui].
    ///
    /// After drawing the widget, the row alignment is reset.
    ///
    /// ## Returns
    ///
    /// A [Response] indicating the result of adding the widget.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let response = ui.add_centered(widget);
    /// ```
    pub fn add_centered(&mut self, widget: impl Widget) -> Response {
        let align = self.placer.align;
        self.placer.align = Align(HorizontalAlign::Center, align.1);
        let resp = self.add_raw(widget).unwrap_or_else(Response::from_error);
        self.placer.align = align;
        self.new_row();
        resp
    }

    /// Adds a widget to the current row of the [Ui] without starting a new row.
    ///
    /// Space is allocated for the next widget after this one.
    ///
    /// ## Returns
    ///
    /// A [Response] indicating the result of adding the widget.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let response = ui.add_horizontal(widget);
    /// ```
    pub fn add_horizontal(&mut self, widget: impl Widget) -> Response {
        let resp = self.add_raw(widget).unwrap_or_else(Response::from_error);
        // Allocate space between widgets; ignore space errors.
        self.allocate_space_no_wrap(self.style().spacing.item_spacing)
            .ok();
        resp
    }

    /// Draws a widget directly to the [Ui] without changing the layout.
    ///
    /// If a debug color is set, the widget's bounding area is drawn with that color.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] wrapping a [Response] for the drawn widget.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// match ui.add_raw(widget) {
    ///     Ok(response) => { /* widget drawn successfully */ },
    ///     Err(e) => { /* handle error */ },
    /// }
    /// ```
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

    /// Returns an immutable reference to the current style of the [Ui].
    ///
    /// ## Returns
    ///
    /// A reference to the [Style].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let current_style = ui.style();
    /// ```
    pub fn style(&self) -> &Style<COL> {
        &self.style
    }

    /// Returns a mutable reference to the current style of the [Ui].
    ///
    /// ## Returns
    ///
    /// A mutable reference to the [Style].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let style = ui.style_mut();
    /// style.background_color = Rgb565::BLACK;
    /// ```
    pub fn style_mut(&mut self) -> &mut Style<COL> {
        &mut self.style
    }

    /// Advances the layout to a new row in the [Ui].
    ///
    /// This method uses the default spacing and widget height from the current style.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.new_row();
    /// ```
    pub fn new_row(&mut self) {
        self.new_row_raw(self.style().spacing.item_spacing.height);
        self.new_row_raw(self.style().default_widget_height);
    }

    /// Advances the layout to a new row in the [Ui] with the specified height.
    ///
    /// ## Parameters
    ///
    /// - `height`: The height for the new row.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.new_row_raw(20);
    /// ```
    pub fn new_row_raw(&mut self, height: u32) {
        self.placer.new_row(height);
    }

    /// Expands the current row height of the [Ui] to at least the given height.
    ///
    /// If the current row height is less than `height`, it is increased.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.expand_row_height(30);
    /// ```
    pub fn expand_row_height(&mut self, height: u32) {
        self.placer.expand_row_height(height);
    }

    /// Draws a [Drawable] item directly using the [Ui]'s underlying draw target.
    ///
    /// ## Returns
    ///
    /// The output produced by drawing the item, or an error if drawing fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// # use embedded_graphics::primitives::PrimitiveStyle;
    /// # let rectangle_shape = Rectangle::new(Point::new(50, 0), Size::new(200, 150));
    /// # // Make it drawable with a filled red style
    /// # let rectangle = rectangle_shape.into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
    /// let result = ui.draw_raw(&rectangle);
    /// ```
    pub fn draw_raw<OUT>(
        &mut self,
        to_draw: &impl Drawable<Color = COL, Output = OUT>,
    ) -> Result<OUT, DRAW::Error> {
        to_draw.draw(self.painter.target)
    }

    /// Returns the remaining available space for widget placement in the [Ui].
    ///
    /// ## Returns
    ///
    /// A [Size] representing the available width and height.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let space = ui.space_available();
    /// println!("Available space: {:?}", space);
    /// ```
    pub fn space_available(&self) -> Size {
        self.placer.space_available()
    }

    /// Checks if the current interaction occurs within the specified area.
    ///
    /// ## Returns
    ///
    /// The [Interaction] if the interaction's point is within the area, otherwise [Interaction::None].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// # let some_rectangle = Rectangle::new(Point::new(50, 0), Size::new(200, 150));
    /// let interaction = ui.check_interact(some_rectangle);
    /// ```
    pub fn check_interact(&self, area: Rectangle) -> Interaction {
        if self
            .interact
            .get_point()
            .map(|pt| {
                if let Some(popup) = &self.popup {
                    if popup.state.stage == PopupStage::Show {
                        return false;
                    }
                }
                return area.contains(pt);
            })
            .unwrap_or(false)
        {
            self.interact
        } else {
            Interaction::None
        }
    }

    /// Allocates an exact space in the [Ui] for a widget of the desired size.
    ///
    /// This method currently wraps [Ui::allocate_space] without extra logic.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] containing an [InternalResponse] with the allocated area and interaction.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let allocation = ui.allocate_exact_size(Size::new(50, 30));
    /// ```
    pub fn allocate_exact_size(&mut self, desired_size: Size) -> GuiResult<InternalResponse> {
        self.allocate_space(desired_size)
    }

    /// Allocates space in the [Ui] for a widget of the desired size, with wrapping if needed.
    ///
    /// The allocated area is adjusted by the [Ui]'s bounds.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] containing an [InternalResponse] with the allocated rectangle and interaction.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let allocation = ui.allocate_space(Size::new(100, 40));
    /// ```
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

    /// Allocates space in the [Ui] for a widget of the desired size without wrapping.
    ///
    /// The allocated area is adjusted by the [Ui]'s bounds.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] containing an [InternalResponse] with the allocated rectangle and interaction.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let allocation = ui.allocate_space_no_wrap(Size::new(80, 25));
    /// ```
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

    /// Returns the current row height used in the [Ui]'s layout.
    ///
    /// ## Returns
    ///
    /// The current row height as a `u32`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let row_height = ui.get_row_height();
    /// println!("Row height: {}", row_height);
    /// ```
    pub fn get_row_height(&self) -> u32 {
        self.placer.row_height()
    }
}

// -- Clearing methods --
impl<COL, DRAW> Ui<'_, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    /// Returns whether the [Ui]'s background was cleared this frame.
    ///
    /// ## Returns
    ///
    /// `true` if the background was cleared, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// if ui.cleared() {
    ///     println!("Background cleared");
    /// }
    /// ```
    pub fn cleared(&self) -> bool {
        self.cleared
    }

    /// Clears the specified area in the [Ui] using the background color.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating success or error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.clear_area(Rectangle::new(Point::new(0,0), Size::new(100, 50))).unwrap();
    /// ```
    pub fn clear_area(&mut self, area: Rectangle) -> GuiResult<()> {
        self.draw(&area.into_styled(PrimitiveStyle::with_fill(self.style.background_color)))
            .map_err(|_| GuiError::DrawError(Some("Couldn't clear area")))
    }

    /// Clears the current row in the [Ui] with the background color.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating whether the row was successfully cleared.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.clear_row().unwrap();
    /// ```
    pub fn clear_row(&mut self) -> GuiResult<()> {
        let row_height = self.placer.row_height();
        let row_rect = Rectangle::new(
            Point::new(0, self.placer.pos.y),
            Size::new(self.placer.bounds.width, row_height),
        );
        self.clear_area(row_rect)
    }

    /// Clears the current row from the current widget position to the end of the row.
    ///
    /// This is useful for removing any rendering remains of partially drawn widgets.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating success or error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.clear_row_to_end().unwrap();
    /// ```
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

    /// Clears the [Ui] from the current placement position down to the bottom of the screen.
    ///
    /// **Warning:** This clears the entire screen area from the current row downwards.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating success or error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.clear_to_bottom().unwrap();
    /// ```
    pub fn clear_to_bottom(&mut self) -> GuiResult<()> {
        self.clear_area(Rectangle::new(
            Point::new(0, self.placer.pos.y),
            Size::new(
                self.placer.bounds.width,
                self.placer.bounds.height - self.placer.pos.y as u32,
            ),
        ))
    }

    /// Clears the entire background of the [Ui] with the background color defined in the style.
    ///
    /// This method updates the [Ui]'s cleared flag.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating success or error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.clear_background().unwrap();
    /// ```
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

// -- Drawing methods --
impl<'a, COL, DRAW> Ui<'a, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    /// Sets the internal drawing buffer for the [Ui].
    ///
    /// This buffer is used for optimized drawing and is optional.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let mut buffer = [Rgb565::BLACK; 10000];
    /// ui.set_buffer(&mut buffer);
    /// ```
    pub fn set_buffer(&mut self, buffer: &'a mut [COL]) {
        self.painter.set_buffer(buffer);
    }

    /// Begins the drawing process for a specified area in the [Ui].
    ///
    /// This initializes the drawing buffer (if set) and clears it with the background color.
    ///
    /// ## Panics
    ///
    /// Panics if the underlying [Painter] is already using its framebuffer.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let draw_area = Rectangle::new(Point::new(30, 20), Size::new(100, 150));
    /// ui.start_drawing(&draw_area);
    /// ```
    pub fn start_drawing(&mut self, area: &Rectangle) {
        self.painter.start_drawing(area);
        self.painter.clear_buffer(self.style.background_color);
    }

    /// Clears the current drawing buffer with the specified color.
    ///
    /// ## Returns
    ///
    /// `true` if the buffer was successfully cleared, or `false` if no buffer is present.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let cleared = ui.clear_buffer_raw(Rgb565::BLACK);
    /// println!("Buffer cleared: {}", cleared);
    /// ```
    pub fn clear_buffer_raw(&mut self, color: COL) -> bool {
        self.painter.clear_buffer(color)
    }

    /// Finalizes the drawing process for the [Ui].
    ///
    /// This flushes any buffered drawing to the underlying draw target.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating success or error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.finalize().unwrap();
    /// ```
    pub fn finalize(&mut self) -> GuiResult<()> {
        self.painter.finalize()
    }

    /// Draws a [Drawable] item onto the [Ui].
    ///
    /// If a buffer is active, the item is drawn to the buffer; otherwise, it is drawn directly.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating whether the drawing operation was successful.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// # use embedded_graphics::primitives::PrimitiveStyle;
    /// # let rectangle_shape = Rectangle::new(Point::new(50, 0), Size::new(200, 150));
    /// # // Make it drawable with a filled red style
    /// # let rectangle = rectangle_shape.into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
    /// ui.draw(&rectangle).unwrap();
    /// ```
    pub fn draw(&mut self, item: &impl Drawable<Color = COL>) -> GuiResult<()> {
        self.painter.draw(item)
    }
}

// -- Sub-[Ui] methods --
impl<COL, DRAW> Ui<'_, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    /// Creates a sub-[Ui] with the given bounds without performing extra bounds checks.
    ///
    /// This sub-[Ui] is useful for drawing to a specific area with its own layout and style.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating success or error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// let sub_bounds = Rectangle::new(Point::new(50, 0), Size::new(200, 150));
    /// ui.unchecked_sub_ui(sub_bounds, |sub_ui| {
    ///     sub_ui.add(widget);
    ///     Ok(())
    /// }).unwrap();
    /// ```
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

        let placer = Placer::new(
            bounds.size,
            true,
            Align(HorizontalAlign::Left, VerticalAlign::Top),
        );

        self.painter.with_subpainter(|painter| {
            let mut sub_ui = Ui {
                painter,
                bounds,
                style: self.style,
                interact: self.interact,
                placer,
                cleared: false,
                debug_color: self.debug_color,
                popup: None,
            };
            (f)(&mut sub_ui)
        })?;

        Ok(())
    }

    /// Creates a sub-[Ui] that shares the same bounds as the parent [Ui].
    ///
    /// Changes to the sub-[Ui]'s layout are reflected in the parent.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating success or error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.sub_ui(|sub_ui| {
    ///     sub_ui.add(widget);
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn sub_ui<F>(&mut self, f: F) -> GuiResult<()>
    where
        F: FnOnce(&mut Ui<DRAW, COL>) -> GuiResult<()>,
    {
        self.painter.with_subpainter(|painter| {
            let mut sub_ui = Ui {
                painter,
                bounds: self.bounds,
                style: self.style,
                interact: self.interact,
                placer: self.placer.clone(),
                cleared: false,
                debug_color: self.debug_color,
                popup: None,
            };
            let res = (f)(&mut sub_ui);
            self.placer = sub_ui.placer;
            res
        })?;

        Ok(())
    }

    /// Creates a right-side panel sub-[Ui] with the specified width.
    ///
    /// If `allow_smaller` is false, an error is returned if there is insufficient space.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating success or error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.right_panel_ui(100, false, |sub_ui| {
    ///     sub_ui.add(widget);
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn right_panel_ui<F>(&mut self, width: u32, allow_smaller: bool, f: F) -> GuiResult<()>
    where
        F: FnOnce(&mut Ui<DRAW, COL>) -> GuiResult<()>,
    {
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

    /// Creates a centered sub-[Ui] panel with the specified width and height.
    ///
    /// The panel is centered within the current bounds. An error is returned if the dimensions exceed the available space.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating success or error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.central_centered_panel_ui(200, 150, |sub_ui| {
    ///     sub_ui.add(widget);
    ///     Ok(())
    /// }).unwrap();
    /// ```
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

impl<'a, COL, DRAW> Ui<'a, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    /// Begins a popup layer with the specified state and buffer.
    ///
    /// The popup layer is used to display interactive widgets that are temporarily shown on top of the main UI.
    ///
    /// ## Parameters
    ///
    /// - `state`: A mutable reference to the [PopupState] that controls the popup's behavior and appearance.
    /// - `buffer`: A mutable reference to the buffer used to draw the popup's contents.
    ///
    /// # Notice
    ///
    /// - The `state` parameter must be declared outside the UI loop.
    ///
    /// # Example
    ///
    /// References [crate::combo_box::ComboBox] Example
    ///
    pub fn begin_popup(&mut self, state: &'a mut PopupState, buffer: &'a mut [COL]) {
        if state.stage == PopupStage::Handled {
            state.stage = PopupStage::Hide;
            self.clear_background().ok();
        }
        self.popup = Some(Popup::new(state, buffer));
    }

    /// Ends the popup layer, draw the popup contents to main UI.
    ///
    /// This method should be called after all widgets have been drawed to main UI.
    ///
    /// # Example
    ///
    /// References [crate::combo_box::ComboBox] Example
    ///
    pub fn end_popup<F>(&mut self, popup_handled: F)
    where
        F: FnOnce(),
    {
        let Some(popup) = self.popup.as_mut() else {
            return;
        };

        let Some(bounds) = popup.state.bounds else {
            return;
        };

        if let Interaction::Click(pt) = popup.interact {
            if !bounds.contains(pt) {
                popup.state.stage = PopupStage::Handled;
            }
        }

        popup.interact = Interaction::None;
        if popup.state.stage == PopupStage::Show {
            if let Some(framebuf) = WidgetFramebuf::try_new(
                popup.buffer,
                bounds.size,
                bounds.top_left + Point::new(0, popup.state.offset_y),
            ) {
                framebuf.draw(self.painter.target).ok();
            }
        }

        if popup.state.stage == PopupStage::Handled {
            popup_handled();
        }
    }

    /// Checks if the popup layer should redraw.
    ///
    /// ## Returns
    ///
    /// `true` if the popup layer should redraw, `false` otherwise.
    ///
    /// # Example
    ///
    /// References [crate::combo_box::ComboBox] Example
    ///
    pub(crate) fn popup_check(&mut self) -> bool {
        let Some(popup) = self.popup.as_mut() else {
            return false;
        };

        if popup.state.stage == PopupStage::Show
            && popup.state.col == self.placer.col
            && popup.state.row == self.placer.row
        {
            popup.interact = if let Some(mut pt) = self.interact.get_point() {
                pt.y -= popup.state.offset_y;
                let popup_interact = match self.interact {
                    Interaction::Click(_) => Interaction::Click(pt),
                    Interaction::Drag(_) => Interaction::Drag(pt),
                    Interaction::Release(_) => Interaction::Release(pt),
                    Interaction::Hover(_) => Interaction::Hover(pt),
                    _ => Interaction::None,
                };
                self.interact = Interaction::None;
                popup_interact
            } else {
                Interaction::None
            };
            return true;
        }

        return false;
    }

    /// Draws the popup layer with the specified contents.
    ///
    /// ## Parameters
    ///
    /// - `top_left`: The top-left position of the popup layer.
    /// - `width`: The width of the popup layer.
    /// - `popup_contents`: A closure that takes a mutable reference to the popup layer's UI and returns a boolean indicating whether the popup should be closed.
    ///
    /// ## Returns
    ///
    /// `Ok(true)` if the popup has handled and will close, `Ok(false)` otherwise.
    ///
    /// # Example
    ///
    /// References [crate::combo_box::ComboBox] Example
    ///
    pub(crate) fn popup_draw<F>(
        &mut self,
        top_left: Point,
        width: u16,
        popup_contents: F,
    ) -> GuiResult<bool>
    where
        COL: PixelColor,
        F: FnOnce(&mut Ui<WidgetFramebuf<COL>, COL>) -> bool,
    {
        if width == 0 {
            return Err(GuiError::DrawError(None));
        }

        let screen_height = self.get_screen_height() as i32;
        let Some(popup) = self.popup.as_mut() else {
            return Err(GuiError::DrawError(Some("Popup layer not initialized")));
        };

        let mut bounds = Rectangle::new(
            top_left,
            Size::new(width as u32, popup.buffer.len() as u32 / width as u32),
        );
        let style = self.style;

        if let Some(mut framebuf) =
            WidgetFramebuf::try_new(popup.buffer, bounds.size, bounds.top_left)
        {
            let mut popup_ui = Ui::new(&mut framebuf, bounds, style);
            if let Some(dbg_color) = self.debug_color {
                popup_ui.draw_widget_bounds_debug(dbg_color);
            }
            popup_ui.interact = popup.interact;
            popup.state.stage = PopupStage::Drawing;
            popup_ui.begin_popup(popup.state, &mut []);
            popup_ui.clear_background()?;
            let selected = popup_contents(&mut popup_ui); // Draw popup contents
            bounds.size.height = popup_ui.get_placer_top_left().y as u32
                + popup_ui.style().spacing.window_border_padding.height;
            popup.state.offset_y = if bounds.top_left.y + bounds.size.height as i32 > screen_height
            {
                screen_height - bounds.size.height as i32 - bounds.top_left.y
            } else {
                0
            };
            if selected {
                popup.state.stage = PopupStage::Handled;
            } else {
                popup.state.stage = PopupStage::Show;
                popup.state.col = self.placer.col;
                popup.state.row = self.placer.row;
                popup.state.bounds = Some(bounds);
            }
            Ok(selected)
        } else {
            Err(GuiError::DrawError(Some("Popup buffer too small")))
        }
    }
}

// -- Debug drawing methods --
impl<COL, DRAW> Ui<'_, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    /// Draws a debug outline around the [Ui]'s bounds using the specified color.
    ///
    /// ## Returns
    ///
    /// A [GuiResult] indicating success or error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.draw_bounds_debug(Rgb565::RED).unwrap();
    /// ```
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

    /// Enables debug drawing of widget bounds in the [Ui] using the specified color.
    ///
    /// Once set, all added widgets will have their bounds outlined in this color for debugging.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_graphics::pixelcolor::Rgb565;
    /// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
    /// # use kolibri_embedded_gui::style::medsize_rgb565_style;
    /// # use kolibri_embedded_gui::ui::Ui;
    /// # use embedded_graphics::prelude::*;
    /// # use embedded_graphics::primitives::Rectangle;
    /// # use embedded_iconoir::prelude::*;
    /// # use kolibri_embedded_gui::ui::*;
    /// # use kolibri_embedded_gui::label::*;
    /// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    /// # let output_settings = OutputSettingsBuilder::new().build();
    /// # let mut window = Window::new("Kolibri Example", &output_settings);
    /// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    /// # let mut widget = Label::new("Hi");
    /// ui.draw_widget_bounds_debug(Rgb565::GREEN);
    /// ```
    pub fn draw_widget_bounds_debug(&mut self, color: COL) {
        self.debug_color = Some(color);
    }
}
