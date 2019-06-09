extern crate glib;
extern crate gtk;
extern crate javascriptcore;

// Probaby don't want this, needed for macros?
use glib::translate::*;

use javascriptcore::*;

struct Foo {
    x: i32,
}

js_vtable! {
    impl VTableHandler for Foo {
        fn get_property(_class: &Class<Foo>, context: &Context, _instance: &Foo, _name: &str) -> Option<Value> {
            Some(Value::new_null(context))
        }

        fn set_property(_class: &Class<Foo>, _context: &Context, _instance: &Foo, _name: &str, _value: &Value) -> bool {
            true
        }

        fn has_property(_class: &Class<Foo>, _context: &Context, _instance: &Foo, _name: &str) -> bool {
            true
        }

        fn delete_property(_class: &Class<Foo>, _context: &Context, _instance: &Foo, _name: &str) -> bool {
            true
        }

        fn enumerate_properties(_class: &Class<Foo>, _context: &Context, _instance: &Foo) -> Option<Vec<String>> {
            //Vec::new()
            None
        }
    }
}

#[test]
fn foobar() {
    match gtk::init() {
        Ok(_v) => {},
        Err(e) => panic!(e),
    }
    let ctx = Context::new();
    let class = ctx.register_class::<Foo, ()>("Foo", None);
    let cons = class.add_constructor(Some("Foo"), |_values| {Foo{x: 0}}, glib::types::Type::Pointer);
    class.add_method("bar", |this, _value| {
        this.x = this.x + 1;
        Value::new_number(&Context::get_current().unwrap(), this.x.into())
    });
    class.add_property("baz",
        Some(Box::new(|this: &mut Foo| Value::new_number(&Context::get_current().unwrap(), this.x.into()))),
        Some(Box::new(|this, value| {
            println!("In setter");
            this.x = value.to_int32().into();
        })));
    ctx.set_value("Foo", &cons);
    ctx.evaluate("var f = new Foo()");
    if ctx.evaluate("f.bar()").unwrap().to_int32() != 1 {
        panic!("Wrong number");
    }
    let r = ctx.evaluate("f.bar()").unwrap().to_int32();
    if r != 2 {
        panic!("Wrong number {}", r);
    }
    ctx.evaluate("f.baz = 0");
    if ctx.evaluate("f.baz").unwrap().to_int32() != 0 { panic!("Wrong number!"); }

    let (_res, obj) = ctx.evaluate_in_object::<&str>("var x = 42;", None, None, "", 1);
    assert_eq!(obj.object_get_property("x").unwrap().to_int32(), 42);

    let s = String::from("blah");
    let m: std::collections::HashMap<&str, &AsJsValue> = [("foo", &42 as _), ("bar", &s as _)].iter().cloned().collect();
    let v = Value::new_array(&ctx, &vec![&String::from("foobar"), &42.0, &m]);
    ctx.set_value("myArray", &v);

    assert_eq!(ValueExt::to_string(&ctx.evaluate("myArray.join(' : ')").unwrap()).as_str(), "foobar : 42 : [object Object]");
}
