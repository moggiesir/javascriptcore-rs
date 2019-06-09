use glib::translate::*;
use javascriptcore_sys;

glib_wrapper! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct GlobalContextRef(Shared<javascriptcore_sys::_JSGlobalContextRef>);

    match fn {
        ref => |ptr| javascriptcore_sys::JSGlobalContextRetain(ptr),
        unref => |ptr| javascriptcore_sys::JSGlobalContextRelease(ptr),
    }
}
