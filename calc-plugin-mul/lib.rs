use calc::{Plugin, PluginData, PluginVTable, StrSlice};
use std::os::raw::c_void;
use std::ptr::NonNull;

static VTABLE: &PluginVTable = &PluginVTable {
    name: name,
    operator: operator,
    calc: calc,
    drop: drop_plugin,
};

#[no_mangle]
pub unsafe extern "C" fn load_plugin() -> PluginData {
    PluginData {
        ctx: NonNull::new(Box::into_raw(Box::new(PluginMul)) as *mut c_void),
        vtable: VTABLE,
    }
}

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
unsafe extern "C" fn drop_plugin(ptr: *mut c_void) {
    drop(Box::from_raw(ptr as *mut PluginMul))
}

#[no_mangle]
unsafe extern "C" fn name(ptr: *const c_void) -> StrSlice {
    let ctx = &*(ptr as *const PluginMul);
    StrSlice::from_str(ctx.name())
}

#[no_mangle]
unsafe extern "C" fn operator<'a>(ptr: *const c_void) -> StrSlice {
    let ctx = &*(ptr as *const PluginMul);
    StrSlice::from_str(ctx.operator())
}

#[no_mangle]
unsafe extern "C" fn calc(ptr: *const c_void, lhs: u32, rhs: u32) -> u32 {
    let ctx = &*(ptr as *const PluginMul);
    ctx.calc(lhs, rhs)
}
