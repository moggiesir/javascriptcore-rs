use Context;
use Value;
use glib::GString;
use glib::ObjectExt;
use glib::StaticType;
use glib::object::ObjectType as ObjectType_;
use glib::translate::*;
use gobject_sys;
use glib_sys::{gpointer, GPtrArray};
use javascriptcore_sys;
use std::borrow::Borrow;
use std::fmt;
use std::ops::{Deref, DerefMut};
use glib::object::IsA;

use javascriptcore_sys::{JSCClass, JSCValue};

use std::boxed::Box as Box_;

glib_wrapper! {
    pub struct _NativeClass(Object<JSCClass, javascriptcore_sys::JSCClassClass, ClassClass>);

    match fn {
        get_type => || javascriptcore_sys::jsc_class_get_type(),
    }
}

pub struct Class<T> {
    native: _NativeClass,
    phantom: std::marker::PhantomData<*const T>,
}

impl<T> From<_NativeClass> for Class<T> {
    fn from(native: _NativeClass) -> Self {
        Class{native, phantom: std::marker::PhantomData}
    }
}

impl<T> Deref for Class<T> {
    type Target = _NativeClass;

    fn deref(&self) -> &Self::Target {
        &self.native
    }
}

impl<T> DerefMut for Class<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.native
    }
}

impl<T> FromGlibPtrNone<*mut JSCClass> for Class<T> {
    unsafe fn from_glib_none(ptr: *mut JSCClass) -> Self {
        Class::<T>::from(_NativeClass::from_glib_none(ptr))
    }
}

impl<T> FromGlibPtrBorrow<*mut JSCClass> for Class<T> {
    unsafe fn from_glib_borrow(ptr: *mut JSCClass) -> Self {
        Class::<T>::from(_NativeClass::from_glib_borrow(ptr))
    }
}

impl<T> Class<T> {

    pub fn get_name(&self) -> Option<GString> {
        unsafe {
            from_glib_none(javascriptcore_sys::jsc_class_get_name(self.native.to_glib_none().0))
        }
    }

    pub fn get_parent(&self) -> Option<Class<T>> {
        unsafe {
            Some(from_glib_none(javascriptcore_sys::jsc_class_get_parent(self.native.to_glib_none().0)))
        }
    }

    pub fn get_property_context(&self) -> Option<Context> {
        unsafe {
            let mut value = glib::Value::from_type(<Context as StaticType>::static_type());
            gobject_sys::g_object_get_property(self.native.as_ptr() as *mut gobject_sys::GObject, b"context\0".as_ptr() as *const _, value.to_glib_none_mut().0);
            value.get()
        }
    }
}

impl<T> fmt::Display for Class<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Class")
    }
}

pub trait ClassExt<T: Sized>: 'static {
    fn add_constructor<P: Fn(Vec<Value>) -> T + 'static>(&self, name: Option<&str>, callback: P, return_type: glib::types::Type) -> Value where Self: Sized;
    fn add_method<P: Fn(&mut T, Vec<Value>) -> Value + 'static>(&self, name: &str, callback: P);
    fn add_property(&self, name: &str, getter: Option<Box<dyn Fn(&mut T) -> Value + 'static>>, setter: Option<Box<dyn Fn(&mut T, &Value) + 'static>>);
}

