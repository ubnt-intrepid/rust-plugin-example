use crate::inner::{LoadPluginResult, PluginVTable};
use libloading::{Library, Symbol};
use std::{os::raw::c_void, path::Path, ptr::NonNull};

#[doc(hidden)] // private API for export_plugin.
pub mod _export {
    pub use crate::inner::{LoadPluginResult, PluginVTable};
    pub use lazy_static::lazy_static;
    pub use std::{os::raw::c_void, ptr::NonNull};
}

#[macro_export]
macro_rules! export_plugin {
    ($Plugin:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn load_plugin() -> $crate::_export::LoadPluginResult {
            $crate::_export::lazy_static! {
                static ref VTABLE: $crate::_export::PluginVTable = $crate::_export::PluginVTable::new::<$Plugin>();
            }
            $crate::_export::LoadPluginResult {
                ctx: $crate::_export::NonNull::new(
                    Box::into_raw(Box::new(<$Plugin>::default())) as *mut $crate::_export::c_void
                ),
                vtable: &*VTABLE,
            }
        }
    };
}

pub trait Plugin: 'static {
    fn name(&self) -> &str;
    fn operator(&self) -> &str;
    fn calc(&self, lhs: u32, rhs: u32) -> u32;
}

struct PluginProxy {
    #[allow(dead_code)]
    lib: Library,
    ctx: NonNull<c_void>,
    vtable: &'static PluginVTable,
}

impl PluginProxy {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let lib = Library::new(path.as_ref())?;

        let ret = unsafe {
            let load_plugin: Symbol<unsafe extern "C" fn() -> LoadPluginResult> =
                lib.get("load_plugin".as_ref())?;
            load_plugin()
        };

        anyhow::ensure!(
            ret.vtable.version == crate::inner::VERSION_STR,
            "plugin version mismatched"
        );

        let ctx = ret
            .ctx
            .ok_or_else(|| anyhow::anyhow!("failed to load the plugin"))?;

        Ok(Self {
            lib,
            ctx,
            vtable: ret.vtable,
        })
    }
}

impl Drop for PluginProxy {
    fn drop(&mut self) {
        unsafe {
            (self.vtable.drop)(self.ctx.as_ptr());
        }
    }
}

impl Plugin for PluginProxy {
    fn name(&self) -> &str {
        unsafe { (self.vtable.name)(self.ctx.as_ref()).into_str() }
    }

    fn operator(&self) -> &str {
        unsafe { (self.vtable.operator)(self.ctx.as_ref()).into_str() }
    }

    fn calc(&self, lhs: u32, rhs: u32) -> u32 {
        unsafe { (self.vtable.calc)(self.ctx.as_ref(), lhs, rhs) }
    }
}

pub struct Loader {
    plugins: Vec<Box<dyn Plugin>>,
}

impl Loader {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    pub fn load(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.plugins.push(Box::new(PluginProxy::load(path)?));
        Ok(())
    }

    pub fn plugins(&self) -> impl Iterator<Item = &'_ dyn Plugin> + '_ {
        self.plugins.iter().map(|plugin| &**plugin)
    }
}

mod inner {
    use super::*;

    pub const VERSION_STR: &str = env!("CARGO_PKG_VERSION");

    #[repr(C)]
    pub struct LoadPluginResult {
        pub ctx: Option<NonNull<c_void>>,
        pub vtable: &'static PluginVTable,
    }

    #[repr(C)]
    pub struct PluginVTable {
        pub(crate) version: &'static str,
        pub(crate) name: unsafe extern "C" fn(*const c_void) -> StrSlice,
        pub(crate) operator: unsafe extern "C" fn(*const c_void) -> StrSlice,
        pub(crate) calc: unsafe extern "C" fn(*const c_void, u32, u32) -> u32,
        pub(crate) drop: unsafe extern "C" fn(*mut c_void),
    }

    impl PluginVTable {
        pub fn new<P: Plugin>() -> Self {
            Self {
                version: VERSION_STR,
                name: name::<P>,
                operator: operator::<P>,
                calc: calc::<P>,
                drop: drop_ctx::<P>,
            }
        }
    }

    #[repr(C)]
    pub(crate) struct StrSlice {
        ptr: *const c_void,
        len: usize,
    }

    impl StrSlice {
        #[inline]
        pub(crate) unsafe fn from_str(s: &str) -> Self {
            Self {
                ptr: s.as_ptr() as *const c_void,
                len: s.len(),
            }
        }

        #[inline]
        pub(crate) unsafe fn into_str<'s>(self) -> &'s str {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.ptr as *mut u8, self.len))
        }
    }

    unsafe extern "C" fn name<P: Plugin>(ptr: *const c_void) -> StrSlice {
        let ctx = &*(ptr as *const P);
        StrSlice::from_str(ctx.name())
    }

    unsafe extern "C" fn operator<P: Plugin>(ptr: *const c_void) -> StrSlice {
        let ctx = &*(ptr as *const P);
        StrSlice::from_str(ctx.operator())
    }

    unsafe extern "C" fn calc<P: Plugin>(ptr: *const c_void, lhs: u32, rhs: u32) -> u32 {
        let ctx = &*(ptr as *const P);
        ctx.calc(lhs, rhs)
    }

    unsafe extern "C" fn drop_ctx<P: Plugin>(ptr: *mut c_void) {
        drop(Box::from_raw(ptr as *mut P))
    }
}
