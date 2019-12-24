use libloading::Library;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr::NonNull;

pub trait Plugin: 'static {
    fn name(&self) -> &str;
    fn operator(&self) -> &str;
    fn calc(&self, lhs: u32, rhs: u32) -> u32;
}

#[repr(C)]
pub struct StrSlice {
    ptr: *const c_void,
    len: usize,
}

impl StrSlice {
    #[inline]
    pub unsafe fn from_str(s: &str) -> Self {
        Self {
            ptr: s.as_ptr() as *const c_void,
            len: s.len(),
        }
    }

    #[inline]
    pub unsafe fn into_str<'s>(self) -> &'s str {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.ptr as *mut u8, self.len))
    }
}

#[repr(C)]
pub struct PluginVTable {
    pub name: unsafe extern "C" fn(*const c_void) -> StrSlice,
    pub operator: unsafe extern "C" fn(*const c_void) -> StrSlice,
    pub calc: unsafe extern "C" fn(*const c_void, u32, u32) -> u32,
    pub drop: unsafe extern "C" fn(*mut c_void),
}

#[repr(C)]
pub struct PluginData {
    pub ctx: Option<NonNull<c_void>>,
    pub vtable: &'static PluginVTable,
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
            let load_plugin =
                lib.get::<unsafe extern "C" fn() -> PluginData>("load_plugin".as_ref())?;
            load_plugin()
        };

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
