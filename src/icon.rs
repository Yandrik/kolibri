//! # Icon Widget
//!
//! The [IconWidget] widget is used to draw icons as widgets in Kolibri.
//! It leverages the [embedded_iconoir] crate for icon data and supports Kolibri's smartstate system
//! for efficient redrawing.
//!
//! ## Core Features
//!
//! - Simple display of icons from the Iconoir icon set
//! - Automatic integration with Kolibri's theming system (uses colors from the current style)
//! - Vertical centering of icons within the allocated space
//! - Support for the smartstate system to minimize unnecessary redraws
//!
//! ## Usage
//!
//! ```no_run
//! # use embedded_graphics::pixelcolor::Rgb565;
//! # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
//! # use kolibri_embedded_gui::style::medsize_rgb565_style;
//! # use kolibri_embedded_gui::ui::Ui;
//! # use embedded_graphics::prelude::*;
//! # use embedded_graphics::primitives::Rectangle;
//! # use embedded_iconoir::prelude::*;
//! # use embedded_iconoir::size12px;
//! # use kolibri_embedded_gui::ui::*;
//! # use embedded_graphics::mono_font::ascii;
//! # use kolibri_embedded_gui::label::*;
//! # use kolibri_embedded_gui::smartstate::*;
//! # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
//! # let output_settings = OutputSettingsBuilder::new().build();
//! # let mut window = Window::new("Kolibri Example", &output_settings);
//! # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
//! # let mut smartstateProvider = SmartstateProvider::<20>::new();
//! # use kolibri_embedded_gui::icon::*;
//! // Basic icon usage
//! ui.add(IconWidget::new(size12px::actions::AddCircle));
//!
//! // Using with the type system instead of passing an icon instance
//! ui.add(IconWidget::<size12px::actions::AddCircle>::new_from_type());
//!
//! // Using smartstate for efficient redrawing
//! let mut my_smartstate = Smartstate::empty();
//! ui.add(IconWidget::new(size12px::actions::AddCircle).smartstate(&mut my_smartstate));
//! ```
//!
//! ## Implementation Details
//!
//! The `Icon` widget uses the default icon color from the current style and allocates
//! space based on the icon's size. It leverages the smartstate system to redraw only
//! when necessary, improving performance for stationary UI elements.
//!

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

/// A widget for displaying an Iconoir icon.
///
/// This widget renders icons from the Iconoir library using the [embedded_iconoir], applying
/// colors from the current style system.
///
/// You can choose an icon from all resolutions of [embedded_iconoir], such as [embedded_iconoir::size12px] up to [embedded_iconoir::size144px].
/// For all icons, see [embedded_iconoir::size12px]
pub struct IconWidget<'a, Ico: IconoirIcon> {
    marker: PhantomData<Ico>,
    smartstate: Container<'a, Smartstate>,
}

impl<'a, Ico: IconoirIcon> IconWidget<'a, Ico> {
    /// Creates a new [IconWidget] from an [IconoirIcon] instance.
    ///
    /// The icon color from the icon instance will be ignored, as the widget
    /// will use the icon color from the current UI style.
    ///
    /// You can choose an icon from all resolutions of [embedded_iconoir], such as [embedded_iconoir::size12px] up to [embedded_iconoir::size144px].
    /// For all icons, see [embedded_iconoir::size12px]
    pub fn new(_icon: Ico) -> Self {
        Self {
            marker: PhantomData,
            smartstate: Container::empty(),
        }
    }

    /// Creates a new [IconWidget] using just the icon's type.
    ///
    /// This is a convenience method that allows creating an icon without
    /// instantiating the icon object first.
    pub fn new_from_type() -> Self {
        Self {
            marker: PhantomData,
            smartstate: Container::empty(),
        }
    }

    /// Attaches a [Smartstate] to this widget for efficient redrawing.
    ///
    /// When a smartstate is attached, the widget will only redraw when its
    /// state changes, improving performance for stationary UI elements.
    ///
    /// Returns `self` for method chaining.
    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

impl<Ico: IconoirIcon> Widget for IconWidget<'_, Ico> {
    /// Draws the icon within the UI.
    ///
    /// This method:
    /// 1. Creates an icon with the current style's icon color
    /// 2. Allocates space based on the icon's size
    /// 3. Updates the smartstate
    /// 4. Draws the icon if necessary (when smartstate changes or is forced to redraw)
    /// 5. Centers the icon vertically within the allocated space
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
