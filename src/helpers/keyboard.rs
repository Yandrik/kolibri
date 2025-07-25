//! Helper for drawing entire on-screen keyboard to the GUI.
//!
//! This module contains keyboard layouts for different languages and regions (QWERTY, QWERTZ, AZERTY),
//! along with functionality to draw an interactive on-screen keyboard.
use crate::button::Button;
use crate::iconbutton::IconButton;
use crate::smartstate::SmartstateProvider;
use crate::ui::{InternalResponse, Response, Ui};
use embedded_graphics::prelude::*;
use embedded_iconoir::size16px;

use crate::spacer::Spacer;
pub use heapless::String;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Key {
    Char(char, char),
}

impl From<(char, char)> for Key {
    fn from(value: (char, char)) -> Self {
        Self::Char(value.0, value.1)
    }
}

pub type KeyList<'a> = &'a [Key];

pub struct Layout<'a> {
    num_row: KeyList<'a>,
    row_1: KeyList<'a>,
    row_2: KeyList<'a>,
    row_3: KeyList<'a>,
}

impl<'a> Layout<'a> {
    pub const fn new(row_1: KeyList<'a>, row_2: KeyList<'a>, row_3: KeyList<'a>) -> Self {
        Self {
            num_row: &[],
            row_1,
            row_2,
            row_3,
        }
    }

    pub const fn new_with_num_row(
        num_row: KeyList<'a>,
        row_1: KeyList<'a>,
        row_2: KeyList<'a>,
        row_3: KeyList<'a>,
    ) -> Self {
        Self {
            num_row,
            row_1,
            row_2,
            row_3,
        }
    }

    pub const fn qwerty_with_special() -> Self {
        Self {
            num_row: &[
                Key::Char('1', '!'),
                Key::Char('2', '@'),
                Key::Char('3', '#'),
                Key::Char('4', '$'),
                Key::Char('5', '%'),
                Key::Char('6', '^'),
                Key::Char('7', '&'),
                Key::Char('8', '*'),
                Key::Char('9', '('),
                Key::Char('0', ')'),
            ],
            row_1: &[
                Key::Char('q', 'Q'),
                Key::Char('w', 'W'),
                Key::Char('e', 'E'),
                Key::Char('r', 'R'),
                Key::Char('t', 'T'),
                Key::Char('y', 'Y'),
                Key::Char('u', 'U'),
                Key::Char('i', 'I'),
                Key::Char('o', 'O'),
                Key::Char('p', 'P'),
                Key::Char('[', '{'),
                Key::Char(']', '}'),
            ],
            row_2: &[
                Key::Char('a', 'A'),
                Key::Char('s', 'S'),
                Key::Char('d', 'D'),
                Key::Char('f', 'F'),
                Key::Char('g', 'G'),
                Key::Char('h', 'H'),
                Key::Char('j', 'J'),
                Key::Char('k', 'K'),
                Key::Char('l', 'L'),
                Key::Char(';', ':'),
                Key::Char('\'', '@'),
                Key::Char('#', '~'),
            ],
            row_3: &[
                Key::Char('z', 'Z'),
                Key::Char('x', 'X'),
                Key::Char('c', 'C'),
                Key::Char('v', 'V'),
                Key::Char('b', 'B'),
                Key::Char('n', 'N'),
                Key::Char('m', 'M'),
                Key::Char(',', '<'),
                Key::Char('.', '>'),
                Key::Char('/', '?'),
            ],
        }
    }

