use glib::object::IsA;
use glib::object::ObjectType as ObjectType_;
use glib::translate::*;
use glib::GString;
use glib::Value as GValue;
use glib::ObjectExt;
use glib::StaticType;
use glib::Type;
use glib::TypedValue;
use glib_sys::{gpointer, GPtrArray};
use gobject_sys;
use libc::c_void;
use std::borrow::Borrow;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use Context;
use Value;

use javascriptcore_sys::{JSCClass, JSCValue};

use std::boxed::Box as Box_;

glib_wrapper! {
    pub struct NativeClass(Object<JSCClass, javascriptcore_sys::JSCClassClass, ClassClass>);

    match fn {
        get_type => || javascriptcore_sys::jsc_class_get_type(),
    }
}

pub struct Class<T> {
    native: NativeClass,
    pub class_list: Vec<MetaClass>,
    phantom: PhantomData<*mut T>,
}

/*impl<T> From<NativeClass> for Class<T> {
    fn from(native: NativeClass) -> Self {
        Class{native, metadata: None, phantom: PhantomData}
    }
}*/

impl<T> Class<T> {
    pub fn wrap(native: NativeClass, class_list: Vec<MetaClass>) -> Self {
        Class {
            native,
            class_list,
            phantom: PhantomData,
        }
    }
}

impl<T> Deref for Class<T> {
    type Target = NativeClass;

    fn deref(&self) -> &Self::Target {
        &self.native
    }
}

impl<T> DerefMut for Class<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.native
    }
}

/*impl<T> FromGlibPtrNone<*mut JSCClass> for Class<T> {
    unsafe fn from_glib_none(ptr: *mut JSCClass) -> Self {
        Class::<T>::wrap(NativeClass::from_glib_none(ptr))
    }
}

impl<T> FromGlibPtrBorrow<*mut JSCClass> for Class<T> {
    unsafe fn from_glib_borrow(ptr: *mut JSCClass) -> Self {
        Class::<T>::wrap(NativeClass::from_glib_borrow(ptr))
    }
}*/

#[derive(Clone, Debug)]
pub struct MetaClass {
    class: *mut javascriptcore_sys::JSCClass,
    upcast: fn(*mut c_void) -> *mut c_void,
}

impl MetaClass {
    pub fn new(
        class: *mut javascriptcore_sys::JSCClass,
        upcast: fn(*mut c_void) -> *mut c_void,
    ) -> Self {
        MetaClass { class, upcast }
    }
}

// TODO: make private?
pub struct Instance {
    pub wrapped: *mut c_void,
    pub class_list: Vec<MetaClass>,
}

pub trait ChildOf<T> {
    fn as_parent<'a>(&'a self) -> &'a T;
}

pub unsafe fn unwrap(this: &Instance, target: *mut javascriptcore_sys::JSCClass) -> *mut c_void {
    let mut result: *mut c_void = this.wrapped;
    for i in (0..this.class_list.len()).rev() {
        let class = this.class_list.get(i);
        if class.unwrap().class == target {
            return result;
        }
        result = (class.unwrap().upcast)(result);
    }
    std::ptr::null_mut()
}

pub trait ClassConstructor<T> {
    fn add_to(self, &Class<T>, Option<&str>) -> Value;
}

pub struct Constructor<T, P: Fn(Vec<Value>) -> T + 'static> {
    exported: javascriptcore_sys::JSCConstructor,
    func: P,
}

unsafe extern "C" fn constructor_trampoline<T, P: Fn(Vec<Value>) -> T + 'static>(
    args: *mut GPtrArray,
    user_data: gpointer,
) -> gpointer {
    let data_wrap: Box<(P, Vec<MetaClass>)> = Box::from_raw(user_data as _);
    let data_wrap = Box::leak(data_wrap);
    let callback: &P = &data_wrap.0;
    glib_sys::g_ptr_array_ref(args);
    let value_ptrs = Vec::from_raw_parts((*args).pdata, (*args).len as _, (*args).len as _);
    let values: Vec<Value> = value_ptrs
        .iter()
        .map(|&val| from_glib_none(val as *mut javascriptcore_sys::JSCValue))
        .collect();
    let ret = (*callback)(values);

    let b = Box::new(ret);

    let raw = Box::into_raw(b);
    let i = Instance {
        wrapped: raw as _,
        class_list: data_wrap.1.to_vec(),
    };
    let instance_box = Box::new(i);
    return Box::into_raw(instance_box) as _;
}

unsafe extern "C" fn constructor_destroy<T, P: Fn(Vec<Value>) -> T + 'static>(
    data: glib_sys::gpointer,
) {
    let _callback: Box<P> = Box::from_raw(data as *mut _);
}

