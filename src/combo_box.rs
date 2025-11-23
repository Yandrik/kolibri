//! # ComboBox Widget
//!
//! A customizable combo box widget that provides a dropdown selection interface.
//!
//! The combo box widget provides a traditional dropdown control that allows users to select
//! from a list of options. It features an arrow icon that indicates the dropdown functionality
//! and integrates with the framework's theming system for consistent appearance.
//!
//! This widget is part of the Kolibri embedded GUI framework's core widget set and integrates
//! with the framework's [Smartstate] system for efficient rendering.

use crate::button::Button;
use crate::smartstate::{Container, Smartstate};
use crate::style::Style;
use crate::ui::{GuiError, GuiResult, Interaction, Response, Ui, Widget};
use core::cmp::max;
use core::ops::{Add, Sub};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::image::Image;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle};
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::{Baseline, Text};
use embedded_iconoir::prelude::*;
use embedded_iconoir::{size12px, size18px, size24px, size32px};

/// Selector for the combo box widget.
///
/// This enum allows the combo box to select either a text or an index.
pub enum Selector<'a, 'b> {
    Text(&'a mut &'b str),
    Index(&'a mut usize),
}

/// A combo box widget for selecting from a list of options.
///
/// The combo box displays the currently selected text and provides a dropdown menu
/// when clicked, allowing users to choose from available options.
///
/// ## Example
///
/// ```no_run
/// # use embedded_graphics::geometry::Size;
/// # use embedded_graphics::pixelcolor::Rgb565;
/// # use embedded_graphics::prelude::{Point, WebColors};
/// # use embedded_graphics_simulator::sdl2::MouseButton;
/// # use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window,};
/// # use kolibri_embedded_gui::style::medsize_rgb565_style;
/// # use kolibri_embedded_gui::combo_box::{Selector, ComboBox};
/// # use kolibri_embedded_gui::ui::{Interaction, PopupState, Ui};
/// # let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));
/// # let output_settings = OutputSettingsBuilder::new().build();
/// # let mut window = Window::new("Kolibri Example", &output_settings);
///
/// let mut popup_state = PopupState::default(); // declare popup state before ui loop
/// let mut selected = "Hello World";
///
/// loop {
///     let mut popup_buffer = [Rgb565::CSS_BLACK; 16*1024];
///     let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
///     ui.clear_background().ok();
///     
///     ui.begin_popup(&mut popup_state, &mut popup_buffer);
///     
///     if ui.add(ComboBox::new(
///                 Selector::Text(&mut selected),
///                 &["Item 1", "Item 2", "Item 3"],
///             )
///             .with_width(100)
///         )
///         .changed()
///     {
///         println!("ComboBox changed to {}", selected);
///     }
///     
///     ui.end_popup(|| {
///         println!("Popup handled");
///     });
/// }
/// ```
pub struct ComboBox<'a, 'b, 'c>
where
    'b: 'a,
    'c: 'a,
    'c: 'b,
{
    selected: Selector<'a, 'b>,
    smartstate: Container<'a, Smartstate>,
    corner_radius: Option<u32>,
    width: u16,
    contents: &'a [&'c str],
}