impl<T: 'static> ClassExt<T> for Class<T> {

    fn add_constructor<P: Fn(Vec<Value>) -> T + 'static>(&self, name: Option<&str>, callback: P, return_type: glib::types::Type) -> Value where Self: Sized {
        let callback_data: Box_<P> = Box::new(callback);
        unsafe extern "C" fn callback_func<T, P: Fn(Vec<Value>) -> T + 'static>(args: *mut GPtrArray, user_data: gpointer) -> gpointer {
            let callback: &P = &*(user_data as *mut _);
            glib_sys::g_ptr_array_ref(args as _);
            let a = *args;
            let v = Vec::from_raw_parts(a.pdata, a.len as usize, a.len as usize);
            let v2: Vec<Value> = v.iter().map(|&val| {
                let vptr: *mut javascriptcore_sys::JSCValue = val as _;
                let r: Value = from_glib_none(vptr);
                r
            }).collect();
            let ret = (*callback)(v2);
            let b = Box::new(ret);
            Box::into_raw(b) as _
        }
        let callback = Some(callback_func::<T, P> as _);
        unsafe extern "C" fn destroy_notify_func<T, P: Fn(Vec<Value>) -> T + 'static>(data: glib_sys::gpointer) {
            let _callback: Box_<P> = Box_::from_raw(data as *mut _);
        }
        let destroy_call4 = Some(destroy_notify_func::<T, P> as _);
        let super_callback0: Box_<P> = callback_data;
        unsafe {
            let class: &_NativeClass = self.native.as_ref();
            from_glib_full(javascriptcore_sys::jsc_class_add_constructor_variadic(class.to_glib_none().0, name.to_glib_none().0, callback, Box::into_raw(super_callback0) as *mut _, destroy_call4, return_type.to_glib()))
        }
    }

    fn add_method<P: Fn(&mut T, Vec<Value>) -> Value + 'static>(&self, name: &str, callback: P) {
        let callback_data: Box_<P> = Box::new(callback);
        unsafe extern "C" fn callback_func<T, P: Fn(&mut T, Vec<Value>) -> Value + 'static>(instance: gpointer, args: *mut GPtrArray, user_data: gpointer) -> *mut JSCValue {
            let ptr: Box<T> = Box::from_raw(instance as _);
            let this: &mut T = Box::leak(ptr);
            glib_sys::g_ptr_array_ref(args as _);
            let a = *args;
            let v = Vec::from_raw_parts(a.pdata, a.len as usize, a.len as usize);
            let v2: Vec<Value> = v.iter().map(|&val| {
                let vptr: *mut javascriptcore_sys::JSCValue = val as _;
                let r: Value = from_glib_none(vptr);
                r
            }).collect();
            let callback: &P = &*(user_data as *mut _);
            let ret = (*callback)(this, v2);
            let res: *mut JSCValue = ret.to_glib_full();
            return res;
        }
        let callback = Some(callback_func::<T, P> as _);
        unsafe extern "C" fn destroy_notify_func<T, P: Fn(&mut T, Vec<Value>) -> Value + 'static>(data: glib_sys::gpointer) {
            let _callback: Box_<P> = Box_::from_raw(data as *mut _);
        }
        let destroy_call4 = Some(destroy_notify_func::<T, P> as _);
        let super_callback0: Box_<P> = callback_data;
        unsafe {
            let class: &_NativeClass = self.native.as_ref();
            javascriptcore_sys::jsc_class_add_method_variadic(class.to_glib_none().0, name.to_glib_none().0, callback, Box::into_raw(super_callback0) as *mut _, destroy_call4, Value::static_type().to_glib());
        }
    }

    fn add_property(&self, name: &str, getter: Option<Box<dyn Fn(&mut T) -> Value + 'static>>, setter: Option<Box<dyn Fn(&mut T, &Value) + 'static>>) {
        let getter_data: Option<Box<dyn Fn(&mut T) -> Value + 'static>> = getter;
        unsafe extern "C" fn getter_func<T>(instance: gpointer, user_data: glib_sys::gpointer) -> *mut javascriptcore_sys::JSCValue {
            let ptr: Box<T> = Box::from_raw(instance as _);
            let this: &mut T = Box::leak(ptr);
            //let callback: &Box_<(Option<Box<dyn Fn(&mut T) -> Value + 'static>>, Option<Box<dyn Fn(&mut T, &Value) + 'static>>)> = &*(user_data as *mut _);
            let callback_box: Box<(Option<Box<dyn Fn(&T) -> Value + 'static>>, Option<Box<dyn Fn(&T, &Value) + 'static>>)> = Box::from_raw(user_data as _);
            let callback = Box::leak(callback_box);
            let res = if let Some(ref callback) = callback.0 {
                callback(this)
            } else {
                panic!("cannot get closure...")
            };
            res/*Not checked*/.to_glib_full()
        }
        let getter = if getter_data.is_some() { Some(getter_func::<T> as _) } else { None };
        let setter_data: Option<Box<dyn Fn(&mut T, &Value) + 'static>> = setter;
        unsafe extern "C" fn setter_func<T>(instance: gpointer, value: *mut javascriptcore_sys::JSCValue, user_data: glib_sys::gpointer) {
            let ptr: Box<T> = Box::from_raw(instance as _);
            let this: &mut T = Box::leak(ptr);
            let value: Value = from_glib_borrow(value);
            //let callback: &Box_<(&Option<Box<dyn Fn(&T) -> Value + 'static>>, &Option<Box<dyn Fn(&T, &Value) + 'static>>)> = &*(user_data as *mut _);
            let callback_box: Box<(Option<Box<dyn Fn(&T) -> Value + 'static>>, Option<Box<dyn Fn(&T, &Value) + 'static>>)> = Box::from_raw(user_data as _);
            let callback = Box::leak(callback_box);
            if let Some(ref cb) = callback.1 {
                cb(this, &value);
            } else {
                panic!("cannot get closure...")
            };
        }
        let setter = if setter_data.is_some() { Some(setter_func::<T> as _) } else { None };
        unsafe extern "C" fn destroy_notify_func<T>(data: glib_sys::gpointer) {
            let _callback: Box_<(&Option<Box<dyn Fn(&mut T) -> Value + 'static>>, &Option<Box<dyn Fn(&mut T, &Value) + 'static>>)> = Box::from_raw(data as *mut _);
        }
        let destroy_call6 = Some(destroy_notify_func::<T> as _);
        let super_callback0: Box_<(Option<Box<dyn Fn(&mut T) -> Value + 'static>>, Option<Box<dyn Fn(&mut T, &Value) + 'static>>)> = Box::new((getter_data, setter_data));
        unsafe {
            let class: &_NativeClass = self.native.as_ref();
            javascriptcore_sys::jsc_class_add_property(class.to_glib_none().0, name.to_glib_none().0, Value::static_type().to_glib(), getter, setter, Box::into_raw(super_callback0) as *mut _, destroy_call6);
        }
    }
}

