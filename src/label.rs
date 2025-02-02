use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiError, GuiResult, Response, Ui, Widget};
use core::hash::BuildHasher;
use core::hash::Hash;
use core::ops::Add;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use foldhash::fast::RandomState;

pub struct Label<'a> {
    text: &'a str,
    font: Option<MonoFont<'a>>,
    smartstate: Container<'a, Smartstate>,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str) -> Label<'a> {
        Label {
            text,
            font: None,
            smartstate: Container::empty(),
        }
    }

    pub fn with_font(mut self, font: MonoFont<'a>) -> Self {
        self.font = Some(font);
        self
    }

    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

impl Widget for Label<'_> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // get size

        let font = if let Some(font) = self.font {
            font
        } else {
            ui.style().default_font
        };

        let mut text = Text::new(
            self.text,
            Point::new(0, 0),
            MonoTextStyle::new(&font, ui.style().text_color),
        );

        let size = text.bounding_box();

        // allocate space

        let iresponse = ui.allocate_space(Size::new(size.size.width, size.size.height))?;

        // move text (center vertically)

        text.translate_mut(iresponse.area.top_left.add(Point::new(
            0,
            (iresponse.area.size.height - size.size.height) as i32 / 2,
        )));
        text.text_style.baseline = Baseline::Top;

        // check smartstate (a bool would work, but this is consistent with other widgets)
        let redraw = !self.smartstate.eq_option(&Some(Smartstate::state(0)));
        self.smartstate.modify(|st| *st = Smartstate::state(0));

        // draw

        if redraw {
            ui.start_drawing(&iresponse.area);
            // clear background if necessary
            if !ui.cleared() {
                ui.clear_area(iresponse.area)?;
            }

            ui.draw(&text)
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw text")))?;

            ui.finalize()?;
        }

        Ok(Response::new(iresponse))
    }
}

/// A hasher for widgets that require hashing of data.
///
/// Make sure to create your hasher outside of the drawing loop, just like you would with a
/// [crate::smartstate::SmartstateProvider].
pub struct Hasher {
    random_state: RandomState,
}

impl Hasher {
    pub fn new() -> Self {
        Self {
            random_state: RandomState::default(),
        }
    }
    pub fn hash<T: Hash + ?Sized>(&self, to_hash: &T) -> u64 {
        self.random_state.hash_one(to_hash)
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HashLabel<'a> {
    text: &'a str,
    font: Option<MonoFont<'a>>,
    smartstate: Container<'a, Smartstate>,
    hasher: &'a Hasher,
}

impl<'a> HashLabel<'a> {
    pub fn new(text: &'a str, smartstate: &'a mut Smartstate, hasher: &'a Hasher) -> Self {
        Self {
            text,
            font: None,
            smartstate: Container::new(smartstate),
            hasher,
        }
    }

    pub fn with_font(mut self, font: MonoFont<'a>) -> Self {
        self.font = Some(font);
        self
    }
}

impl Widget for HashLabel<'_> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // get size

        let font = if let Some(font) = self.font {
            font
        } else {
            ui.style().default_font
        };

        let mut text = Text::new(
            self.text,
            Point::new(0, 0),
            MonoTextStyle::new(&font, ui.style().text_color),
        );

        let size = text.bounding_box();

        // allocate space

        let iresponse = ui.allocate_space(Size::new(size.size.width, size.size.height))?;

        let hash = self.hasher.hash(self.text) as u32;

        let redraw = !self.smartstate.eq_option(&Some(Smartstate::state(hash)));
        self.smartstate.modify(|st| *st = Smartstate::state(hash));

        if redraw {
            // move text (center vertically)

            text.translate_mut(iresponse.area.top_left.add(Point::new(
                0,
                (iresponse.area.size.height - size.size.height) as i32 / 2,
            )));
            text.text_style.baseline = Baseline::Top;

            // check smartstate (a bool would work, but this is consistent with other widgets)

            // draw

            ui.start_drawing(&iresponse.area);
            // clear background if necessary
            if !ui.cleared() {
                ui.clear_area(iresponse.area)?;
            }

            ui.draw(&text)
                .map_err(|_| GuiError::DrawError(Some("Couldn't draw text")))?;

            ui.finalize()?;
        }

        Ok(Response::new(iresponse))
    }
}
