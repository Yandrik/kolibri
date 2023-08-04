use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::geometry::Size;
use embedded_graphics::image::Image;
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::pixelcolor::{PixelColor, Rgb565, RgbColor};
use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, StyledDrawable};
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::Text;
use embedded_graphics_simulator::sdl2::MouseButton;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use kolibri_embedded_gui::button::Button;
use kolibri_embedded_gui::checkbox::Checkbox;
use kolibri_embedded_gui::icon::IconWidget;
use kolibri_embedded_gui::iconbutton::IconButton;
use kolibri_embedded_gui::icons::{size12px, size18px, size24px, size32px};
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::prelude::*;
use kolibri_embedded_gui::smartstate::{Container, Smartstate, SmartstateProvider};
use kolibri_embedded_gui::spacer::Spacer;
use kolibri_embedded_gui::style::{medsize_rgb565_debug_style, medsize_rgb565_style};
use kolibri_embedded_gui::ui::{GuiResult, Interaction, Response, Ui, Widget};
use std::cmp::max;
use std::ops::{Add, Sub};
use std::time::Duration;

#[derive(Debug, Copy, Clone)]
pub enum PistonAction {
    Extend,
    Retract,
    Rest,
}

#[derive(Debug, Copy, Clone)]
pub enum Step {
    Clamp(PistonAction),
    Injection(PistonAction),
    Pushrod(PistonAction),
    Wait(Duration),
}

struct StepWidget<'a> {
    step: &'a mut Step,
    smartstate: Container<'a, Smartstate>,
}

impl<'a> StepWidget<'a> {
    fn new(step: &'a mut Step) -> Self {
        Self {
            step,
            smartstate: Container::empty(),
        }
    }

    fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

enum ButtonPress {
    Up,
    Down,
    Center,
}

impl Widget for StepWidget<'_> {
    fn draw<
        DRAW: DrawTarget<Color = COL>,
        COL: PixelColor,
        CST: TextRenderer<Color = COL> + Clone,
    >(
        &mut self,
        mut ui: &mut Ui<DRAW, COL, CST>,
    ) -> GuiResult<Response> {
        // calc size

        // icons: 12px
        const ICON_SIZE: u32 = 18;

        let height = ICON_SIZE * 2 + ICON_SIZE + ui.style().spacing.item_spacing.height * 2;

        let width = match self.step {
            Step::Wait(dur) => {
                Text::new(
                    &*format!("{}s", dur.as_secs()),
                    Point::zero(),
                    MonoTextStyle::new(&ui.style().default_font, ui.style().text_color),
                )
                .bounding_box()
                .size
                .width
                    + ui.style().spacing.item_spacing.width
                    + ICON_SIZE
            }
            _ => {
                ICON_SIZE * 2 + ui.style().spacing.item_spacing.width // 12px
            }
        };

        let iresponse = ui.allocate_space(Size::new(width, height))?;

        // check for interaction
        let interact = iresponse.interaction;

        let lower_button_start = iresponse.area.size.height as i32 - ICON_SIZE as i32;
        let height_i32 = height as i32;

        let prevstate = self.smartstate.clone_inner();

        let intr = match interact {
            Interaction::Click(pos) | Interaction::Drag(pos) | Interaction::Release(pos) => {
                match pos.y - iresponse.area.top_left.y {
                    0..=16 => {
                        self.smartstate.modify(|sm| sm.set_state(1));
                        Some(ButtonPress::Up)
                    }
                    17..=36 => {
                        // 36 because padding
                        // not quite perfect, but good enough for now. If chain is better
                        self.smartstate.modify(|sm| sm.set_state(2));
                        Some(ButtonPress::Center)
                    }
                    37.. => {
                        self.smartstate.modify(|sm| sm.set_state(3));
                        Some(ButtonPress::Down)
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                self.smartstate.modify(|sm| sm.set_state(0));
                None
            }
        };

        let pressed = matches!(interact, Interaction::Release(_));

        // change (up/down/center)

        if pressed {
            if let Some(intr) = &intr {
                match intr {
                    ButtonPress::Up => match self.step {
                        Step::Wait(dur) => {
                            *dur = dur.add(Duration::from_secs(1));
                        }
                        Step::Clamp(action) | Step::Injection(action) | Step::Pushrod(action) => {
                            *action = match action {
                                PistonAction::Extend => PistonAction::Rest,
                                PistonAction::Rest => PistonAction::Retract,
                                PistonAction::Retract => PistonAction::Extend,
                            }
                        }
                    },
                    ButtonPress::Down => match self.step {
                        Step::Wait(dur) => {
                            *dur = max(dur.sub(Duration::from_secs(1)), Duration::from_secs(1));
                        }
                        Step::Clamp(action) | Step::Injection(action) | Step::Pushrod(action) => {
                            *action = match action {
                                PistonAction::Extend => PistonAction::Retract,
                                PistonAction::Rest => PistonAction::Extend,
                                PistonAction::Retract => PistonAction::Rest,
                            }
                        }
                    },
                    ButtonPress::Center => {
                        // TODO
                    }
                }
            }
        }

        let changed = matches!(intr, Some(ButtonPress::Up) | Some(ButtonPress::Down));

        if !self.smartstate.eq_option(&prevstate) {
            // draw
            ui.start_drawing(&iresponse.area);

            let icon =
                size18px::navigation::NavArrowUp::new(if matches!(intr, Some(ButtonPress::Up)) {
                    ui.style().primary_color
                } else {
                    ui.style().icon_color
                });
            let top_nav = Image::new(
                &icon,
                iresponse.area.top_left
                    + Point::new(((iresponse.area.size.width - ICON_SIZE) / 2) as i32, 0),
            );

            ui.draw(&top_nav)?;

            let col = if matches!(intr, Some(ButtonPress::Center)) {
                ui.style().primary_color
            } else {
                ui.style().icon_color
            };
            let pos = iresponse.area.top_left
                + Point::new(
                    0,
                    ICON_SIZE as i32 + ui.style().spacing.item_spacing.height as i32,
                );

            match self.step {
                Step::Clamp(action) => {
                    // todo: animations::TransitionLeft icon
                    let i = size18px::animations::TransitionLeft::new(col);
                    let icon = Image::new(&i, pos);
                    ui.draw(&icon)?;
                }
                Step::Injection(action) => {
                    // todo: system::Type icon
                    let i = size18px::system::Type::new(col);
                    let icon = Image::new(&i, pos);
                    ui.draw(&icon)?;
                }
                Step::Pushrod(action) => {
                    // todo: actions::MenuScale icon
                    let i = size18px::actions::MenuScale::new(col);
                    let icon = Image::new(&i, pos);
                    ui.draw(&icon)?;
                }
                Step::Wait(dur) => {
                    let i = size18px::activities::Hourglass::new(col);
                    let icon = Image::new(&i, pos);
                    ui.draw(&icon)?;
                }
            }

            let pos = iresponse.area.top_left
                + Point::new(
                    ICON_SIZE as i32 + ui.style().spacing.item_spacing.width as i32,
                    ICON_SIZE as i32 + ui.style().spacing.item_spacing.height as i32,
                );

            match self.step {
                Step::Clamp(action) | Step::Injection(action) | Step::Pushrod(action) => {
                    match action {
                        PistonAction::Extend => {
                            ui.draw(&Image::new(&size18px::actions::Upload::new(col), pos))?
                        }
                        PistonAction::Retract => {
                            ui.draw(&Image::new(&size18px::actions::Download::new(col), pos))?
                        }
                        PistonAction::Rest => {
                            ui.draw(&Image::new(&size18px::actions::Minus::new(col), pos))?
                        }
                    }
                }
                Step::Wait(dur) => {
                    // text
                    let font = ui.style().default_font;
                    let text_style = MonoTextStyle::new(&font, col);
                    let pos = Point::new(
                        ICON_SIZE as i32 + ui.style().spacing.item_spacing.width as i32,
                        ICON_SIZE as i32 + ui.style().spacing.item_spacing.height as i32,
                    ) + iresponse.area.top_left;
                    let val = format!("{}s", dur.as_secs());
                    let mut text = Text::new(&val, pos, text_style);
                    text.position += Point::new(
                        0,
                        ((ICON_SIZE / 2 + text.bounding_box().size.height / 4) as i32),
                    );
                    ui.draw(&text)?;
                }
            }

            ui.draw(&Image::new(
                &size18px::navigation::NavArrowDown::new(
                    if matches!(intr, Some(ButtonPress::Down)) {
                        ui.style().primary_color
                    } else {
                        ui.style().icon_color
                    },
                ),
                iresponse.area.top_left
                    + Point::new(
                        ((iresponse.area.size.width - ICON_SIZE) / 2) as i32,
                        iresponse.area.size.height as i32 - ICON_SIZE as i32,
                    ),
            ))?;

            ui.finalize()?;
        }

        Ok(Response::new(iresponse)
            .set_changed(changed)
            .set_clicked(pressed))
    }
}

fn main() -> Result<(), core::convert::Infallible> {
    // ILI9341-clone like display
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

    let circ = Circle::new(Point::new(100, 100), 50)
        .draw_styled(&PrimitiveStyle::with_stroke(Rgb565::RED, 2), &mut display);

    let output_settings = OutputSettingsBuilder::new()
        // .pixel_spacing(2)
        .scale(2)
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

    let mut step = Step::Clamp(PistonAction::Extend);
    let mut step1 = Step::Injection(PistonAction::Extend);
    let mut step11 = Step::Pushrod(PistonAction::Extend);
    let mut step2 = Step::Wait(Duration::from_secs(2));

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

        // === UI ===

        ui.add_horizontal(StepWidget::new(&mut step).smartstate(smartstates.next()));
        ui.add_horizontal(StepWidget::new(&mut step1).smartstate(smartstates.next()));
        ui.add_horizontal(StepWidget::new(&mut step11).smartstate(smartstates.next()));
        ui.add_horizontal(StepWidget::new(&mut step2).smartstate(smartstates.next()));
        ui.add_horizontal(StepWidget::new(&mut step2).smartstate(smartstates.next()));
        ui.add(StepWidget::new(&mut step2).smartstate(smartstates.next()));

        // === END UI ===

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
