use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::ascii;
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

    let mut ib1 = false;
    let mut ib2 = false;

    let mut smartstates = SmartstateProvider::<20>::new();
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
            .add_horizontal(Button::new("Something").smartstate(smartstates.next()))
            .clicked()
        {
            i = i.saturating_add(1);
            println!("Clicked! i: {}", i);
            smartstates.peek().force_redraw();
        }
        ui.add_horizontal(
            Label::new(format!("Clicked {} times", i).as_ref()).smartstate(smartstates.next()),
        );

        ui.clear_row_to_end().unwrap();
        ui.new_row();

        ui.expand_row_height(20);
        ui.add_horizontal(Button::new("This is creative!").smartstate(smartstates.next()));
        ui.add(IconWidget::<size24px::layout::CornerBottomLeft>::new_from_type());

        /*
        ui.right_panel_ui(200, true, |ui| {
            ui.add(Label::new("Right panel").smartstate(smartstates.next()));
            ui.add(Label::new("Cool, ryte?").smartstate(smartstates.next()));

            ui.sub_ui(|ui| {
                let style = ui.style_mut();

                style.item_background_color = Rgb565::CSS_ORANGE_RED;
                style.highlight_item_background_color = Rgb565::CSS_ORANGE_RED;
                style.primary_color = Rgb565::CSS_RED;
                style.text_color = Rgb565::CSS_BLACK;

                ui.add_horizontal(
                    None,
                    IconButton::<size32px::audio::MicRemove>::new_from_type(&mut ib1)
                        .smartstate(smartstates.next()),
                );

                Ok(())
            })
                .unwrap();

            ui.add_horizontal(None, Spacer::new((20, 0).into()));

            ui.sub_ui(|ui| {
                let style = ui.style_mut();

                style.item_background_color = Rgb565::CSS_LIME_GREEN;
                style.highlight_item_background_color = Rgb565::CSS_LIME_GREEN;
                style.primary_color = Rgb565::CSS_GREEN;
                style.text_color = Rgb565::CSS_BLACK;

                ui.add(
                    IconButton::<size32px::audio::MicAdd>::new_from_type(&mut ib2)
                        .smartstate(smartstates.next()),
                );

                Ok(())
            })
                .unwrap();

            Ok(())
        })
            .unwrap();
        */

        /*
        ui.central_centered_panel_ui(280, 200, |ui| {
            if smartstates.peek().is_empty() {
                ui.clear_background().ok();
            }
            ui.style_mut().icon_color = Rgb565::RED;
            ui.add(IconWidget::<size32px::actions::WarningTriangle>::new_from_type().smartstate(smartstates.next()));
            ui.add(Label::new("Caution!").with_font(ascii::FONT_8X13_BOLD).smartstate(smartstates.next()));
            ui.add(Label::new("This is heavy equipment.\nIf you are not sure what \nexactly you are doing,\nyou might hurt yourself badly.\n").smartstate(smartstates.next()));

            if ui.add(Button::new("I know what I am doing").smartstate(smartstates.next())).clicked() {

            }

            Ok(())
        }).unwrap();

         */

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