    pub const fn qwerty_uk_with_special() -> Self {
        Self {
            num_row: &[
                Key::Char('1', '!'),
                Key::Char('2', '"'),
                Key::Char('3', '£'),
                Key::Char('4', '$'),
                Key::Char('5', '%'),
                Key::Char('6', '^'),
                Key::Char('7', '&'),
                Key::Char('8', '*'),
                Key::Char('9', '('),
                Key::Char('0', ')'),
            ],
            row_1: &[
                Key::Char('q', 'Q'),
                Key::Char('w', 'W'),
                Key::Char('e', 'E'),
                Key::Char('r', 'R'),
                Key::Char('t', 'T'),
                Key::Char('y', 'Y'),
                Key::Char('u', 'U'),
                Key::Char('i', 'I'),
                Key::Char('o', 'O'),
                Key::Char('p', 'P'),
                Key::Char('[', '{'),
                Key::Char(']', '}'),
            ],
            row_2: &[
                Key::Char('a', 'A'),
                Key::Char('s', 'S'),
                Key::Char('d', 'D'),
                Key::Char('f', 'F'),
                Key::Char('g', 'G'),
                Key::Char('h', 'H'),
                Key::Char('j', 'J'),
                Key::Char('k', 'K'),
                Key::Char('l', 'L'),
                Key::Char(';', ':'),
                Key::Char('\'', '@'),
                Key::Char('#', '~'),
            ],
            row_3: &[
                Key::Char('z', 'Z'),
                Key::Char('x', 'X'),
                Key::Char('c', 'C'),
                Key::Char('v', 'V'),
                Key::Char('b', 'B'),
                Key::Char('n', 'N'),
                Key::Char('m', 'M'),
                Key::Char(',', '<'),
                Key::Char('.', '>'),
                Key::Char('/', '?'),
            ],
        }
    }

    pub const fn qwertz_with_special() -> Self {
        Self {
            num_row: &[
                Key::Char('1', '!'),
                Key::Char('2', '"'),
                Key::Char('3', '§'),
                Key::Char('4', '$'),
                Key::Char('5', '%'),
                Key::Char('6', '&'),
                Key::Char('7', '/'),
                Key::Char('8', '('),
                Key::Char('9', ')'),
                Key::Char('0', '='),
            ],
            row_1: &[
                Key::Char('q', 'Q'),
                Key::Char('w', 'W'),
                Key::Char('e', 'E'),
                Key::Char('r', 'R'),
                Key::Char('t', 'T'),
                Key::Char('z', 'Z'),
                Key::Char('u', 'U'),
                Key::Char('i', 'I'),
                Key::Char('o', 'O'),
                Key::Char('p', 'P'),
                Key::Char('ü', 'Ü'),
                Key::Char('+', '*'),
            ],
            row_2: &[
                Key::Char('a', 'A'),
                Key::Char('s', 'S'),
                Key::Char('d', 'D'),
                Key::Char('f', 'F'),
                Key::Char('g', 'G'),
                Key::Char('h', 'H'),
                Key::Char('j', 'J'),
                Key::Char('k', 'K'),
                Key::Char('l', 'L'),
                Key::Char('ö', 'Ö'),
                Key::Char('ä', 'Ä'),
                Key::Char('#', '\''),
            ],
            row_3: &[
                Key::Char('y', 'Y'),
                Key::Char('x', 'X'),
                Key::Char('c', 'C'),
                Key::Char('v', 'V'),
                Key::Char('b', 'B'),
                Key::Char('n', 'N'),
                Key::Char('m', 'M'),
                Key::Char(',', ';'),
                Key::Char('.', ':'),
                Key::Char('-', '_'),
            ],
        }
    }

    pub const fn azerty_with_special() -> Self {
        Self {
            num_row: &[
                // can only do non-unicode (ascii) chars
                Key::Char('1', '&'),
                Key::Char('2', '2'),
                Key::Char('3', '"'),
                Key::Char('4', '\''),
                Key::Char('5', '('),
                Key::Char('6', '-'),
                Key::Char('7', '7'),
                Key::Char('8', '_'),
                Key::Char('9', '9'),
                Key::Char('0', '0'),
            ],
            row_1: &[
                Key::Char('a', 'A'),
                Key::Char('z', 'Z'),
                Key::Char('e', 'E'),
                Key::Char('r', 'R'),
                Key::Char('t', 'T'),
                Key::Char('y', 'Y'),
                Key::Char('u', 'U'),
                Key::Char('i', 'I'),
                Key::Char('o', 'O'),
                Key::Char('p', 'P'),
            ],
            row_2: &[
                Key::Char('q', 'Q'),
                Key::Char('s', 'S'),
                Key::Char('d', 'D'),
                Key::Char('f', 'F'),
                Key::Char('g', 'G'),
                Key::Char('h', 'H'),
                Key::Char('j', 'J'),
                Key::Char('k', 'K'),
                Key::Char('l', 'L'),
                Key::Char('m', 'M'),
            ],
            row_3: &[
                Key::Char('w', 'W'),
                Key::Char('x', 'X'),
                Key::Char('c', 'C'),
                Key::Char('v', 'V'),
                Key::Char('b', 'B'),
                Key::Char('n', 'N'),
                Key::Char(',', '?'),
                Key::Char(';', '.'),
                Key::Char(':', '/'),
                Key::Char('!', '§'),
            ],
        }
    }

