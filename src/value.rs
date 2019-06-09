use Class;
use Context;
use Value;
use ValueExt;
use ValuePropertyFlags;
use glib;
use glib::GString;
use glib::object::IsA;
use glib::translate::*;
use glib_sys::{gpointer, GPtrArray};
use gobject_sys::GCallback;
use javascriptcore_sys;
use std::boxed::Box as Box_;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::vec::Vec;

pub trait AsJsValue {
    fn as_js_value(&self, context: &Context) -> Value;
}

impl AsJsValue for Value {
    fn as_js_value(&self, _context: &Context) -> Value {
        self.clone()
    }
}

impl AsJsValue for String {
    fn as_js_value(&self, context: &Context) -> Value {
        Value::new_string(context, Some(self))
    }
}

impl AsJsValue for bool {
    fn as_js_value(&self, context: &Context) -> Value {
        Value::new_boolean(context, *self)
    }
}

impl AsJsValue for i32 {
    fn as_js_value(&self, context: &Context) -> Value {
        Value::new_number(context, (*self).into())
    }
}

impl AsJsValue for f64 {
    fn as_js_value(&self, context: &Context) -> Value {
        Value::new_number(context, *self)
    }
}

impl AsJsValue for Vec<&AsJsValue> {
    fn as_js_value(&self, context: &Context) -> Value {
        Value::new_array(context, self)
    }
}

impl AsJsValue for HashMap<&str, &AsJsValue> {
    fn as_js_value(&self, context: &Context) -> Value {
        let obj = Value::new_object::<(), Context>(context, None, None);
        for (k, v) in self {
            obj.object_set_property(k, &v.as_js_value(context));
        }
        return obj;
    }
}

pub struct UndefinedFactory();

impl AsJsValue for UndefinedFactory {
    fn as_js_value(&self, context: &Context) -> Value {
        Value::new_undefined(context)
    }
}

pub const UNDEFINED: UndefinedFactory = UndefinedFactory();

pub struct NullFactory();

impl AsJsValue for NullFactory {
    fn as_js_value(&self, context: &Context) -> Value {
        Value::new_null(context)
    }
}

pub const NULL: NullFactory = NullFactory();

impl Value {
    pub fn new_array<P: IsA<Context>>(context: &P, array: &Vec<&AsJsValue>) -> Value {
        unsafe {
            let free_fn: unsafe extern "C" fn(*mut gobject_sys::GObject) = gobject_sys::g_object_unref as _;
            let free_ptr: *const _ = &free_fn as _;
            let converted: *const unsafe extern "C" fn(glib_sys::gpointer) = free_ptr as _;
            let garray = glib_sys::g_ptr_array_new_full(array.len().try_into().unwrap(), Some(*converted));
            for item in array {
                let v = item.as_js_value(context.as_ref());
                let native_value: *mut javascriptcore_sys::JSCValue = v.to_glib_full();
                glib_sys::g_ptr_array_add(garray, native_value as _);
            }
            let result = from_glib_full(javascriptcore_sys::jsc_value_new_array_from_garray(context.as_ref().to_glib_none().0, garray));
            glib_sys::g_ptr_array_unref(garray);
            return result;
        }
    }

