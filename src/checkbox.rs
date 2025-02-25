//! # Checkbox Widget
//!
//! A customizable checkbox widget that provides a simple boolean state control.
//!
//! The checkbox widget provides a traditional square control that can be toggled between checked
//! and unchecked states. It features an automatic icon that scales based on the available space
//! and integrates with the framework's theming system for consistent appearance.
//!
//! This widget is part of the Kolibri embedded GUI framework's core widget set and integrates
//! with the framework's [Smartstate] system for efficient rendering.
//!
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

/// A checkbox widget for toggling boolean values.
///
/// The checkbox state is stored in a mutable reference to a boolean value, allowing
/// the application to directly access the current state.
///
/// ## Example
///
/// ```no_run
/// # use embedded_graphics::pixelcolor::Rgb565;
/// # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
/// # use kolibri_embedded_gui::style::medsize_rgb565_style;
/// # use kolibri_embedded_gui::ui::Ui;
/// # use embedded_graphics::prelude::*;
/// # use embedded_graphics::primitives::Rectangle;
/// # use embedded_iconoir::prelude::*;
/// # use embedded_iconoir::size12px;
/// # use kolibri_embedded_gui::ui::*;
/// # use kolibri_embedded_gui::label::*;
/// # use kolibri_embedded_gui::smartstate::*;
/// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
/// # let output_settings = OutputSettingsBuilder::new().build();
/// # let mut window = Window::new("Kolibri Example", &output_settings);
/// # use kolibri_embedded_gui::checkbox::Checkbox;
///
/// let mut checked = false;
/// let mut smartstates = SmartstateProvider::<10>::new();
///
/// // Create a simple checkbox
/// # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
/// let checkbox = Checkbox::new(&mut checked);
///
/// ui.add(checkbox);
///
/// if checked {
///     // Handle checked state
/// }
///
/// // OR
///
/// # let checkbox = Checkbox::new(&mut checked);
/// if ui.add(checkbox).changed() {
///    // Handle state change only when the checkbox is toggled
/// }
///
/// // Create a checkbox with smartstate for optimized rendering
/// let checkbox_with_smartstate = Checkbox::new(&mut checked)
///     .smartstate(smartstates.nxt());
///
/// ```
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

    /// Attaches a [Smartstate] to the checkbox for incremental redrawing.
    ///
    /// When a smartstate is attached, the checkbox will only redraw when its state
    /// or appearance changes, improving performance on resource-constrained devices.
    ///
    /// This is particularly useful when using embedded displays with slow update rates
    /// or when minimizing power consumption is important.
    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

impl Checkbox<'_> {
    /// Draws the icon for the checkbox.
    ///
    /// This internal helper method handles drawing the check mark icon when the checkbox
    /// is in the checked state. It positions the icon in the center of the checkbox area
    /// based on the provided center offset.
    ///
    /// The icon size is determined by the calling code based on available space.
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

impl Widget for Checkbox<'_> {
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