    pub const fn qwerty() -> Self {
        Self {
            num_row: &[
                Key::Char('1', '!'),
                Key::Char('2', '@'),
                Key::Char('3', '#'),
                Key::Char('4', '$'),
                Key::Char('5', '%'),
                Key::Char('6', '^'),
                Key::Char('7', '&'),
                Key::Char('8', '*'),
                Key::Char('9', '('),
                Key::Char('0', ')'),
            ],
            row_1: &[
                Key::Char('q', 'Q'),
                Key::Char('w', 'W'),
                Key::Char('e', 'E'),
                Key::Char('r', 'R'),
                Key::Char('t', 'T'),
                Key::Char('y', 'Y'),
                Key::Char('u', 'U'),
                Key::Char('i', 'I'),
                Key::Char('o', 'O'),
                Key::Char('p', 'P'),
            ],
            row_2: &[
                Key::Char('a', 'A'),
                Key::Char('s', 'S'),
                Key::Char('d', 'D'),
                Key::Char('f', 'F'),
                Key::Char('g', 'G'),
                Key::Char('h', 'H'),
                Key::Char('j', 'J'),
                Key::Char('k', 'K'),
                Key::Char('l', 'L'),
            ],
            row_3: &[
                Key::Char('z', 'Z'),
                Key::Char('x', 'X'),
                Key::Char('c', 'C'),
                Key::Char('v', 'V'),
                Key::Char('b', 'B'),
                Key::Char('n', 'N'),
                Key::Char('m', 'M'),
            ],
        }
    }

    pub const fn qwerty_uk() -> Self {
        Self {
            num_row: &[
                Key::Char('1', '!'),
                Key::Char('2', '"'),
                Key::Char('3', '£'),
                Key::Char('4', '$'),
                Key::Char('5', '%'),
                Key::Char('6', '^'),
                Key::Char('7', '&'),
                Key::Char('8', '*'),
                Key::Char('9', '('),
                Key::Char('0', ')'),
            ],
            row_1: &[
                Key::Char('q', 'Q'),
                Key::Char('w', 'W'),
                Key::Char('e', 'E'),
                Key::Char('r', 'R'),
                Key::Char('t', 'T'),
                Key::Char('y', 'Y'),
                Key::Char('u', 'U'),
                Key::Char('i', 'I'),
                Key::Char('o', 'O'),
                Key::Char('p', 'P'),
            ],
            row_2: &[
                Key::Char('a', 'A'),
                Key::Char('s', 'S'),
                Key::Char('d', 'D'),
                Key::Char('f', 'F'),
                Key::Char('g', 'G'),
                Key::Char('h', 'H'),
                Key::Char('j', 'J'),
                Key::Char('k', 'K'),
                Key::Char('l', 'L'),
            ],
            row_3: &[
                Key::Char('z', 'Z'),
                Key::Char('x', 'X'),
                Key::Char('c', 'C'),
                Key::Char('v', 'V'),
                Key::Char('b', 'B'),
                Key::Char('n', 'N'),
                Key::Char('m', 'M'),
            ],
        }
    }

