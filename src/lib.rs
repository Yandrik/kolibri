#![cfg_attr(not(test), no_std)]
#![allow(clippy::needless_doctest_main)]
#![allow(clippy::doc_nested_refdefs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

pub mod button;
pub mod checkbox;
pub mod combo_box;
// mod icon;
// pub mod icon;
pub mod icon;
pub mod label;
pub mod smartstate;
pub mod spacer;
pub mod style;
// mod temp;
pub mod framebuf;
pub mod helpers;
pub mod iconbutton;
pub mod slider;
pub mod toggle_button;
pub mod toggle_switch;
pub mod ui;

pub mod prelude {
    pub use embedded_iconoir::prelude::*;
}

pub use embedded_iconoir::icons;

pub enum RefOption<'a, T> {
    Some(&'a mut T),
    None,
}

impl<T: Copy> RefOption<'_, T> {
    pub fn copy(&self) -> Option<T> {
        match self {
            RefOption::Some(t) => Some(**t),
            RefOption::None => None,
        }
    }
}

impl<'a, T> RefOption<'a, T> {
    pub fn new(t: &'a mut T) -> Self {
        RefOption::Some(t)
    }
}
