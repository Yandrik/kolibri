use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, StyledDrawable};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use kolibri_embedded_gui::button::Button;
use kolibri_embedded_gui::style::medsize_rgb565_style;
use kolibri_embedded_gui::ui::Ui;

fn main() -> Result<(), core::convert::Infallible> {
    // ILI9341-clone like display
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

    let circ = Circle::new(Point::new(100, 100), 50)
        .draw_styled(&PrimitiveStyle::with_stroke(Rgb565::RED, 2), &mut display);

    let output_settings = OutputSettingsBuilder::new()
        // .pixel_spacing(2)
        // .scale(2)
        .build();
    let mut window = Window::new("Hello World", &output_settings);

    let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    ui.add(Button::new("Something")).unwrap();
    ui.add_horizontal(Some(20), Button::new("This is creative!"))
        .unwrap();
    ui.add(Button::new("Isn't it? \nIt totally is!")).unwrap();
    ui.add(Button::new("More Text")).unwrap();
    ui.add(Button::new("This is a wall\nI believe")).unwrap();
    ui.add(Button::new("Smol")).unwrap();

    'outer: loop {
        window.update(&display);
        for evt in window.events() {
            match evt {
                SimulatorEvent::KeyUp { .. } => {}
                SimulatorEvent::KeyDown { .. } => {}
                SimulatorEvent::MouseButtonUp { .. } => {}
                SimulatorEvent::MouseButtonDown { .. } => {}
                SimulatorEvent::MouseWheel { .. } => {}
                SimulatorEvent::MouseMove { .. } => {}
                SimulatorEvent::Quit => break 'outer,
            }
        }
    }

    Ok(())
}