    // The return type should be Option<Value>
    pub fn new_function<P: IsA<Context>, Q: Fn(Vec<Value>) -> Value + 'static>(context: &P, name: Option<&str>, callback: Q, return_type: glib::types::Type) -> Value {
        skip_assert_initialized!();
        let callback_data: Box<Q> = Box::new(callback);
        unsafe extern "C" fn callback_func<P: IsA<Context>, Q: Fn(Vec<Value>) -> Value + 'static>(args: *mut glib_sys::GPtrArray, user_data: glib_sys::gpointer) -> *mut javascriptcore_sys::JSCValue {
            glib_sys::g_ptr_array_ref(args as _);
            let a = *args;
            let v = Vec::from_raw_parts(a.pdata, a.len as usize, a.len as usize);
            let v2: Vec<Value> = v.iter().map(|&val| {
                let vptr: *mut javascriptcore_sys::JSCValue = val as _;
                let r: Value = from_glib_none(vptr);
                r
            }).collect();
            let callback: &Q = &*(user_data as *mut _);
            let r: Value = (*callback)(v2);
            let r3: &Value = r.as_ref() as _;
            let r2: *mut javascriptcore_sys::JSCValue = r3.to_glib_full();
            r2
        }
        let callback = Some(callback_func::<P, Q> as _);
        unsafe extern "C" fn destroy_notify_func<P: IsA<Context>, Q: Fn(Vec<Value>) -> Value + 'static>(data: glib_sys::gpointer) {
            let _callback: Box<Q> = Box::from_raw(data as *mut _);
        }
        let destroy_call4 = Some(destroy_notify_func::<P, Q> as _);
        let super_callback0: Box<Q> = callback_data;
        unsafe {
            from_glib_full(javascriptcore_sys::jsc_value_new_function_variadic(context.as_ref().to_glib_none().0, name.to_glib_none().0, callback, Box::into_raw(super_callback0) as *mut _, destroy_call4, return_type.to_glib()))
        }
    }

    pub fn new_object<T, P: IsA<Context>>(context: &P, object_instance: Option<T>, object_class: Option<&Class<T>>) -> Value {
        let instance: gpointer = object_instance.map_or_else(std::ptr::null_mut, |o| {
            let b = Box::new(o);
            Box::into_raw(b) as _
        });
        unsafe {
            from_glib_full(javascriptcore_sys::jsc_value_new_object(
                context.as_ref().to_glib_none().0,
                instance,
                object_class.map_or(std::ptr::null_mut(), |o| o.to_glib_none().0)))
        }
    }
}

pub trait ValueExtManual: 'static {
    fn object_define_property_accessor(&self, property_name: &str, flags: ValuePropertyFlags, property_type: glib::types::Type, getter: Option<Box<dyn Fn() -> Value + 'static>>, setter: Option<Box<dyn Fn(&Value) -> Value + 'static>>);
}

impl <O: IsA<Value>> ValueExtManual for O {
    fn object_define_property_accessor(&self, property_name: &str, flags: ValuePropertyFlags, property_type: glib::types::Type, getter: Option<Box<dyn Fn() -> Value + 'static>>, setter: Option<Box<dyn Fn(&Value) -> Value + 'static>>) {
        let getter_data: Option<Box<dyn Fn() -> Value + 'static>> = getter;
        unsafe extern "C" fn getter_func(user_data: glib_sys::gpointer) -> *mut javascriptcore_sys::JSCValue {
            let callback: &Box_<(&Option<Box<dyn Fn() -> Value + 'static>>, &Option<Box<dyn Fn(&Value) -> Value + 'static>>)> = &*(user_data as *mut _);
            let res = if let Some(ref callback) = callback.0 {
                callback()
            } else {
                panic!("cannot get closure...")
            };
            res.to_glib_full()
        }
        let getter = if getter_data.is_some() { Some(getter_func as _) } else { None };
        let setter_data: Option<Box<dyn Fn(&Value) -> Value + 'static>> = setter;
        unsafe extern "C" fn setter_func(value: *mut javascriptcore_sys::JSCValue, user_data: glib_sys::gpointer) {
            let value = from_glib_borrow(value);
            let callback: &Box_<(&Option<Box<dyn Fn() -> Value + 'static>>, &Option<Box<dyn Fn(&Value) -> Value + 'static>>)> = &*(user_data as *mut _);
            let _res = if let Some(ref callback) = callback.1 {
                callback(&value);
            } else {
                panic!("cannot get closure...")
            };
        }
        let setter = if setter_data.is_some() { Some(setter_func as _) } else { None };
        unsafe extern "C" fn destroy_notify_func(data: glib_sys::gpointer) {
            let _callback: Box_<(&Option<Box<dyn Fn() -> Value + 'static>>, &Option<Box<dyn Fn(&Value) -> Value + 'static>>)> = Box_::from_raw(data as *mut _);
        }
        let destroy_call7 = Some(destroy_notify_func as _);
        let super_callback0: Box_<(&Option<Box<dyn Fn() -> Value + 'static>>, &Option<Box<dyn Fn(&Value) -> Value + 'static>>)> = Box_::new((&getter_data, &setter_data));
        unsafe {
            javascriptcore_sys::jsc_value_object_define_property_accessor(self.as_ref().to_glib_none().0, property_name.to_glib_none().0, flags.to_glib(), property_type.to_glib(), getter, setter, Box::into_raw(super_callback0) as *mut _, destroy_call7);
        }
    }
}
