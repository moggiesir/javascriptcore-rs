// This file was generated by gir (https://github.com/gtk-rs/gir)
// from gir-files (https://github.com/gtk-rs/gir-files)
// DO NOT EDIT

use glib::translate::*;
use javascriptcore_sys;
use std::fmt;

glib_wrapper! {
    pub struct VirtualMachine(Object<javascriptcore_sys::JSCVirtualMachine, javascriptcore_sys::JSCVirtualMachineClass, VirtualMachineClass>);

    match fn {
        get_type => || javascriptcore_sys::jsc_virtual_machine_get_type(),
    }
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        assert_initialized_main_thread!();
        unsafe {
            from_glib_full(javascriptcore_sys::jsc_virtual_machine_new())
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

pub const NONE_VIRTUAL_MACHINE: Option<&VirtualMachine> = None;

impl fmt::Display for VirtualMachine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtualMachine")
    }
}