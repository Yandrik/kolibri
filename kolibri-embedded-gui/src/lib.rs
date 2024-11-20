#![cfg_attr(not(test), no_std)]

pub mod button;
pub mod checkbox;
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
pub mod ui;

pub mod prelude {
    pub use embedded_iconoir::prelude::*;
}

pub use embedded_iconoir::icons;

pub enum RefOption<'a, T> {
    Some(&'a mut T),
    None,
}

impl<'a, T: Copy> RefOption<'a, T> {
    pub fn copy(&self) -> Option<T> {
        match self {
            RefOption::Some(t) => Some(***&t),
            RefOption::None => None,
        }
    }
}

impl<'a, T> RefOption<'a, T> {
    pub fn new(t: &'a mut T) -> Self {
        RefOption::Some(t)
    }
}
