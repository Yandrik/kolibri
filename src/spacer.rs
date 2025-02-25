//! # Spacer Widget
//!
//! The [Spacer] widget creates empty space between UI elements to improve layout and readability.
//! It's a simple utility widget that helps with spacing and alignment without drawing any visible content.
//!
//! ## Core Features
//!
//! - Creates precise empty space with specified width and height
//! - Helps with UI layout and alignment
//! - Zero visual footprint (draws nothing)
//! - Useful for margins, padding, and visual separation between components
//!
//! ## Usage
//!
//! ```no_run
//! # use embedded_graphics::pixelcolor::Rgb565;
//! # use embedded_graphics_simulator::{SimulatorDisplay, OutputSettingsBuilder, Window};
//! # use kolibri_embedded_gui::style::medsize_rgb565_style;
//! # use kolibri_embedded_gui::ui::Ui;
//! # use embedded_graphics::prelude::*;
//! # use kolibri_embedded_gui::spacer::Spacer;
//! # use kolibri_embedded_gui::label::Label;
//! # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
//! # let output_settings = OutputSettingsBuilder::new().build();
//! # let mut window = Window::new("Kolibri Example", &output_settings);
//! # let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
//!
//! // Add a UI element
//! ui.add(Label::new("First element"));
//!
//! // Add vertical space of 10 pixels
//! ui.add(Spacer::new(Size::new(0, 10)));
//!
//! // This label is 10px lower than the previous label
//! ui.add(Label::new("Second element"));
//!
//! // Creating horizontal space
//! ui.add_horizontal(Spacer::new(Size::new(20, 0)));
//!
//! // This label is offset horizontally from the left screen edge
//! ui.add_horizontal(Label::new("Second element"));
//!
//! ```
//!
//! ## Implementation Details
//!
//! The `Spacer` widget simply allocates space in the UI layout system without drawing
//! any content. It's one of the most lightweight widgets in Kolibri, as it only interacts
//! with the layout system to reserve space where nothing will be drawn.
//!

use crate::ui::{GuiResult, Response, Ui, Widget};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::PixelColor;

pub struct Spacer {
    space: Size,
}

impl Spacer {
    pub fn new(space: Size) -> Spacer {
        Spacer { space }
    }
}

impl Widget for Spacer {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // allocate space
        let space = ui.allocate_space(self.space)?;

        Ok(Response::new(space))
    }
}
