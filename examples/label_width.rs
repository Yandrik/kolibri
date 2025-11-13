use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::Point;
use embedded_graphics_simulator::sdl2::MouseButton;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use kolibri_embedded_gui::label::{HashLabel, Hasher, Label};
use kolibri_embedded_gui::slider::Slider;
use kolibri_embedded_gui::smartstate::SmartstateProvider;
use kolibri_embedded_gui::spacer::Spacer;
use kolibri_embedded_gui::style::*;
use kolibri_embedded_gui::ui::Ui;

fn main() -> Result<(), core::convert::Infallible> {
    // ILI9341-clone like display
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
    let mut additional = 0;

    let output_settings = OutputSettingsBuilder::new()
        // .pixel_spacing(2)
        .scale(2)
        .build();
    let mut window = Window::new("Hello World", &output_settings);
    let mut smartstates = SmartstateProvider::<20>::new();
    let mut _mouse_down = false;
    let mut _last_down = false;
    let mut _location = Point::new(0, 0);

    // clear bg once
    let mut ui = Ui::new_fullscreen(&mut display, medsize_sakura_rgb565_style());
    ui.clear_background().unwrap();

    // alloc buffer
    let mut buffer = [Rgb565::new(0, 0, 0); 100 * 100];
    let hasher = Hasher::new();

    'outer: loop {
        let mut ui = Ui::new_fullscreen(&mut display, medsize_sakura_rgb565_style());
        if additional == 0 {
            ui.clear_background().unwrap();
            smartstates.force_redraw_all();
        }
        //ui.draw_widget_bounds_debug(Rgb565::CSS_RED);
        ui.set_buffer(&mut buffer);
        smartstates.restart_counter();
        let short = "A short label";
        let long = "A label that is far too long to fit the width of the display";
        ui.add(Label::new(short));
        ui.add(Label::new(long));
        ui.add(HashLabel::new(long, smartstates.nxt(), &hasher).auto_truncate());
        ui.add(Label::new(short));
        ui.add(Label::new(long).auto_truncate());
        if additional == 0 {
            ui.clear_row().unwrap();
        }
        ui.add_horizontal(Spacer::new(Size::new(0, 20)));
        ui.add(Label::new(long).auto_truncate_to(150 + additional));
        ui.add(Slider::new(&mut (additional as i16), 0..=150).width(150 + additional));
        additional = (additional + 1) % 150;
        ui.add(Label::new(short));

        window.update(&display);

        for evt in window.events() {
            match evt {
                SimulatorEvent::KeyUp { .. } => {}
                SimulatorEvent::KeyDown { .. } => {}
                SimulatorEvent::MouseButtonUp { mouse_btn, point } => {
                    if let MouseButton::Left = mouse_btn {
                        _mouse_down = false;
                    }
                    _location = point;
                }
                SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
                    if let MouseButton::Left = mouse_btn {
                        _mouse_down = true;
                    }
                    _location = point;
                }
                SimulatorEvent::MouseWheel { .. } => {}
                SimulatorEvent::MouseMove { point } => {
                    _location = point;
                }
                SimulatorEvent::Quit => break 'outer,
            }
        }
    }
    Ok(())
}
