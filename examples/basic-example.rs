use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::ascii;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::Point;
use embedded_graphics_simulator::sdl2::MouseButton;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use kolibri_embedded_gui::button::Button;
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::style::medsize_rgb565_style;
use kolibri_embedded_gui::ui::{Interaction, Ui};

fn main() -> Result<(), core::convert::Infallible> {
    // Simulator Setup (ILI9341-like Display)
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

    // Output Settings. Change for other screen appearance / size scaling.
    let output_settings = OutputSettingsBuilder::new()
        // .pixel_spacing(2)
        .scale(2)
        .build();
    let mut window = Window::new("Hello World", &output_settings);

    // input handling variables
    let mut mouse_down = false;
    let mut last_down = false;
    let mut location = Point::new(0, 0);

    // counter for incrementing thingy
    let mut i = 0u8;

    // clear bg once
    let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    ui.clear_background().unwrap();

    'outer: loop {
        // create UI (needs to be done each frame)
        let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());

        // handle input
        match (last_down, mouse_down, location) {
            (false, true, loc) => {
                ui.interact(Interaction::Click(loc));
            }
            (true, true, loc) => {
                ui.interact(Interaction::Drag(loc));
            }
            (true, false, loc) => {
                ui.interact(Interaction::Release(loc));
            }
            (false, false, loc) => {
                ui.interact(Interaction::Hover(loc));
            }
        }

        // clear UI background (for non-incremental redrawing framebuffered applications)
        ui.clear_background().ok();

        last_down = mouse_down;

        // === ACTUAL UI CODE STARTS HERE ===

        ui.add(Label::new("Basic Example").with_font(ascii::FONT_10X20));

        ui.add(Label::new("Basic Counter (7LOC)"));

        if ui.add_horizontal(Button::new("-")).clicked() {
            i = i.saturating_sub(1);
        }
        ui.add_horizontal(Label::new(format!("Clicked {} times", i).as_ref()));
        if ui.add_horizontal(Button::new("+")).clicked() {
            i = i.saturating_add(1);
        }

        // === ACTUAL UI CODE ENDS HERE ===

        // simulator window update
        window.update(&display);

        // take input, and quit application if necessary
        for evt in window.events() {
            match evt {
                SimulatorEvent::KeyUp { .. } => {}
                SimulatorEvent::KeyDown { .. } => {}
                SimulatorEvent::MouseButtonUp { mouse_btn, point } => {
                    if let MouseButton::Left = mouse_btn {
                        mouse_down = false;
                    }
                    location = point;
                }
                SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
                    if let MouseButton::Left = mouse_btn {
                        mouse_down = true;
                    }
                    location = point;
                }
                SimulatorEvent::MouseWheel { .. } => {}
                SimulatorEvent::MouseMove { point } => {
                    location = point;
                }
                SimulatorEvent::Quit => break 'outer,
            }
        }
    }
    Ok(())
}