impl<'a, 'b, 'c> ComboBox<'a, 'b, 'c>
where
    'b: 'a,
    'c: 'a,
    'c: 'b,
{
    pub fn new(selected: Selector<'a, 'b>, contents: &'a [&'c str]) -> ComboBox<'a, 'b, 'c> {
        ComboBox {
            selected,
            smartstate: Container::empty(),
            corner_radius: None,
            width: 0,
            contents,
        }
    }

    /// Sets the width of the combo box.
    ///
    /// # Arguments
    /// * `width` - The width of the combo box
    ///
    /// # Returns
    /// Self with the specified width
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Adds smartstate support to the combo box for incremental redrawing.
    ///
    /// When a smartstate is provided, the combo box will only redraw when its visual state changes,
    /// significantly improving performance especially on slower displays.
    ///
    /// # Arguments
    /// * `smartstate` - The smartstate to use for tracking the combo box's state
    ///
    /// # Returns
    /// Self with smartstate configured
    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }

    /// Sets a custom corner radius for the combo box.
    ///
    /// If not specified, the combo box will use the corner radius from the UI style.
    ///
    /// # Arguments
    /// * `radius` - The corner radius in pixels
    ///
    /// # Returns
    /// Self with the specified corner radius
    pub fn with_radius(mut self, radius: u32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Calculates the size required to display the combo box with the given text.
    ///
    /// This internal helper method determines the width and height needed to fit the combo box
    /// based on the selected text and the current UI style.
    ///
    /// # Arguments
    /// * `text` - The text to display in the combo box
    /// * `style` - The current UI style
    ///
    /// # Returns
    /// The required size to display the combo box with the given text
    fn get_size<C: PixelColor>(&mut self, text: &str, style: &Style<C>) -> Size {
        let height = style.default_widget_height;
        let padding = style.spacing.button_padding;
        let border = style.border_width;
        let item_spacing_width = style.spacing.item_spacing.width;
        let text_style = MonoTextStyle::new(&style.default_font, style.text_color);

        let text_size = text_style
            .measure_string(text, Point::new(0, 0), Baseline::Top)
            .bounding_box
            .size;
        let height = max(text_size.height + 2 * padding.height + 2 * border, height);
        let min_width = 2 * padding.width + 2 * border + height + item_spacing_width;
        let width = if self.width > 0 {
            max(self.width as u32, min_width)
        } else {
            text_size.width + 2 * padding.width + 2 * border + height + item_spacing_width
        };

        return Size::new(width, height);
    }
}

impl ComboBox<'_, '_, '_> {
    /// Draws the icon for the combo box.
    ///
    /// This internal helper method handles drawing the arrow down icon
    /// It positions the icon in the right of the combo box area
    ///
    /// The icon size is determined by the calling code based on available space.
    fn draw_icon<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
        icon: impl ImageDrawable<Color = COL>,
        area: &Rectangle,
        center_offset: Point,
    ) -> GuiResult<()> {
        let img = Image::new(
            &icon,
            area.top_left.add(
                Point::new(area.size.width as i32 / 2, area.size.height as i32 / 2 - 2)
                    .sub(center_offset),
            ),
        );
        ui.draw(&img)
            .map_err(|_| GuiError::DrawError(Some("Couldn't draw ComboBox icon")))
    }
}

