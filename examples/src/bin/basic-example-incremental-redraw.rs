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
    // Simulator Setup (ILI9341-like Display)
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

    // Output Settings. Change for other screen appearance / size scaling.
    let output_settings = OutputSettingsBuilder::new()
        // .pixel_spacing(2)
        // .scale(2)
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

    // smartstates (for incremental redrawing)
    // How this works:
    //
    // The widget keeps track of its state in a Smartstate.
    // For example, each visual state (hover, pressed, no interaction) of a button will be saved
    // as a distinct value.
    // With that, the button "knows" when it needs to be redrawn, as its computed state
    // (from interaction and such) is different from the last time it was drawn, and
    // redraws itself only when it needs to do so.
    //
    // One caveat here is that changes in the values passed to the widget (e.g. the label of a button)
    // will not be automatically detected. Therefore, the widget needs to be told when to redraw itself
    // by using the smartstate.force_redraw() method on the appropriate smartstate.
    // In the below example, this is done to the smartstate of the label whenever one of the buttons
    // that change the value of `i` is pressed.

    let mut smartstates = SmartstateProvider::<20 /* number of smartstates. Guess this.*/>::new();

    // Quick note on the performance improvements:
    // Smartstates can increase the framerate (especially on slow screens) by a factor of over 100x.
    // So, especially when you are working on an SPI screen, you should definitely use them.

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

        // Don't clear the UI background, as otherwise all widgets will be cleared with the background.
        // DO NOT: ui.clear_background().ok();

        last_down = mouse_down;

        // Reset the smartstate provider's internal counter to zero at the start (or end) of the loop
        smartstates.restart_counter();

        // === ACTUAL UI CODE STARTS HERE ===

        // Most widgets have a `.smartstate()` method, which takes a smartstate and returns
        // the widget itself, but now with added smart redraw functionality
        ui.add(
            Label::new("Basic Example (incremental)")
                .with_font(ascii::FONT_10X20)
                .smartstate(smartstates.next()),
        );

        // Smartstates of labels are never redrawn, unless they are forced to be.
        ui.add(Label::new("Basic Counter (7LOC)").smartstate(smartstates.next()));

        // Smartstates of buttons are redrawn when they are interacted with.
        if ui
            .add_horizontal(Button::new("-").smartstate(smartstates.next()))
            .clicked()
        {
            i = i.saturating_sub(1);

            // force smartstate of label to be redrawn if the button is clicked
            smartstates.next().force_redraw();
        }
        ui.add_horizontal(
            Label::new(&format!("Clicked {} times", i)).smartstate(smartstates.next()),
        );
        if ui
            .add_horizontal(Button::new("+").smartstate(smartstates.next()))
            .clicked()
        {
            i = i.saturating_add(1);

            smartstates.prev().force_redraw();
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
