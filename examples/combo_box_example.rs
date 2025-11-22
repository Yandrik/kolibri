use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::ascii;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Point, WebColors};
use embedded_graphics_simulator::sdl2::MouseButton;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use kolibri_embedded_gui::checkbox::Checkbox;
use kolibri_embedded_gui::combo_box::ComboBox;
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::smartstate::SmartstateProvider;
use kolibri_embedded_gui::ui::{Interaction, PopupState, Ui};

fn main() -> Result<(), core::convert::Infallible> {
    // Simulator Setup (ILI9341-like Display)
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

    // Output Settings. Change for other screen appearance / size scaling.
    let output_settings = OutputSettingsBuilder::new()
        // .pixel_spacing(2)
        .scale(1)
        .build();
    let mut window = Window::new("Hello World", &output_settings);

    // input handling variables
    let mut mouse_down = false;
    let mut last_down = false;
    let mut location = Point::new(0, 0);

    // counter for incrementing thingy
    let mut popup_state = PopupState::default();
    let mut smartstates = SmartstateProvider::<20>::new();

    // clear bg once
    let mut ui = Ui::new_fullscreen(
        &mut display,
        kolibri_embedded_gui::style::medsize_light_rgb565_style(),
    );
    ui.clear_background().unwrap();

    println!("Hello World!");

    let mut checked = false;
    let mut selected0 = "Hello";
    let mut selected1 = "World";
    let mut selected2 = "Cross the bounds";

    'outer: loop {
        let mut popup_buffer = [Rgb565::CSS_BLACK; 320 * 240];

        // create UI (needs to be done each frame)
        let mut ui = Ui::new_fullscreen(
            &mut display,
            kolibri_embedded_gui::style::medsize_light_rgb565_style(),
        );
        //ui.draw_widget_bounds_debug(Rgb565::RED);

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
        //ui.clear_background().ok();
        ui.begin_popup(&mut popup_state, &mut popup_buffer);
        smartstates.restart_counter();

        last_down = mouse_down;

        // === ACTUAL UI CODE STARTS HERE ===

        ui.add(Label::new("ComboBox Example").with_font(ascii::FONT_10X20));

        // if ComboBox::new()
        //     .selected_text(selected0)
        //     .smartstate(smartstates.nxt())
        //     .show_ui(&mut ui, || {
        //         ComboBox::new_contents(
        //             &mut selected0,
        //             &["Hello 1", "Hello Hello 2", "Hello Hello Hello 3"],
        //         )
        //     })
        //     .changed()
        // {
        //     println!("ComboBox changed to {}", selected0);
        // }

        ui.add_horizontal(Checkbox::new(&mut checked));

        /* 
        if ui.add(ComboBox::new(
& || {
                ComboBox::new_contents(
                    &mut selected1,
                    &["World 1", "World World 2", "World World World 3"],
                )
            },
        )
        .selected_text(selected1)
            .smartstate(smartstates.nxt())
            .with_width(100))
            .changed()
            
        {
            println!("ComboBox changed to {}", selected1);
        }*/
        ui.new_row();
        ui.add(Label::new("Label 1").smartstate(smartstates.nxt()));
        ui.add(Label::new("Label 2").smartstate(smartstates.nxt()));
        ui.add(Label::new("Long Label 3").smartstate(smartstates.nxt()));
        ui.add(Label::new("Long Long Label 4").smartstate(smartstates.nxt()));
        ui.add(Label::new("Long Long Long Label 5").smartstate(smartstates.nxt()));
        ui.new_row();

        if ui.add(ComboBox::new(
            &mut selected0,
            &["Cross 1", "the the 2", "bounds bounds bounds 3"],

        )
            .smartstate(smartstates.nxt()))
            .changed()
        {
            println!("ComboBox changed to {}", selected2);
        }

        ui.end_popup(|| {
            println!("Popup handled");
            smartstates.force_redraw_all();
        });
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
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    Ok(())
}
