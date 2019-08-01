use glib::translate::*;
use glib::types::{StaticType, Type};
use glib_sys::gpointer;
use javascriptcore_sys::JSCValue;
use std::convert::TryInto;
use Value;

#[macro_export]
macro_rules! js_function {
    (@convert u32) => {u32};
    (@convert f64) => {f64};
    (@convert String) => {*const libc::c_char};
    (@convert $argty:ident) => {gpointer};

    (@from u32, $arg:ident) => {$arg};
    (@from f64, $arg:ident) => {$arg};
    (@from String, $arg:ident) => {String::from_glib_none($arg)};
    (@from $argty:ident, $arg:ident) => {
        {
            // TODO: the pointer is not guaranteed to be a &T - we need to check the type of the ptr.
            // Also, this is pretending it's a T, but it's really an Instance<Subclass<T>>
            let b: Box<$argty> = Box::from_raw($arg as _);
            Box::leak(b)
        }
    };

    (@map u32) => {u32};
    (@map f64) => {f64};
    (@map String) => {String};
    (@map $argty:ident) => {&$argty};

    // function
    ($ctx:expr, $name:expr, [$($arg:ident: $argty:ident),*], $impl:expr) => {
    {
        fn add_function<T: Fn($(js_function!(@map $argty)),*) -> Value + 'static>(
                ctx: &Context, name: Option<&str>, callback: T) -> Value {
            let v = vec![$(<$argty>::static_type()),*];
            unsafe extern "C" fn callback_func
            <T: Fn($(js_function!(@map $argty)),*) -> Value + 'static>(
                    $($arg: js_function!(@convert $argty)),*, user_data: gpointer) -> *mut JSCValue {
                let callback: &T = &*(user_data as *mut _);
                let result: Value = (*callback)($(js_function!(@from $argty, $arg)),*);
                let result_ref: &Value = result.as_ref();
                let native_result: *mut javascriptcore_sys::JSCValue = result_ref.to_glib_full();
                native_result
            }
            let c: unsafe extern "C" fn(
                    $($arg: js_function!(@convert $argty)),*, user_data: gpointer) -> *mut JSCValue
                = callback_func::<T> as _;
            let b: Box<T> = Box::new(callback);
            unsafe extern "C" fn destroy_notify_func<T>(data: gpointer) {
                let _callback: Box<T> = Box::from_raw(data as *mut _);
            }
            let destroy_call = Some(destroy_notify_func::<T> as _);
            unsafe {
                let gc: gobject_sys::GCallback = Some(std::mem::transmute(c));
                let jscvalue = javascriptcore_sys::jsc_value_new_functionv(
                    ctx.to_glib_none().0,
                    name.to_glib_none().0, gc,
                    Box::into_raw(b) as *mut _,
                    destroy_call,
                    Value::static_type().to_glib(),
                    v.len().try_into().unwrap(),
                    v.to_glib_none().0);
                Value::from_glib_full(jscvalue)
            }
        }
        add_function($ctx, $name, $impl)
    }
    };

    ($ctx:expr, $name:expr, |$($arg:ident: $argty:ident),*| $impl:tt) => {
    {
        js_function!($ctx, $name, [$($arg: $argty),*], |$($arg: js_function!(@map $argty)),*| $impl)
    }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use Context;
    use ContextExtManual;
    use ValueExt;

    #[derive(Debug, Clone)]
    struct Bar {
        x: u32,
    }

    #[derive(Debug, Clone)]
    struct Foo {
        x: Bar,
    }

    impl StaticType for Foo {
        fn static_type() -> Type {
            Type::Pointer
        }
    }

    #[test]
    fn foo() {
        let ctx = Context::new();
        let foo_class = ctx.register_class("Foo", None);
        let func = |u: u32, _: f64, _: String, _: &Foo| {
            return Value::new_number(&Context::get_current().unwrap(), u as _);
        };
        let jsfunc = js_function!(
            &ctx,
            Some("name"),
            [n: u32, f: f64, s: String, foo: Foo],
            func
        );
        let mut output = jsfunc.function_callv(&vec![
            Value::new_number(&ctx, 1 as _),
            Value::new_number(&ctx, 12.0),
            Value::new_string(&ctx, Some("foobar")),
            Value::new_object(&ctx, Some(Foo { x: Bar { x: 24 } }), Some(&foo_class)),
        ]);
        assert_eq!(output.unwrap().to_int32(), 1);
        let jsfunc2 = js_function!(
            &ctx,
            Some("name"),
            |n: u32, _f: f64, _s: String, _foo: Foo| {
                Value::new_number(&Context::get_current().unwrap(), n as _)
            }
        );
        output = jsfunc2.function_callv(&vec![
            Value::new_number(&ctx, 2 as _),
            Value::new_number(&ctx, 12.0),
            Value::new_string(&ctx, Some("foobar")),
            Value::new_object(&ctx, Some(Foo { x: Bar { x: 24 } }), Some(&foo_class)),
        ]);
        assert_eq!(output.unwrap().to_int32(), 2);
    }
}
