# Kolibri - A GUI framework made to be as lightweight as its namesake

## What is Kolibri?
Kolibri is an embedded Immediate Mode GUI mini-framework very strongly inspired by [egui](https://docs.rs/egui/latest/egui/).

## Current State
Kolibri is maturing at a fast pace. Right now, it already has everything you need for a small, basic application. 
It's still in alpha tho, so breaking changes are to be expected for now.

If you're interested in contributing, feel free to open an issue or
a pull request.

You can also generally find me in the [embedded graphics matrix channel](https://matrix.to/#/#rust-embedded-graphics:matrix.org)
if you have any questions.

### Implemented and Planned Features

#### Basic Features

- [x] embedded-graphics based ui rendering
- [x] basic widgets (button, label, checkbox, ...)
- [x] icons
- [x] incremental rendering (only redraw what's actually needed)
- [x] optional buffer-based rendering
- [x] styling 

#### Advanced (and granular)

- [ ] layout
    - [x] right-to-left top-to-bottom layout
    - [ ] aligns (center, right, bottom, ...)
    - [x] side panels (right)
    - [ ] side panels (all sides)
    - [ ] layers (e.g. drawing a modal in front of the ui)


- [ ] styling
    - [x] Styling System
    - [x] Premade Styles for RGB565
    - [ ] Premade Styles for other color types
    - [x] Sub-UIs for editing styles on the fly

- [ ] Widgets
    - [x] Button
    - [x] Label
    - [x] Checkbox
    - [x] Icon
    - [x] Spacer
    - [x] IconButton
    - [ ] ListBox
    - [ ] Something like a ScrollArea
    - [ ] ProgressBar
    - [ ] Toggle
    - [ ] Graph

- [ ] performance
    - [x] heap-less if necessary
    - [x] small buffer to draw everything
    - [x] incremental redraws

- [ ] input
    - [x] generic input system
    - [x] smartstate-reactive basic widgets
    - [ ] custom gestures

**Something missing?** Add an issue with the features you believe would be good.


> #### Sidenote: What is a Kolibri?
> Kolibri is the german word for Hummingbird, which is a small bird that is very fast and agile. 
> There is also an OS with a similar name (KolibriOS), for similar reasons: It is small and fast.
> This library is in no way associated with the [KolibriOS](https://kolibrios.org/en/) project, but
> I do encourage you to check it out if you're interested.

## Why another GUI framework?

Actually, it's in some ways the first, at least in its very, very specific niche.
The `embedded-graphics` environment is awesome, although fairly low-level. The goal of this
library is to make creating simple to somewhat-complex GUIs trivially easy. There is nothing that
does this, really, at least at the time of writing this library.

### What is this not?
Kolibri is not a high-end GUI framework. It is not meant to be used for creating 
very complicated or super nice-looking user interfaces. If that's something you're 
interested in, check out [the rust bindings for lvgl](https://github.com/lvgl/lv_binding_rust/) or 
[slint (not free for commercial use)](https://slint.rs/).


## License

Licensed under either of

* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as 
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