impl<T, P: Fn(Vec<Value>) -> T + 'static> Constructor<T, P> {
    pub fn new(f: P) -> Constructor<T, P> {
        Constructor {
            exported: Some(constructor_trampoline::<T, P>),
            func: f,
        }
    }
}

impl<T, P: Fn(Vec<Value>) -> T + 'static> ClassConstructor<T> for Constructor<T, P> {
    fn add_to(self, class: &Class<T>, name: Option<&str>) -> Value {
        let callback_data: Box<(P, &Vec<MetaClass>)> = Box::new((self.func, &class.class_list));
        unsafe {
            let class: &NativeClass = class.native.as_ref();
            from_glib_full(javascriptcore_sys::jsc_class_add_constructor_variadic(
                class.to_glib_none().0,
                name.to_glib_none().0,
                self.exported,
                Box::into_raw(callback_data) as *mut _,
                Some(constructor_destroy::<T, P> as _),
                glib::types::Type::Pointer.to_glib(),
            ))
        }
    }
}

impl<T, P: Fn(Vec<Value>) -> T + 'static> ClassConstructor<T> for P {
    fn add_to(self, class: &Class<T>, name: Option<&str>) -> Value {
        let callback_data: Box<(P, Vec<MetaClass>)> = Box::new((self, class.class_list.to_vec()));
        unsafe {
            from_glib_full(javascriptcore_sys::jsc_class_add_constructor_variadic(
                (&class.native).to_glib_none().0,
                name.to_glib_none().0,
                Some(constructor_trampoline::<T, P> as _),
                Box::into_raw(callback_data) as *mut _,
                Some(constructor_destroy::<T, P> as _),
                glib::types::Type::Pointer.to_glib(),
            ))
        }
    }
}

impl<T> Class<T> {
    pub fn get_name(&self) -> Option<GString> {
        unsafe {
            from_glib_none(javascriptcore_sys::jsc_class_get_name(
                self.native.to_glib_none().0,
            ))
        }
    }

    pub fn get_parent(&self) -> Option<NativeClass> {
        unsafe {
            from_glib_none(javascriptcore_sys::jsc_class_get_parent(
                self.native.to_glib_none().0,
            ))
        }
    }

    pub fn get_property_context(&self) -> Option<Context> {
        unsafe {
            let mut value = glib::Value::from_type(<Context as StaticType>::static_type());
            gobject_sys::g_object_get_property(
                self.native.as_ptr() as *mut gobject_sys::GObject,
                b"context\0".as_ptr() as *const _,
                value.to_glib_none_mut().0,
            );
            value.get()
        }
    }
}

impl<T> fmt::Display for Class<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Class")
    }
}

pub trait Returnable<T> {
    fn return_type() -> Type;
    unsafe fn to_glib_or_full(self, a: *mut c_void) -> T;
}

impl Returnable<u32> for u32 {
    fn return_type() -> Type {
        Type::U32
    }
    unsafe fn to_glib_or_full(self, _a: *mut c_void) -> Self {
        self
    }
}

impl Returnable<i32> for i32 {
    fn return_type() -> Type {
        Type::I32
    }
    unsafe fn to_glib_or_full(self, _a: *mut c_void) -> Self {
        self
    }
}

impl Returnable<*const JSCValue> for Value {
    fn return_type() -> Type {
        Value::static_type()
    }
    unsafe fn to_glib_or_full(self, _a: *mut c_void) -> *const JSCValue {
        self.to_glib_full()
    }
}

impl<'a, T> Returnable<*const c_void> for &'a T {
    fn return_type() -> Type {
        Type::Pointer
    }
    unsafe fn to_glib_or_full(self, a: *mut c_void) -> *const c_void {
        a as _
    }
}

impl<'a, T> Returnable<*mut c_void> for &'a mut T {
    fn return_type() -> Type {
        Type::Pointer
    }
    unsafe fn to_glib_or_full(self, a: *mut c_void) -> *mut c_void {
        a
    }
}

pub trait ClassExt<T: Sized>: 'static {
    fn add_constructor<C: ClassConstructor<T>>(&self, name: Option<&str>, cons: C) -> Value;
    fn add_method<'a, V, R: Returnable<V>, P: Fn(&'a mut T, Vec<Value>) -> R + 'static>(&self, name: &str, callback: P) where T: 'a;
    fn add_property(
        &self,
        name: &str,
        getter: Option<Box<dyn Fn(&mut T) -> Value + 'static>>,
        setter: Option<Box<dyn Fn(&mut T, &Value) + 'static>>,
    );
}

