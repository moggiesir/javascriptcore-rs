// This file was generated by gir (https://github.com/gtk-rs/gir)
// from gir-files (https://github.com/gtk-rs/gir-files)
// DO NOT EDIT

use glib::translate::*;
use javascriptcore_sys;

bitflags! {
    pub struct ValuePropertyFlags: u32 {
        const CONFIGURABLE = 1;
        const ENUMERABLE = 2;
        const WRITABLE = 4;
    }
}

#[doc(hidden)]
impl ToGlib for ValuePropertyFlags {
    type GlibType = javascriptcore_sys::JSCValuePropertyFlags;

    fn to_glib(&self) -> javascriptcore_sys::JSCValuePropertyFlags {
        self.bits()
    }
}

#[doc(hidden)]
impl FromGlib<javascriptcore_sys::JSCValuePropertyFlags> for ValuePropertyFlags {
    fn from_glib(value: javascriptcore_sys::JSCValuePropertyFlags) -> ValuePropertyFlags {
        skip_assert_initialized!();
        ValuePropertyFlags::from_bits_truncate(value)
    }
}

