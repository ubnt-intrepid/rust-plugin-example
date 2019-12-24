use calc::{Plugin, PluginData, PluginVTable};
use std::os::raw::c_void;
use std::ptr::NonNull;

calc::lazy_static! {
    static ref VTABLE: PluginVTable = PluginVTable::new::<PluginAdd>();
}

#[no_mangle]
pub unsafe extern "C" fn load_plugin() -> PluginData {
    PluginData {
        ctx: NonNull::new(Box::into_raw(Box::new(PluginAdd)) as *mut c_void),
        vtable: &*VTABLE,
    }
}

struct PluginAdd;

impl Plugin for PluginAdd {
    fn name(&self) -> &str {
        "add"
    }

    fn operator(&self) -> &str {
        "+"
    }

    fn calc(&self, lhs: u32, rhs: u32) -> u32 {
        lhs + rhs
    }
}
