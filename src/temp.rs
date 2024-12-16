use crate::smartstate::{Container, Smartstate};
use crate::ui::{GuiResult, Response, Ui, Widget};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::text::renderer::TextRenderer;
use kolibri_embedded_gui::smartstate::{Container, Smartstate};
use kolibri_embedded_gui::ui::{GuiResult, Response, Ui, Widget};

struct SomeWidget<'a> {
    active: &'a mut bool,
    smartstate: Container<'a, Smartstate>,
}

impl<'a> SomeWidget<'a> {
    /* ... */

    // With this function, we can "activate" the smartstate:
    pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
        self.smartstate.set(smartstate);
        self
    }
}

impl Widget for SomeWidget<'_> {
    fn draw<DRAW: DrawTarget<Color = COL>, COL: PixelColor>(
        &mut self,
        ui: &mut Ui<DRAW, COL>,
    ) -> GuiResult<Response> {
        // ... do preparation & space allocation ...

        // decide look (e.g. in this example with the bool `active`)

        // Here's where the smartstate is generally used. First, we get the current ('prev') smartstate:
        let prev = self.smartstate.clone_inner();

        // Then, we'll set a state with a unique (for this widget) id per state:
        let style = if active {
            self.smartstate.modify(|st| *st = Smartstate::state(1));
            PrimitiveStyleBuilder::new()
                .fill_color(ui.style().highlight_item_background_color)
                .stroke_color(ui.style().highlight_border_color)
                .stroke_width(ui.style().highlight_border_width)
        } else {
            self.smartstate.modify(|st| *st = Smartstate::state(2));
            PrimitiveStyleBuilder::new()
                .fill_color(ui.style().item_background_color)
                .stroke_color(ui.style().border_color)
                .stroke_width(ui.style().border_width)
        };

        // At the end, we check whether a redraw is necessary:
        let redraw = self.smartstate.eq_option(&prev);

        /* ... then we redraw if necessary ... */

        Ok(Response::new(iresponse).set_redraw(redraw))
    }
}
