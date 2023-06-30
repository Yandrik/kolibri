use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, StyledDrawable};
use embedded_graphics::text::Text;
use embedded_graphics_simulator::sdl2::MouseButton;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use kolibri_embedded_gui::button::Button;
use kolibri_embedded_gui::checkbox::Checkbox;
use kolibri_embedded_gui::icon::IconWidget;
use kolibri_embedded_gui::icons::{size12px, size24px, size32px};
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::prelude::*;
use kolibri_embedded_gui::smartstate::{Smartstate, SmartstateProvider};
use kolibri_embedded_gui::style::{medsize_rgb565_debug_style, medsize_rgb565_style};
use kolibri_embedded_gui::ui::{Interaction, Ui};

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

    let mut mouse_down = false;
    let mut last_down = false;
    let mut location = Point::new(0, 0);

    let mut i = 0u8;

    let (mut b1, mut b2, mut b3, mut b4, mut b5, mut b6) =
        (false, false, false, false, false, false);

    let mut smartstates = SmartstateProvider::<10>::new();
    let mut c1 = false;

    // clear bg once
    let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    ui.clear_background().unwrap();

    // alloc buffer
    let mut buffer = [Rgb565::new(0, 0, 0); 100 * 100];

    'outer: loop {
        let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
        ui.set_buffer(&mut buffer);
        smartstates.restart_counter();

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

        last_down = mouse_down;

        if ui
            .add_horizontal(
                None,
                Button::new("Something", &mut b1).smartstate(smartstates.next()),
            )
            .clicked()
        {
            i = i.saturating_add(1);
            println!("Clicked! i: {}", i);
            smartstates.peek().force_redraw();
        }
        ui.add_horizontal(
            None,
            Label::new(format!("Clicked {} times", i).as_ref()).smartstate(smartstates.next()),
        );

        ui.clear_col_to_end().unwrap();
        ui.new_row();

        ui.add_horizontal(
            Some(20),
            Button::new("This is creative!", &mut b2).smartstate(smartstates.next()),
        );
        ui.add(IconWidget::<size24px::layout::CornerBottomLeft>::new_from_type());
        ui.add(Button::new("Isn't it? \nIt totally is!", &mut b3).smartstate(smartstates.next()));
        ui.add(Button::new("", &mut b4).smartstate(smartstates.next()));
        ui.add_horizontal(
            None,
            Label::new("Wanna live?").smartstate(smartstates.next()),
        );
        ui.add(Checkbox::new(&mut c1).smartstate(smartstates.next()));

        window.update(&display);

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
