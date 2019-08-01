use glib::object::IsA;
use glib::translate::*;
use glib_sys::gpointer;
use libc::c_void;
use AsNativeVTable;
use ChildOf;
use Class;
use ClassExt;
use Context;
use MetaClass;
use NativeClass;
use Value;

pub trait ContextExtManual: 'static {
    fn evaluate_in_object<T>(
        &self,
        code: &str,
        object_instance: Option<T>,
        object_class: Option<&Class<T>>,
        uri: &str,
        line_number: u32,
    ) -> (Value, Value);
    fn register_class<T>(&self, name: &str, vtable: Option<&dyn AsNativeVTable<T>>) -> Class<T>;
    fn register_subclass<'a, Child: ChildOf<Parent>, Parent: 'a>(
        &self,
        name: &str,
        parent_class: &Class<Parent>,
        vtable: Option<&dyn AsNativeVTable<Child>>,
    ) -> Class<Child>;
}

impl<O: IsA<Context>> ContextExtManual for O {
    fn evaluate_in_object<T>(
        &self,
        code: &str,
        object_instance: Option<T>,
        object_class: Option<&Class<T>>,
        uri: &str,
        line_number: u32,
    ) -> (Value, Value) {
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
                out_param,
            ));
            (result, from_glib_borrow(new_object))
        }
    }

    fn register_class<T>(&self, name: &str, vtable: Option<&dyn AsNativeVTable<T>>) -> Class<T> {
        unsafe extern "C" fn destroy_notify_func<T>(data: glib_sys::gpointer) {
            let _instance = Box::from_raw(data as *mut T);
        }
        let destroy_call = Some(destroy_notify_func::<T> as _);
        unsafe {
            let b = vtable.map(|v| {
                let boxed = Box::new(v.as_vtable());
                Box::into_raw(boxed) as *mut javascriptcore_sys::JSCClassVTable
            });
            let ptr = javascriptcore_sys::jsc_context_register_class(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                std::ptr::null_mut(),
                b.map_or(std::ptr::null_mut(), |x| x),
                destroy_call,
            );
            let class: NativeClass = from_glib_borrow(ptr);
            Class::wrap(class, vec![MetaClass::new(ptr, |input| input)])
        }
    }

    fn register_subclass<'a, Child: ChildOf<Parent>, Parent: 'a>(
        &self,
        name: &str,
        parent_class: &Class<Parent>,
        vtable: Option<&dyn AsNativeVTable<Child>>,
    ) -> Class<Child> {
        unsafe extern "C" fn destroy_notify_func<T>(data: glib_sys::gpointer) {
            let _instance = Box::from_raw(data as *mut T);
        }
        let destroy_call = Some(destroy_notify_func::<Child> as _);
        let parent = parent_class.to_glib_none().0;
        let mut newvec = parent_class.class_list.to_vec();
        unsafe {
            let b = vtable.map(|v| {
                let boxed = Box::new(v.as_vtable());
                Box::into_raw(boxed) as *mut javascriptcore_sys::JSCClassVTable
            });
            let ptr = javascriptcore_sys::jsc_context_register_class(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                parent,
                b.map_or(std::ptr::null_mut(), |x| x),
                destroy_call,
            );
            newvec.push(MetaClass::new(ptr, capture_as_parent::<Child, Parent>()));
            let class: NativeClass = from_glib_borrow(ptr);
            Class::wrap(class, newvec)
        }
    }
}

fn capture_as_parent<Child: ChildOf<Parent>, Parent>() -> fn(*mut c_void) -> *mut c_void
{
    fn extractor<Child: ChildOf<Parent>, Parent>(input: *mut c_void) -> *mut c_void {
        unsafe {
            let child_ptr: *mut Child = input as _;
            let child: &Child = &*child_ptr;
            let parent: &Parent = child.as_parent();
            let parent_ptr: *const Parent = parent as _;
            parent_ptr as _
        }
    }
    extractor::<Child, Parent> as _
}

#[cfg(test)]
mod tests {
    use super::*;
    use ValueExt;

    #[test]
    fn evaluate_in_object() {
        gtk::init().unwrap();
        let ctx = Context::new();
        let (_res, obj) = ctx.evaluate_in_object::<&str>("var x = 42;", None, None, "", 1);
        assert_eq!(obj.object_get_property("x").unwrap().to_int32(), 42);
    }
}