    pub const fn qwertz() -> Self {
        Self {
            num_row: &[
                Key::Char('1', '!'),
                Key::Char('2', '"'),
                Key::Char('3', '§'),
                Key::Char('4', '$'),
                Key::Char('5', '%'),
                Key::Char('6', '&'),
                Key::Char('7', '/'),
                Key::Char('8', '('),
                Key::Char('9', ')'),
                Key::Char('0', '='),
            ],
            row_1: &[
                Key::Char('q', 'Q'),
                Key::Char('w', 'W'),
                Key::Char('e', 'E'),
                Key::Char('r', 'R'),
                Key::Char('t', 'T'),
                Key::Char('z', 'Z'),
                Key::Char('u', 'U'),
                Key::Char('i', 'I'),
                Key::Char('o', 'O'),
                Key::Char('p', 'P'),
            ],
            row_2: &[
                Key::Char('a', 'A'),
                Key::Char('s', 'S'),
                Key::Char('d', 'D'),
                Key::Char('f', 'F'),
                Key::Char('g', 'G'),
                Key::Char('h', 'H'),
                Key::Char('j', 'J'),
                Key::Char('k', 'K'),
                Key::Char('l', 'L'),
            ],
            row_3: &[
                Key::Char('y', 'Y'),
                Key::Char('x', 'X'),
                Key::Char('c', 'C'),
                Key::Char('v', 'V'),
                Key::Char('b', 'B'),
                Key::Char('n', 'N'),
                Key::Char('m', 'M'),
            ],
        }
    }

    pub const fn azerty() -> Self {
        Self {
            num_row: &[
                // can only do non-unicode (ascii) chars
                Key::Char('1', '&'),
                Key::Char('2', '2'),
                Key::Char('3', '"'),
                Key::Char('4', '\''),
                Key::Char('5', '('),
                Key::Char('6', '-'),
                Key::Char('7', '7'),
                Key::Char('8', '_'),
                Key::Char('9', '9'),
                Key::Char('0', '0'),
            ],
            row_1: &[
                Key::Char('a', 'A'),
                Key::Char('z', 'Z'),
                Key::Char('e', 'E'),
                Key::Char('r', 'R'),
                Key::Char('t', 'T'),
                Key::Char('y', 'Y'),
                Key::Char('u', 'U'),
                Key::Char('i', 'I'),
                Key::Char('o', 'O'),
                Key::Char('p', 'P'),
            ],
            row_2: &[
                Key::Char('q', 'Q'),
                Key::Char('s', 'S'),
                Key::Char('d', 'D'),
                Key::Char('f', 'F'),
                Key::Char('g', 'G'),
                Key::Char('h', 'H'),
                Key::Char('j', 'J'),
                Key::Char('k', 'K'),
                Key::Char('l', 'L'),
                Key::Char('m', 'M'),
            ],
            row_3: &[
                Key::Char('w', 'W'),
                Key::Char('x', 'X'),
                Key::Char('c', 'C'),
                Key::Char('v', 'V'),
                Key::Char('b', 'B'),
                Key::Char('n', 'N'),
            ],
        }
    }
}

/// Draw a keyboard to the screen using buttons for each key.
/// The keyboard will be drawn at the given position in the given row,
/// and will add / remove characters to / from the given string.
///
/// The keyboard will automatically switch between lower and upper case
/// using the given shift boolean.
///
/// The keyboard will automatically use smartstates to animate the buttons,
/// if the given [SmartstateProvider] is not `None`.
///
/// # Caveats and Considerations
///
/// Drawing the keyboard will take a lot of space. Therefore, if your screen size is limited,
/// use `ui.sub_ui()` to create a sub-ui, and change the style using `ui.style_mut()`. Especially
/// the `button_padding` and `spacing` fields are important to consider, as well as the
/// font (and therefore text size).
///
/// # Arguments
///
/// * `ui`: The `Ui` to draw to.
/// * `layout`: The `Layout` to use for the keyboard.
/// * `smartstates`: The `SmartstateProvider` to use for the keyboard.
///   If `None`, no smartstates will be used.
/// * `draw_num_row`: Whether the number row shall be drawn
/// * `pad`: Whether to pad the rows so that they appear more centered (more like a "real" keyboard)
///   padding generally looks better with lower `button_padding` and `spacing` than standard.
/// * `shift`: The boolean to use for the shift state.
///   If this changes, the returned `response.changed()` will be `true`.
/// * `open`: The boolean to use for the open state. If this is `false`, the keyboard will not be drawn.
///   If this changes, the returned `response.changed()` will be `true`.
/// * `text`: The string to add / remove characters to / from.
///   If this changes, the returned `response.changed()` will be `true`.
///
/// # Returns
///
/// * A `Response` made from an `InternalResponse::empty()`.
///   If a key was pressed, shift was clicked, or a key was erased, `response.changed()` will be `true`.
///   If a key was pressed (irrelevant of changes), `response.clicked()` will be `true`.
#[allow(clippy::too_many_arguments)]
pub fn draw_keyboard<
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
    const M: usize,
    const N: usize,
