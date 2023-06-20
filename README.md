# Kolibri - A GUI framework made to be as lightweight as its namesake

## What is Kolibri?
Kolibri is an embedded Immediate Mode GUI mini-framework very strongly inspired by [egui](https://docs.rs/egui/latest/egui/).

## Current State
Kolibri is currently in a very early stage of development. It is not yet ready for use in any
non-experimental projects. If you're interested in contributing, feel free to open an issue or
a pull request.

You can also generally find me in the [embedded graphics matrix channel](https://matrix.to/#/#rust-embedded-graphics:matrix.org)
if you have any questions.

> #### Sidenote: What is a Kolibri?
> Kolibri is the german word for Hummingbird, which is a small bird that is very fast and agile. 
> There is also an OS with a similar name (KolibriOS), for similar reasons: It is small and fast.
> This library is in no way associated with the [KolibriOS](https://kolibrios.org/en/) project, but
> I do encourage you to check it out if you're interested.

## Why another GUI framework?

Actually, it's in some ways the first, at least in its very, very specific niche.
The `embedded-graphics` environment is awesome, although fairly low-level. The goal of this
library is to make creating simple to somewhat-complex GUIs trivially easy. There is nothing that
does this,really, at least at the time of writing this library.

### What is this not?
Kolibri is not a high-end GUI framework. It is not meant to be used for creating 
very complicated or super nice-looking. If that's something you're 
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
