use calc::Plugin;
use std::os::raw::c_void;

struct PluginMul;

impl Plugin for PluginMul {
    fn name(&self) -> &str {
        "mul"
    }

    fn operator(&self) -> &str {
        "*"
    }

    fn calc(&self, lhs: u32, rhs: u32) -> u32 {
        lhs * rhs
    }
}

#[no_mangle]
pub unsafe extern "C" fn load_plugin() -> *mut c_void {
    Box::into_raw(Box::new(PluginMul)) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn release_plugin(ptr: *mut c_void) {
    drop(Box::from_raw(ptr as *mut PluginMul))
}

#[no_mangle]
pub unsafe extern "C" fn name<'a>(ptr: *mut c_void) -> &'a str {
    let ctx = &*(ptr as *mut PluginMul);
    ctx.name()
}

#[no_mangle]
pub unsafe extern "C" fn operator<'a>(ptr: *mut c_void) -> &'a str {
    let ctx = &*(ptr as *mut PluginMul);
    ctx.operator()
}

#[no_mangle]
pub unsafe extern "C" fn calc(ptr: *mut c_void, lhs: u32, rhs: u32) -> u32 {
    let ctx = &*(ptr as *mut PluginMul);
    ctx.calc(lhs, rhs)
}