>(
    ui: &mut Ui<DRAW, COL>,
    layout: &Layout<'_>,
    mut smartstates: Option<&mut SmartstateProvider<M>>,
    draw_num_row: bool,
    pad: bool,
    shift: &mut bool,
    open: &mut bool,
    text: &mut heapless::String<N>,
) -> Response {
    // if open: clear to bottom and draw keyboard
    // if not open: clear to bottom
    // This is only cleared if the smartstates require it

    let redraw = smartstates
        .as_mut()
        .map(|smp| smp.nxt())
        .map(|sm| {
            // if the state is not the same as open, redraw (=> open has changed)
            let redraw = !sm.is_state(*open as u32);
            sm.set_state(*open as u32);
            redraw
        })
        .unwrap_or(true);

    // FIXME: fix redraw after each key press
    if redraw {
        if let Some(smp) = smartstates.as_mut() {
            smp.force_redraw_remaining();
        }
        ui.clear_to_bottom().ok();
    }

    if *open {
    } else {
        return Response::new(InternalResponse::empty());
    }

    // get first *widget* smartstate num
    let first_smartstate_num = smartstates.as_ref().map(|smp| smp.get_pos());

    // chars can be at max 4 bytes long
    let mut buf = [0, 4];

    // padding-related. Saves the previous padding to stagger the keyboard if the padding's the same
    // (and make it look nicer)
    let mut prev_pad = 0;

    let mut clicked = false;
    let mut changed = false;

    if draw_num_row {
        if pad {
            // padding if required (pad from bottom row)
            let padding = layout.row_1.len().saturating_sub(layout.num_row.len()) as u32
                * ui.style().spacing.item_spacing.width * 2 /* use 2 spacings as a button approx */;

            // add raw to prevent the spacer from adding the standard UI spacing
            ui.add_raw(Spacer::new((padding, 0).into())).ok();

            // no prev-pad as this and the next padding cannot be equal
        }

        for Key::Char(l, u) in layout.num_row {
            let btn_char = if *shift { *u } else { *l };
            let mut button = Button::new(btn_char.encode_utf8(&mut buf));

            if let Some(smartstates) = smartstates.as_mut() {
                button = button.smartstate(smartstates.nxt());
            }

            if ui.add_horizontal(button).clicked() {
                clicked = true;
                if text.push(btn_char).is_ok() {
                    changed = true;
                }
            }
        }

        ui.new_row();

        if pad {
            // padding if required (pad based on num row if it's longer)
            let padding = layout.num_row.len().saturating_sub(layout.row_1.len()) as u32
                * ui.style().spacing.item_spacing.width * 2 /* use 2 spacings as a button approx */;

            // add raw to prevent the spacer from adding the standard UI spacing
            ui.add_raw(Spacer::new((padding, 0).into())).ok();

            prev_pad = padding;
        }
    }

    for Key::Char(l, u) in layout.row_1 {
        let btn_char = if *shift { *u } else { *l };
        let mut button = Button::new(btn_char.encode_utf8(&mut buf));

        if let Some(smartstates) = smartstates.as_mut() {
            button = button.smartstate(smartstates.nxt());
        }

        if ui.add_horizontal(button).clicked() {
            clicked = true;
            if text.push(btn_char).is_ok() {
                changed = true;
            }
        }
    }
    if ui
        .add(IconButton::<size16px::navigation::NavArrowLeft>::new_from_type())
        .clicked()
    {
        clicked = true;
        if text.pop().is_some() {
            changed = true;
        }
    }

    // row 2

    if pad {
        // padding if required
        let mut padding = (layout.row_1.len() + 1).saturating_sub(layout.row_2.len()) as u32
            * ui.style().spacing.item_spacing.width * 2 /* use 2 spacings as a button approx */;

        if padding > 0 && prev_pad == padding {
            padding += 2;
        }

        // add raw to prevent the spacer from adding the standard UI spacing
        ui.add_raw(Spacer::new((padding, 0).into())).ok();

        prev_pad = padding;
    }

    for Key::Char(l, u) in layout.row_2 {
        let btn_char = if *shift { *u } else { *l };
        let mut button = Button::new(btn_char.encode_utf8(&mut buf));

        if let Some(smartstates) = smartstates.as_mut() {
            button = button.smartstate(smartstates.nxt());
        }

        if ui.add_horizontal(button).clicked() {
            clicked = true;
            if text.push(btn_char).is_ok() {
                changed = true;
            }
        }
    }

    ui.new_row();

    // row 3

    if pad {
        // padding if required
        let mut padding = layout.row_2.len().saturating_sub(layout.row_3.len()) as u32
            * ui.style().spacing.item_spacing.width * 2 /* use 2 spacings as a button approx */;

        if padding > 0 && prev_pad == padding {
            padding += 2;
        }

        // add raw to prevent the spacer from adding the standard UI spacing
        ui.add_raw(Spacer::new((padding, 0).into())).ok();

        prev_pad = padding;
    }

    for Key::Char(l, u) in layout.row_3 {
        let btn_char = if *shift { *u } else { *l };
        let mut button = Button::new(btn_char.encode_utf8(&mut buf));

        if let Some(smartstates) = smartstates.as_mut() {
            button = button.smartstate(smartstates.nxt());
        }

        if ui.add_horizontal(button).clicked() {
            clicked = true;
            if text.push(btn_char).is_ok() {
                changed = true;
            }
        }
    }

    ui.sub_ui(|ui| {
        if *shift {
            ui.style_mut().item_background_color = ui.style().primary_color;
        }

        if ui
            .add({
                let b = IconButton::<size16px::navigation::NavArrowUp>::new_from_type();
                if let Some(smartstates) = smartstates.as_mut() {
                    b.smartstate(smartstates.nxt())
                } else {
                    b
                }
            })
            .clicked()
        {
            clicked = true;
            changed = true;
            *shift = !*shift;
            if let Some(smartstates) = smartstates.as_mut() {
                // first_smartstate_num is always Some(_) if smartstates is Some(_)
                smartstates.force_redraw_from(first_smartstate_num.unwrap());
            }
        }
        Ok(())
    })
    .unwrap();

    // space row

    if pad {
        // padding if required
        let mut padding = (layout.row_3.len() + 1).saturating_sub(6 /* approx 6 buttons long */) as u32
            * ui.style().spacing.item_spacing.width * 2 /* use 2 spacings as a button approx */;

        if padding > 0 && prev_pad == padding {
            padding += 2;
        }

        // add raw to prevent the spacer from adding the standard UI spacing
        ui.add_raw(Spacer::new((padding, 0).into())).ok();
    }

    // space button
    if ui
        .add_horizontal({
            let b = Button::new("|                |");
            if let Some(smartstates) = smartstates.as_mut() {
                b.smartstate(smartstates.nxt())
            } else {
                b
            }
        })
        .clicked()
    {
        clicked = true;
        if text.push(' ').is_ok() {
            changed = true;
        }
    }

    if ui
        .add({
            let b = IconButton::<size16px::navigation::NavArrowDown>::new_from_type();
            if let Some(smartstates) = smartstates.as_mut() {
                b.smartstate(smartstates.nxt())
            } else {
                b
            }
        })
        .clicked()
    {
        clicked = true;
        changed = true;
        *open = !*open;
        // redraw is automatic
    }

    Response::new(InternalResponse::empty())
        .set_clicked(clicked)
        .set_changed(changed)
        .set_down(redraw)
}
