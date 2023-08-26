use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::mapping::ISO_8859_1;
use embedded_graphics::mono_font::{ascii, iso_8859_10};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::{Point, WebColors};
use embedded_graphics::primitives::{Circle, PrimitiveStyle, StyledDrawable};
use embedded_graphics::text::Text;
use embedded_graphics_simulator::sdl2::MouseButton;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use kolibri_embedded_gui::button::Button;
use kolibri_embedded_gui::checkbox::Checkbox;
use kolibri_embedded_gui::icon::IconWidget;
use kolibri_embedded_gui::iconbutton::IconButton;
use kolibri_embedded_gui::icons::{size12px, size24px, size32px};
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::prelude::*;
use kolibri_embedded_gui::smartstate::{Smartstate, SmartstateProvider};
use kolibri_embedded_gui::spacer::Spacer;
use kolibri_embedded_gui::style::{
    medsize_blue_rgb565_style, medsize_crt_rgb565_style, medsize_light_rgb565_style,
    medsize_retro_rgb565_style, medsize_rgb565_debug_style, medsize_rgb565_style,
    medsize_sakura_rgb565_style,
};
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

    // mutable variables for state persistence
    let mut checkbox1 = false;

    // theme to use
    let mut theme = medsize_rgb565_style();

    // clear bg once
    let mut ui = Ui::new_fullscreen(&mut display, theme.clone());
    ui.clear_background().unwrap();

    'outer: loop {
        // create UI (needs to be done each frame)
        let mut ui = Ui::new_fullscreen(&mut display, theme.clone());

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

        ui.add(Label::new("Theming Example").with_font(ascii::FONT_10X20));

        ui.add_horizontal(Label::new("Label"));
        ui.add(Label::new("Small Label").with_font(ascii::FONT_6X13));

        ui.add_horizontal(Button::new("Button"));
        ui.add(Checkbox::new(&mut checkbox1));

        // add button first to center icon vertically
        ui.add_horizontal(IconButton::new(size24px::navigation::ArrowUpCircle));
        ui.add(IconWidget::<size24px::actions::RefreshDouble>::new_from_type());

        // one row offset
        ui.new_row();

        // theming buttons and such
        ui.add(Label::new("Set Colors"));
        ui.add(Label::new("Note that these are themes cobbled together in\na few minutes, so they might not look great.").with_font(ascii::FONT_5X8));
        if ui.add_horizontal(Button::new("Dark")).clicked() {
            theme = medsize_rgb565_style();
        }
        if ui.add_horizontal(Button::new("Light")).clicked() {
            theme = medsize_light_rgb565_style();
        }
        if ui.add_horizontal(Button::new("Sakura")).clicked() {
            theme = medsize_sakura_rgb565_style();
        }
        if ui.add_horizontal(Button::new("Blue")).clicked() {
            theme = medsize_blue_rgb565_style();
        }
        ui.new_row();
        if ui.add_horizontal(Button::new("CRT")).clicked() {
            theme = medsize_crt_rgb565_style();
        }
        if ui.add_horizontal(Button::new("Retro")).clicked() {
            theme = medsize_retro_rgb565_style();
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
