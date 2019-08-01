// Copyright 2013-2017, The Gtk-rs Project Developers.
// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under the MIT license, see the LICENSE file or <http://opensource.org/licenses/MIT>
#![allow(unused_imports)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate glib;
extern crate glib_sys;
extern crate gobject_sys;
extern crate gtk;
extern crate javascriptcore_sys;
extern crate libc;

pub use javascriptcore_sys::*;

macro_rules! assert_initialized_main_thread {
    () => {
        if !::gtk::is_initialized_main_thread() {
            if ::gtk::is_initialized() {
                panic!("GTK may only be used from the main thread.");
            } else {
                panic!("GTK has not been initialized. Call `gtk::init` first.");
            }
        }
    };
}

macro_rules! skip_assert_initialized {
    () => {};
}

mod auto;
mod class;
mod class_vtable;
mod context;
mod function;
mod global_context_ref;
mod value;
mod value_ref;

pub use auto::*;

pub use class::*;
pub use class_vtable::*;
pub use context::*;
pub use global_context_ref::*;
pub use value::*;
pub use value_ref::*;
