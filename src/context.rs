use _NativeClass;
use AsNativeVTable;
use Class;
use ClassExt;
use Context;
use Value;
use glib::object::IsA;
use glib::translate::*;
use glib_sys::gpointer;
use std::boxed::Box as Box_;

pub trait ContextExtManual: 'static {
    fn evaluate_in_object<T>(&self, code: &str, object_instance: Option<T>, object_class: Option<&Class<T>>, uri: &str, line_number: u32) -> (Value, Value);
    fn register_class<T, V: AsNativeVTable>(&self, name: &str, parent_class: Option<&Class<T>>) -> Class<T>;
}

impl<O: IsA<Context>> ContextExtManual for O {
    fn evaluate_in_object<T>(&self, code: &str, object_instance: Option<T>, object_class: Option<&Class<T>>, uri: &str, line_number: u32) -> (Value, Value) {
        let mut new_object: *mut javascriptcore_sys::JSCValue = std::ptr::null_mut() as _;
        let out_param: *mut *mut javascriptcore_sys::JSCValue = &mut new_object as _;
        let length = code.len() as isize;
        let instance: gpointer = object_instance.map_or_else(std::ptr::null_mut, |o| {
            let b = Box::new(o);
            Box::into_raw(b) as _
        });
        unsafe {
            let result: Value = from_glib_full(javascriptcore_sys::jsc_context_evaluate_in_object(
                self.as_ref().to_glib_none().0,
                code.to_glib_none().0,
                length,
                instance,
                object_class.map_or(std::ptr::null_mut(), |o| o.to_glib_none().0),
                uri.to_glib_none().0,
                line_number,
                out_param));
            (result, from_glib_borrow(new_object))
        }
    }

    fn register_class<T, V: AsNativeVTable>(&self, name: &str, parent_class: Option<&Class<T>>) -> Class<T> {
        unsafe extern "C" fn destroy_notify_func<T>(data: glib_sys::gpointer) {
            let _instance = Box_::from_raw(data as *mut T);
        }
        let destroy_call4 = Some(destroy_notify_func::<T> as _);
        unsafe {
            let vtable = V::as_vtable();
            let b = vtable.map(|v| {
                let boxed = Box::new(v);
                Box::into_raw(boxed) as *mut javascriptcore_sys::JSCClassVTable
            });
            let class: _NativeClass = from_glib_borrow(javascriptcore_sys::jsc_context_register_class(self.as_ref().to_glib_none().0,
                name.to_glib_none().0, parent_class.map_or(std::ptr::null_mut(), |p| p.to_glib_none().0), b.map_or(std::ptr::null_mut(), |x| x), destroy_call4));
            Class::<T>::from(class)
        }
    }
}
