use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, StyledDrawable};
use embedded_graphics_simulator::sdl2::MouseButton;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use kolibri_embedded_gui::button::Button;
use kolibri_embedded_gui::helpers::keyboard;
use kolibri_embedded_gui::helpers::keyboard::{draw_keyboard, Layout};
use kolibri_embedded_gui::icon::IconWidget;
use kolibri_embedded_gui::icons::size24px;
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::smartstate::SmartstateProvider;
use kolibri_embedded_gui::style::medsize_rgb565_style;
use kolibri_embedded_gui::ui::{Interaction, Ui};

fn main() -> Result<(), core::convert::Infallible> {
    // ILI9341-clone like display
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

    Circle::new(Point::new(100, 100), 50)
        .draw_styled(&PrimitiveStyle::with_stroke(Rgb565::RED, 2), &mut display)
        .ok();

    let output_settings = OutputSettingsBuilder::new()
        // .pixel_spacing(2)
        // .scale(2)
        .build();
    let mut window = Window::new("Hello World", &output_settings);

    let mut mouse_down = false;
    let mut last_down = false;
    let mut location = Point::new(0, 0);
    let mut smartstates = SmartstateProvider::<50>::new();

    // clear bg once
    let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    ui.clear_background().unwrap();

    // alloc buffer
    let mut buffer = [Rgb565::new(0, 0, 0); 100 * 100];

    let mut shift = false;
    let mut open = true;
    let mut text = keyboard::String::<16>::new();
    let mut last_len: usize = 0;

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

        let text_smartstate = smartstates.get_pos();
        ui.add_and_clear_col_remainder(
            Label::new(format!("Text: {}", text).as_ref()).smartstate(smartstates.nxt()),
            last_len > text.len(),
        );
        last_len = text.len();

        ui.clear_row_to_end().unwrap();
        ui.new_row();

        ui.expand_row_height(20);
        if ui
            .add_horizontal(Button::new("Open Keyboard").smartstate(smartstates.nxt()))
            .clicked()
        {
            open = true;
        };
        ui.add(IconWidget::<size24px::layout::CornerBottomLeft,Rgb565>::new_from_type());

        // ui.add(Keyboard::new(Layout::qwerty(), &mut None, &mut false));

        /*
        ui.add_horizontal(Button::new("Q"));
        ui.add_horizontal(Button::new("W"));
        ui.add_horizontal(Button::new("E"));
        ui.add_horizontal(Button::new("R"));
        ui.add_horizontal(Button::new("T"));
        ui.add_horizontal(Button::new("Y"));
        ui.add_horizontal(Button::new("U"));
        ui.add_horizontal(Button::new("I"));
        ui.add_horizontal(Button::new("O"));
        ui.add_horizontal(Button::new("P"));
        ui.add(IconButton::<size16px::navigation::NavArrowLeft>::new_from_type());
         */

        ui.sub_ui(|ui| {
            ui.style_mut().spacing.item_spacing.width = 3;
            ui.style_mut().spacing.button_padding.width = 5;
            ui.style_mut().spacing.item_spacing.height = 2;

            if draw_keyboard(
                ui,
                // &Layout::qwerty_with_special(),
                // &Layout::qwerty(),
                &Layout::qwertz_with_special(),
                Some(&mut smartstates),
                true,
                true,
                &mut shift,
                &mut open,
                &mut text,
            )
            .changed()
            {
                println!("Text: {}", text);
                smartstates.get(text_smartstate).force_redraw();
            };
            Ok(())
        })
        .unwrap();

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
