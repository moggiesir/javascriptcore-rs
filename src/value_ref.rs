use glib::translate::*;
use javascriptcore_sys::*;
use std::ops::{Deref, DerefMut};

use GlobalContextRef;

pub struct RefWrapper {
    context: JSGlobalContextRef,
    value: JSValueRef,
}

unsafe fn unwrap(wrapper: *mut RefWrapper) -> JSValueRef {
    (*wrapper).value
}

glib_wrapper! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ValueRef(Shared<RefWrapper>);

    match fn {
        ref => |ptr| {
            if (*ptr).context.is_null() { return; }
            javascriptcore_sys::JSValueProtect((*ptr).context, (*ptr).value)
        },
        unref => |ptr| {
            if (*ptr).context.is_null() { return; }
            javascriptcore_sys::JSValueUnprotect((*ptr).context, (*ptr).value)
        },
    }
}

impl ValueRef {
    pub fn is_boolean(&self, context: &GlobalContextRef) -> bool {
        unsafe { JSValueIsBoolean(context.to_glib_none().0, unwrap(self.to_glib_none().0)) != 0 }
    }

    pub fn is_null(&self, context: &GlobalContextRef) -> bool {
        unsafe { JSValueIsNull(context.to_glib_none().0, unwrap(self.to_glib_none().0)) != 0 }
    }

    pub fn is_undefined(&self, context: &GlobalContextRef) -> bool {
        unsafe { JSValueIsUndefined(context.to_glib_none().0, unwrap(self.to_glib_none().0)) != 0 }
    }

    pub fn is_number(&self, context: &GlobalContextRef) -> bool {
        unsafe { JSValueIsNumber(context.to_glib_none().0, unwrap(self.to_glib_none().0)) != 0 }
    }

    pub fn is_string(&self, context: &GlobalContextRef) -> bool {
        unsafe { JSValueIsString(context.to_glib_none().0, unwrap(self.to_glib_none().0)) != 0 }
    }

    pub fn is_object(&self, context: &GlobalContextRef) -> bool {
        unsafe { JSValueIsObject(context.to_glib_none().0, unwrap(self.to_glib_none().0)) != 0 }
    }

    pub fn is_array(&self, context: &GlobalContextRef) -> bool {
        unsafe { JSValueIsArray(context.to_glib_none().0, unwrap(self.to_glib_none().0)) != 0 }
    }

    pub fn is_date(&self, context: &GlobalContextRef) -> bool {
        unsafe { JSValueIsDate(context.to_glib_none().0, unwrap(self.to_glib_none().0)) != 0 }
    }

    pub fn to_number(&self, context: &GlobalContextRef) -> Option<f64> {
        let mut exception = std::ptr::null_mut();
        let result = unsafe {
            JSValueToNumber(
                context.to_glib_none().0,
                unwrap(self.to_glib_none().0),
                &mut exception,
            )
        };
        if exception.is_null() {
            Some(result)
        } else {
            None
        }
    }

    pub fn to_boolean(&self, context: &GlobalContextRef) -> bool {
        unsafe { JSValueToBoolean(context.to_glib_none().0, unwrap(self.to_glib_none().0)) != 0 }
    }

    pub fn to_string(&self, context: &GlobalContextRef) -> Option<String> {
        unsafe {
            let mut exception = std::ptr::null_mut();
            let jsstring = JSValueToStringCopy(
                context.to_glib_none().0,
                unwrap(self.to_glib_none().0),
                &mut exception,
            );

            if exception.is_null() {
                let cap = JSStringGetMaximumUTF8CStringSize(jsstring);
                let mut buf = Vec::<u8>::with_capacity(cap);
                let len = JSStringGetUTF8CString(jsstring, buf.as_mut_ptr() as _, cap);
                JSStringRelease(jsstring);
                buf.set_len(len - 1);
                String::from_utf8(buf).ok()
            } else {
                None
            }
        }
    }
}

// TODO: Delete these after switching all callers to NativeValueRef.
impl FromGlibPtrNone<JSValueRef> for ValueRef {
    unsafe fn from_glib_none(ptr: JSValueRef) -> Self {
        let mut wrapper = RefWrapper {
            context: std::ptr::null_mut() as _,
            value: ptr,
        };
        let pointer: *mut _ = &mut wrapper;
        from_glib_none(pointer)
    }
}

impl FromGlibPtrFull<JSValueRef> for ValueRef {
    unsafe fn from_glib_full(ptr: JSValueRef) -> Self {
        let mut wrapper = RefWrapper {
            context: std::ptr::null_mut() as _,
            value: ptr,
        };
        let pointer: *mut _ = &mut wrapper;
        from_glib_full(pointer)
    }
}