impl<T: 'static> ClassExt<T> for Class<T> {
    fn add_constructor<C: ClassConstructor<T>>(&self, name: Option<&str>, cons: C) -> Value {
        cons.add_to(self, name)
    }

    // TODO: Make "this" not mut - it's not thread safe. If mut is needed, need to use Mutex.
    fn add_method<'a, V, R: Returnable<V>, P: Fn(&'a mut T, Vec<Value>) -> R + 'static>(&self, name: &str, callback: P) where T: 'a {
        let class: &NativeClass = self.native.as_ref();
        let callback_data: Box_<(P, *mut JSCClass)> = Box::new((callback, class.to_glib_none().0));
        unsafe extern "C" fn callback_func<'a, T: 'a, V, R: Returnable<V>, P: Fn(&'a mut T, Vec<Value>) -> R + 'static>(
            instance: gpointer,
            args: *mut GPtrArray,
            user_data: gpointer,
        ) -> V {
            glib_sys::g_ptr_array_ref(args as _);
            let a = *args;
            let v = Vec::from_raw_parts(a.pdata, a.len as usize, a.len as usize);
            let v2: Vec<Value> = v
                .iter()
                .map(|&val| from_glib_none(val as *mut JSCValue))
                .collect();
            let wrapper: Box<(P, *mut JSCClass)> = Box::from_raw(user_data as _);
            let wrapper = Box::leak(wrapper);
            let instance_box: Box<Instance> = Box::from_raw(instance as _);
            let instance_ref = Box::leak(instance_box);
            let casted = unwrap(instance_ref, wrapper.1);
            let ptr: Box<T> = Box::from_raw(casted as _);
            let this: &mut T = Box::leak(ptr);
            let callback: &P = &wrapper.0;
            let ret = (*callback)(this, v2);
            ret.to_glib_or_full(instance)
        }
        let callback_type: unsafe extern "C" fn(*mut libc::c_void, *mut glib_sys::GPtrArray, *mut libc::c_void) -> V
           = callback_func::<'a, T, V, R, P>;
        let callback = unsafe { Some(std::mem::transmute(callback_type)) };
        unsafe extern "C" fn destroy_notify_func<
            'a,
            T: 'a,
            V,
            R: Returnable<V>, P: Fn(&'a mut T, Vec<Value>) -> R + 'static>(
            data: glib_sys::gpointer,
        ) {
            let _callback: Box<(P, *mut JSCClass)> = Box_::from_raw(data as *mut _);
        }
        let destroy_call4 = Some(destroy_notify_func::<'a, T, V, R, P> as _);
        unsafe {
            javascriptcore_sys::jsc_class_add_method_variadic(
                class.to_glib_none().0,
                name.to_glib_none().0,
                callback,
                Box::into_raw(callback_data) as *mut _,
                destroy_call4,
                R::return_type().to_glib(),
            );
        }
    }

    fn add_property(
        &self,
        name: &str,
        getter: Option<Box<dyn Fn(&mut T) -> Value + 'static>>,
        setter: Option<Box<dyn Fn(&mut T, &Value) + 'static>>,
    ) {
        // TODO: Add target class to the box.
        let getter_data: Option<Box<dyn Fn(&mut T) -> Value + 'static>> = getter;
        unsafe extern "C" fn getter_func<T>(
            instance: gpointer,
            user_data: glib_sys::gpointer,
        ) -> *mut javascriptcore_sys::JSCValue {
            let ptr: Box<T> = Box::from_raw(instance as _);
            let this: &mut T = Box::leak(ptr);
            //let callback: &Box_<(Option<Box<dyn Fn(&mut T) -> Value + 'static>>, Option<Box<dyn Fn(&mut T, &Value) + 'static>>)> = &*(user_data as *mut _);
            let callback_box: Box<(
                Option<Box<dyn Fn(&T) -> Value + 'static>>,
                Option<Box<dyn Fn(&T, &Value) + 'static>>,
            )> = Box::from_raw(user_data as _);
            let callback = Box::leak(callback_box);
            let res = if let Some(ref callback) = callback.0 {
                callback(this)
            } else {
                panic!("cannot get closure...")
            };
            res /*Not checked*/
                .to_glib_full()
        }
        let getter = if getter_data.is_some() {
            Some(getter_func::<T> as _)
        } else {
            None
        };
        // TODO: Add target class to the box.
        let setter_data: Option<Box<dyn Fn(&mut T, &Value) + 'static>> = setter;
        unsafe extern "C" fn setter_func<T>(
            instance: gpointer,
            value: *mut javascriptcore_sys::JSCValue,
            user_data: glib_sys::gpointer,
        ) {
            let ptr: Box<T> = Box::from_raw(instance as _);
            let this: &mut T = Box::leak(ptr);
            let value: Value = from_glib_borrow(value);
            //let callback: &Box_<(&Option<Box<dyn Fn(&T) -> Value + 'static>>, &Option<Box<dyn Fn(&T, &Value) + 'static>>)> = &*(user_data as *mut _);
            let callback_box: Box<(
                Option<Box<dyn Fn(&T) -> Value + 'static>>,
                Option<Box<dyn Fn(&T, &Value) + 'static>>,
            )> = Box::from_raw(user_data as _);
            let callback = Box::leak(callback_box);
            if let Some(ref cb) = callback.1 {
                cb(this, &value);
            } else {
                panic!("cannot get closure...")
            };
        }
        let setter = if setter_data.is_some() {
            Some(setter_func::<T> as _)
        } else {
            None
        };
        unsafe extern "C" fn destroy_notify_func<T>(data: glib_sys::gpointer) {
            let _callback: Box_<(
                &Option<Box<dyn Fn(&mut T) -> Value + 'static>>,
                &Option<Box<dyn Fn(&mut T, &Value) + 'static>>,
            )> = Box::from_raw(data as *mut _);
        }
        let destroy_call6 = Some(destroy_notify_func::<T> as _);
        let super_callback0: Box_<(
            Option<Box<dyn Fn(&mut T) -> Value + 'static>>,
            Option<Box<dyn Fn(&mut T, &Value) + 'static>>,
        )> = Box::new((getter_data, setter_data));
        unsafe {
            let class: &NativeClass = self.native.as_ref();
            javascriptcore_sys::jsc_class_add_property(
                class.to_glib_none().0,
                name.to_glib_none().0,
                Value::static_type().to_glib(),
                getter,
                setter,
                Box::into_raw(super_callback0) as *mut _,
                destroy_call6,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ContextExt;
    use ContextExtManual;
    use ExceptionExt;
    use ValueExt;

    #[derive(Debug)]
    struct Foo {
        x: i32,
    }

    struct Bar {
        foo: Foo,
        y: i32,
    }

    impl ChildOf<Foo> for Bar {
        fn as_parent<'a>(&'a self) -> &'a Foo {
            &self.foo
        }
    }

    #[test]
    fn test_add() {
        gtk::init().unwrap();
        let ctx = Context::new();
        let class = ctx.register_class("Foo", None);
        let cons = class.add_constructor(Some("Foo"), |_values| Foo { x: 0 });
        class.add_method("bar", |this, _value| {
            println!("{:?}", this);
            this.x = this.x + 1;
            Value::new_number(&Context::get_current().unwrap(), this.x.into())
        });
        class.add_method("bazel", |this, _values| {
            println!("bazel {:?}", this);
            this
        });
        class.add_property(
            "baz",
            Some(Box::new(|this: &mut Foo| {
                Value::new_number(&Context::get_current().unwrap(), this.x.into())
            })),
            Some(Box::new(|this, value| {
                this.x = value.to_int32().into();
            })),
        );
        ctx.set_value("Foo", &cons);
        ctx.evaluate("var f = new Foo()");
        ctx.evaluate("f.bazel()");
        if ctx.get_exception().is_some() {
            println!("{:?}", ctx.get_exception().unwrap().get_message());
        }
        assert_eq!(ctx.get_exception(), None);
        assert_eq!(ctx.evaluate("f.bazel().bar()").unwrap().to_int32(), 1);
        assert_eq!(ctx.evaluate("f.bar()").unwrap().to_int32(), 2);
        ctx.evaluate("f.baz = 0");
        assert_eq!(ctx.evaluate("f.baz").unwrap().to_int32(), 0);
    }

    #[test]
    fn test_subclass() {
        gtk::init().unwrap();
        let ctx = Context::new();
        let class = ctx.register_class("Foo", None);
        let subclass = ctx.register_subclass("Bar", &class, None);
        let cons = subclass.add_constructor(None, |_values| Bar {foo:Foo{x:5},y:7});
        ctx.set_value("Bar", &cons);
        class.add_method("x", |this, _value| {
            this.x = this.x + 1;
            this.x
        });
        subclass.add_method("y", |this, _value| {
            this.y = this.y + 1;
            Value::new_number(&Context::get_current().unwrap(), this.y.into())
        });
        let result = ctx.evaluate("var bar = new Bar(); bar.x() + bar.y()");
        assert_eq!(result.unwrap().to_int32(), 14);
    }
}