impl Widget for ComboBox<'_, '_, '_> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        let style = ui.style();
        let padding = style.spacing.button_padding;
        let border = style.border_width;
        let side_lenght = style.default_widget_height;
        let item_spacing_width = style.spacing.item_spacing.width;
        let min_width = 2 * padding.width + 2 * border + side_lenght + item_spacing_width;

        let window_border_padding = style.spacing.window_border_padding.width as i32;
        let selected_text = match &self.selected {
            Selector::Text(p_txt) => **p_txt,
            Selector::Index(p_idx) => {
                if **p_idx < self.contents.len() {
                    self.contents[**p_idx]
                } else {
                    ""
                }
            }
        };
        let size = self.get_size(selected_text, &style);
        let mut top_left = ui.get_placer_top_left().add(Point::new(
            window_border_padding,
            window_border_padding + size.height as i32,
        ));
        if top_left.x + size.width as i32 > ui.get_width() as i32 {
            top_left.x = window_border_padding;
            top_left.y += size.height as i32;
        }

        let current_text = if self.width > 0 {
            if self.width > min_width as u16 {
                let text_width = self.width as u32 - min_width;

                crate::style::slice_text_by_width(text_width, selected_text, style)
            } else {
                ""
            }
        } else {
            selected_text
        };
        let cb_size = self.get_size(current_text, style);

        let iresponse = ui.allocate_space(cb_size)?;

        // check for click
        let click = matches!(iresponse.interaction, Interaction::Release(_));
        let down = matches!(
            iresponse.interaction,
            Interaction::Click(_) | Interaction::Drag(_)
        );

        // styles and smartstate
        let prevstate = self.smartstate.clone_inner();

        let rect_style = match iresponse.interaction {
            Interaction::None => {
                self.smartstate.modify(|st| *st = Smartstate::state(1));

                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().border_color)
                    .stroke_width(ui.style().border_width)
                    .fill_color(ui.style().item_background_color)
                    .build()
            }
            Interaction::Hover(_) => {
                self.smartstate.modify(|st| *st = Smartstate::state(2));

                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
                    .fill_color(ui.style().highlight_item_background_color)
                    .build()
            }

            _ => {
                self.smartstate.modify(|st| *st = Smartstate::state(3));

                PrimitiveStyleBuilder::new()
                    .stroke_color(ui.style().highlight_border_color)
                    .stroke_width(ui.style().highlight_border_width)
                    .fill_color(ui.style().primary_color)
                    .build()
            }
        };

        if !self.smartstate.eq_option(&prevstate) {
            let font = ui.style().default_font;

            let mut text = Text::new(
                current_text,
                Point::new(0, 0),
                MonoTextStyle::new(&font, ui.style().text_color),
            );

            // move text
            let mut top_left = iresponse.area.top_left.add(Point::new(
                (padding.width + border) as i32,
                (padding.height + border) as i32,
            ));
            text.translate_mut(top_left);

            text.text_style.baseline = Baseline::Top;

            ui.start_drawing(&iresponse.area);

            let corner_radius = self.corner_radius.unwrap_or(ui.style().corner_radius);
            let rounded_rect = RoundedRectangle::with_equal_corners(
                Rectangle::new(iresponse.area.top_left, iresponse.area.size),
                Size::new(corner_radius, corner_radius),
            );

            ui.draw(&rounded_rect.into_styled(rect_style)).ok();
            ui.draw(&text).ok();

            top_left.x = iresponse.area.top_left.x + cb_size.width as i32
                - (padding.width + border + side_lenght) as i32;
            let icon_size = Size::new(side_lenght, side_lenght);
            match side_lenght {
                0..=18 => {
                    top_left.y += (side_lenght as i32 - 12) / 2;
                    self.draw_icon(
                        ui,
                        size12px::navigation::NavArrowDown::new(ui.style().text_color),
                        &Rectangle::new(top_left, icon_size),
                        Point::new(6, 6),
                    )
                }
                19..=23 => {
                    top_left.y += (side_lenght as i32 - 18) / 2;
                    self.draw_icon(
                        ui,
                        size18px::navigation::NavArrowDown::new(ui.style().text_color),
                        &Rectangle::new(top_left, icon_size),
                        Point::new(9, 9),
                    )
                }
                24..=32 => {
                    top_left.y += (side_lenght as i32 - 24) / 2;
                    self.draw_icon(
                        ui,
                        size24px::navigation::NavArrowDown::new(ui.style().text_color),
                        &Rectangle::new(top_left, icon_size),
                        Point::new(12, 12),
                    )
                }
                _ => {
                    top_left.y += (side_lenght as i32 - 32) / 2;
                    self.draw_icon(
                        ui,
                        size32px::navigation::NavArrowDown::new(ui.style().text_color),
                        &Rectangle::new(top_left, icon_size),
                        Point::new(16, 16),
                    )
                }
            }?;

            ui.finalize()?;
        }
        let mut resp = Response::new(iresponse).set_clicked(click).set_down(down);
        if resp.clicked() || ui.popup_check() {
            let changed = ui
                .popup_draw(top_left, size.width as u16, |popup_ui| {
                    let style = popup_ui.style_mut();
                    style.spacing.item_spacing.height = 0;
                    style.spacing.button_padding.width = 0;
                    style.border_width = 0;
                    style.corner_radius = 0;
                    let item_width =
                        (size.width - 2 * style.spacing.window_border_padding.width) as u16;
                    let mut selected = false;
                    for (idx, item) in self.contents.iter().enumerate() {
                        if popup_ui
                            .add(Button::new(item).with_width(item_width))
                            .clicked()
                        {
                            match &mut self.selected {
                                Selector::Text(p_txt) => **p_txt = *item,
                                Selector::Index(p_idx) => **p_idx = idx,
                            }
                            selected = true;
                        }
                    }
                    selected
                })
                .unwrap_or(false);

            resp = resp.set_changed(changed);
        }
        Ok(resp)
    }
}
