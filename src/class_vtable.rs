// This file was generated by gir (https://github.com/gtk-rs/gir)
// from gir-files (https://github.com/gtk-rs/gir-files)
// DO NOT EDIT

use Class;
use Context;
use Value;
use glib::GString;
use glib::StaticType;
use glib::Value as GValue;
use glib::object::IsA;
use glib::translate::*;
use gobject_sys;
use javascriptcore_sys::{JSCClass, JSCClassVTable, JSCContext, JSCValue};
use std::fmt;

pub trait AsNativeVTable {
    unsafe fn as_vtable() -> Option<JSCClassVTable>;
}

pub trait VTableHandler<T> where Self: std::marker::Sized {
    fn get_property(_class: &Class<T>, _context: &Context, _instance: &T, _name: &str) -> Option<Value> {unimplemented!();}
    fn set_property(_class: &Class<T>, _context: &Context, _instance: &T, _name: &str, _value: &Value) -> bool {unimplemented!();}
    fn has_property(_class: &Class<T>, _context: &Context, _instance: &T, _name: &str) -> bool {unimplemented!();}
    fn delete_property(_class: &Class<T>, _context: &Context, _instance: &T, _name: &str) -> bool {unimplemented!();}
    fn enumerate_properties(_class: &Class<T>, _context: &Context, _instance: &T) -> Option<Vec<String>> {unimplemented!();}
}

pub struct ClassVTable<T> {
    get_property: Option<fn(&Class<T>, &Context, &T, &str) -> Option<Value>>,
    set_property: Option<fn(&Class<T>, &Context, &T, &str, &Value) -> bool>,
    has_property: Option<fn(&Class<T>, &Context, &T, &str) -> bool>,
    delete_property: Option<fn(&Class<T>, &Context, &T, &str) -> bool>,
    enumerate_properties: Option<fn(&Class<T>, &Context, &T) -> Option<Vec<String>>>,
}

pub const DEFAULT_VTABLE: JSCClassVTable = JSCClassVTable {
                    get_property: None,
                    set_property: None,
                    has_property: None,
                    delete_property: None,
                    enumerate_properties: None,
                    _jsc_reserved0: None,
                    _jsc_reserved1: None,
                    _jsc_reserved2: None,
                    _jsc_reserved3: None,
                };

#[macro_export]
macro_rules! native_impl {
    ($vtable:ident, $name:ty, $fn_name:ident, $ret:ty, $glibber:expr) => {
        {
            unsafe extern "C" fn $fn_name(class: *mut JSCClass, context: *mut JSCContext, instance: glib_sys::gpointer, name: *const i8) -> $ret {
                let rusted: Box<$name> = Box::from_raw(instance as _);
                let nameo: Option<String> = glib::translate::from_glib_none(name);
                let ret = <$name>::$fn_name(&glib::translate::from_glib_borrow(class), &glib::translate::from_glib_borrow(context), &rusted, nameo.unwrap().as_str());
                ($glibber)(ret)
            }
            $vtable.$fn_name = Some($fn_name as _);
        }
    }
}

#[macro_export]
macro_rules! js_vtable_handler {
    ($vtable:ident, $name:ty, get_property) => {
        native_impl!($vtable, $name, get_property, *mut JSCValue, |ret: Option<Value>| { ret.to_glib_none().0 });
    };
    ($vtable:ident, $name:ty, set_property) => {
        {
            unsafe extern "C" fn set_property(class: *mut JSCClass, context: *mut JSCContext, instance: glib_sys::gpointer, name: *const i8, value: *mut JSCValue) -> glib_sys::gboolean {
                let rusted: Box<$name> = Box::from_raw(instance as _);
                let nameo: Option<String> = glib::translate::from_glib_none(name);
                let ret = <$name>::set_property(&glib::translate::from_glib_borrow(class), &glib::translate::from_glib_borrow(context), &rusted, nameo.unwrap().as_str(), &glib::translate::from_glib_borrow(value));
                ret.to_glib()
            }
            $vtable.set_property = Some(set_property as _);
        }
    };
    ($vtable:ident, $name:ty, enumerate_properties) => {
        {
            unsafe extern "C" fn enumerate_properties(class: *mut JSCClass, context: *mut JSCContext, instance: glib_sys::gpointer) -> *mut *mut libc::c_char {
                let rusted: Box<$name> = Box::from_raw(instance as _);
                let maybe_v = <$name>::enumerate_properties(&glib::translate::from_glib_borrow(class), &glib::translate::from_glib_borrow(context), &rusted);
                if maybe_v.is_none() { return std::ptr::null_mut(); }
                let mut vec = maybe_v.unwrap();
                let mut strs: Vec<*mut libc::c_char> = vec.iter().map(|item| {
                    item.to_glib_full()
                }).collect();
                vec.shrink_to_fit();
                strs.push(std::ptr::null_mut());
                let ptr = strs.as_mut_ptr();
                std::mem::forget(strs);
                return ptr;
            }
            $vtable.enumerate_properties = Some(enumerate_properties as _);
        }

    };
    ($vtable:ident, $name:ty, $fn_name:ident) => {
        native_impl!($vtable, $name, $fn_name, glib_sys::gboolean, |ret: bool| { ret.to_glib() });
    };
}

#[macro_export]
macro_rules! js_vtable {
    (
        $(#[$attr:meta])*
        impl VTableHandler for $name:ident {
            $(fn $fn_name:ident $method:tt $(-> $ret:ty)* $body:block)*
        }
    ) => {
        impl VTableHandler<$name> for $name {
            $(fn $fn_name $method $(-> $ret)* $body)*
        }

        impl AsNativeVTable for $name {
            unsafe fn as_vtable() -> Option<JSCClassVTable> {
                let mut vtable = JSCClassVTable {
                    ..DEFAULT_VTABLE
                };
                $(js_vtable_handler!(vtable, $name, $fn_name);)*
                return Some(vtable);
            }
        }
    };
}

pub struct Foo();

impl AsNativeVTable for Foo {
    unsafe fn as_vtable() -> Option<JSCClassVTable> {
        None
    }
}

pub const FOO: Foo = Foo();

impl AsNativeVTable for () {
    unsafe fn as_vtable() -> Option<JSCClassVTable> {
        None
    }
}

/*pub struct ClassVTable(pub JSCClassVTable);

impl fmt::Display for ClassVTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Class")
    }
}*/
