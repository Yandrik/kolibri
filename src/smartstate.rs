pub struct Container<'a, T> {
    optional_something: Option<&'a mut T>,
}

impl<'a, T> Container<'a, T> {
    pub fn empty() -> Self {
        Self {
            optional_something: None,
        }
    }

    pub fn new(inner: &'a mut T) -> Self {
        Self {
            optional_something: Some(inner),
        }
    }

    pub fn set(&mut self, val: &'a mut T) {
        self.optional_something = Some(val);
    }

    pub fn modify(&mut self, modify: impl FnOnce(&mut T)) {
        if let Some(inner) = self.optional_something.as_mut() {
            (modify)(*inner);
        }
    }
}

impl<T: Clone> Container<'_, T> {
    pub fn clone_inner(&self) -> Option<T> {
        self.optional_something
            .as_ref()
            .map(|inner| (*inner).clone())
    }
}

impl<T: PartialEq> Container<'_, T> {
    pub fn eq_inner(&self, other: &T) -> bool {
        if let Some(inner) = self.optional_something.as_ref() {
            *inner == other
        } else {
            false
        }
    }

    /// Compare the inner value of this container with an option.
    /// Hereby, all comparisons with None have `false` as the result. Comparisons work like this:
    ///
    /// | self | other | result               |
    /// |------|-------|----------------------|
    /// | Some | Some  | compare inner values |
    /// | Some | None  | false                |
    /// | None | Some  | false                |
    /// | None | None  | false                |
    ///
    pub fn eq_option(&self, other: &Option<T>) -> bool {
        match (self.optional_something.as_ref(), other) {
            (Some(inner), Some(other)) => *inner == other,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
/// Smartstates are used to dynamically redraw widgets. By doing so, there's no need to redraw
/// widgets that haven't changed.
/// Widgets can optionally use smartstates to redraw themselves, or they can just redraw themselves
/// every time.
///
/// ## Caveats
///
/// Smartstate only works if the **background is not cleared** after each iteration.
/// If you want to clear the background anyways, you can use the `Smartstate::force_redraw()` function to
/// guarantee that all smartstate widgets are redrawn.
///
/// ## Usage
///
/// The concept behind smartstates is to give specific "states" to different draw calls. For
/// example, in the button there are states for:
///
/// - Pressed
/// - Hover
/// - None
///
/// SmartStates are designed to be used with `Container`s. This is because the `Container` can
/// massively reduce the pain of an optional `&'a mut`.
///
/// ### Example
///
/// ```rust
/// use kolibri_embedded_gui::smartstate::{Container, Smartstate};
/// # use kolibri_embedded_gui::ui::{GuiResult, InternalResponse, Response, Ui, Widget};
/// # use embedded_graphics::draw_target::DrawTarget;
/// # use embedded_graphics::pixelcolor::PixelColor;
/// # use embedded_graphics::primitives::PrimitiveStyleBuilder;
/// # use embedded_graphics::text::renderer::TextRenderer;
///
/// // Here's the widget that we're going to 'smartstate':
/// struct SomeWidget<'a> {
///     active: &'a mut bool,
///     smartstate: Container<'a, Smartstate>,
/// }
///
/// impl<'a> SomeWidget<'a> {
///     /* ... */
///
///     // With this function, we can "activate" the smartstate:
///     pub fn smartstate(mut self, smartstate: &'a mut Smartstate) -> Self {
///         self.smartstate.set(smartstate);
///         self
///     }
/// }
///
/// impl Widget for SomeWidget<'_> {
///     fn draw<
///         DRAW: DrawTarget<Color = COL>,
///         COL: PixelColor,
///     >(
///         &mut self,
///         ui: &mut Ui<DRAW, COL>,
///     ) -> GuiResult<Response> {
///         // ... do preparation & space allocation ...
///
///         // decide look (e.g. in this example with the bool `active`)
///
///         // Here's where the smartstate is generally used. First, we get the current ('prev') smartstate:
///         let prev = self.smartstate.clone_inner();
///
///         // ...allocate space
///         # let iresponse = InternalResponse::empty();
///
///         // ... derive whether widget is "active" or not
///         # let active = true;
///
///         // Then, we'll set a state with a unique (for this widget) id per state:
///         let style = if active {
///             self.smartstate.modify(|st| *st = Smartstate::state(1));
///             PrimitiveStyleBuilder::new()
///                 // ...
///                 # .fill_color(ui.style().highlight_item_background_color)
///                 # .stroke_color(ui.style().highlight_border_color)
///                 # .stroke_width(ui.style().highlight_border_width)
///                 .build()
///         } else {
///             self.smartstate.modify(|st| *st = Smartstate::state(2));
///             PrimitiveStyleBuilder::new()
///                 // ...
///                 # .fill_color(ui.style().item_background_color)
///                 # .stroke_color(ui.style().border_color)
///                 # .stroke_width(ui.style().border_width)
///                 .build()
///         };
///
///         // At the end, we check whether a redraw is necessary:
///         let redraw = self.smartstate.eq_option(&prev);
///
///         /* ... then we redraw if necessary ... */
///
///         Ok(Response::new(iresponse).set_redraw(redraw))
///     }
/// }
/// ```
///
pub struct Smartstate(u32, bool);

impl Smartstate {
    pub fn empty() -> Self {
        Self(0, false)
    }

    pub fn state(state: u32) -> Self {
        Self(state, true)
    }

    pub fn set_state(&mut self, state: u32) {
        self.0 = state;
        self.1 = true;
    }

    pub fn is_empty(&self) -> bool {
        !self.1
    }

    pub fn is_state(&self, state: u32) -> bool {
        self.1 && self.0 == state
    }

    pub fn force_redraw(&mut self) {
        self.1 = false;
    }
}

impl PartialEq for Smartstate {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 && other.1
    }
}

pub struct SmartstateProvider<const N: usize = 16> {
    states: [Smartstate; N],
    pos: usize,
}

impl<const N: usize> SmartstateProvider<N> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            states: [Smartstate::empty(); N],
            pos: 0,
        }
    }

    #[inline(always)]
    pub fn restart_counter(&mut self) {
        self.pos = 0;
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        N
    }

    pub fn get_pos(&self) -> usize {
        self.pos
    }

    /// Get a smartstate based on a relative position to the current position, without
    /// changing the counter.
    ///
    /// (e.g. get_relative(0) is equivalent to peek(), or get_relative(-2) is equivalent to prev())
    #[inline(always)]
    pub fn get_relative(&mut self, pos: i32) -> &mut Smartstate {
        self.states
            .get_mut((self.pos as i32 + pos) as usize)
            .expect(
                "ERROR: Smartstate Index out of range! Did you call get_relative() before nxt()?",
            )
    }

    #[inline(always)]
    pub fn nxt(&mut self) -> &mut Smartstate {
        let state = self
            .states
            .get_mut(self.pos)
            .expect("ERROR: Smartstate buffer too small! Increase N in SmartstateProvider<N>.");
        self.pos += 1;
        state
    }

    #[inline(always)]
    pub fn current(&mut self) -> &mut Smartstate {
        self.states
            .get_mut(self.pos - 1)
            .expect("ERROR: Smartstate Index out of range! Did you call current() before nxt()?")
    }

    #[inline(always)]
    pub fn prev(&mut self) -> &mut Smartstate {
        self.states
            .get_mut(self.pos - 2)
            .expect("ERROR: Smartstate Index out of range! Did you call prev() before 2 * nxt()?")
    }

    #[inline(always)]
    pub fn peek(&mut self) -> &mut Smartstate {
        self.states
            .get_mut(self.pos)
            .expect("ERROR: Smartstate Index out of range! Did you call peek() at max capacity?")
    }

    #[inline(always)]
    pub fn skip_one(&mut self) {
        self.skip(1);
    }

    #[inline(always)]
    pub fn skip(&mut self, n: usize) {
        self.pos += n;
    }

    #[inline(always)]
    pub fn get(&mut self, pos: usize) -> &mut Smartstate {
        self.states
            .get_mut(pos)
            .expect("ERROR: Invalid index in SmartstateProvider!")
    }

    #[inline(always)]
    pub fn force_redraw_all(&mut self) {
        for state in self.states.iter_mut() {
            state.force_redraw();
        }
    }

    /// Force redraw in all smartstates after (and including) the current position.
    ///
    /// This is useful if you want to force a redraw of all widgets after a certain
    /// point in the UI.
    #[inline(always)]
    pub fn force_redraw_remaining(&mut self) {
        self.force_redraw_from_offset(0);
    }

    /// Force redraw in all smartstates after (and including) the current position plus an offset.
    ///
    ///  For example, `force_redraw_from_offset(0)` is equivalent to `force_redraw_remaining()`.
    ///
    /// This is useful if you want to force a redraw of all widgets after a certain
    /// point in the UI.
    #[inline(always)]
    pub fn force_redraw_from_offset(&mut self, offset: i32) {
        self.force_redraw_range((self.pos as i32 + offset) as usize..N);
    }

    /// Force redraw in all smartstates after (and including) the given **absolute** position.
    ///
    /// For example, `force_redraw_from(0)` is equivalent to `force_redraw_all()`.
    #[inline(always)]
    pub fn force_redraw_from(&mut self, pos: usize) {
        self.force_redraw_range(pos..N);
    }

    /// Force redraw for all states in the given range **relative to the current position**
    #[inline(always)]
    pub fn force_redraw_range_relative(&mut self, range: impl IntoIterator<Item = i32>) {
        for i in range.into_iter().map(|i| self.pos as i32 + i) {
            self.states[i as usize].force_redraw();
        }
    }

    /// Force redraw for all states in the given range
    #[inline(always)]
    pub fn force_redraw_range(&mut self, range: impl IntoIterator<Item = usize>) {
        for i in range.into_iter() {
            self.states[i].force_redraw();
        }
    }
}

impl<const N: usize> Default for SmartstateProvider<N> {
    fn default() -> Self {
        Self::new()
    }
}
