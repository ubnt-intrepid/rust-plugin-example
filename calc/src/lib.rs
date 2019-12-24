use libloading::Library;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr::NonNull;

pub trait Plugin: 'static {
    fn name(&self) -> &str;
    fn operator(&self) -> &str;
    fn calc(&self, lhs: u32, rhs: u32) -> u32;
}

#[derive(Debug)]
pub struct Loader {
    plugins: Vec<PluginProxy>,
}

impl Loader {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    pub fn load(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let lib = Library::new(path.as_ref())?;

        let ctx = unsafe {
            let load_plugin =
                lib.get::<unsafe extern "C" fn() -> *mut c_void>("load_plugin".as_ref())?;
            let ctx = load_plugin();
            NonNull::new(ctx).ok_or_else(|| anyhow::anyhow!("failed to load the plugin context"))?
        };

        self.plugins.push(PluginProxy { lib, ctx });
        Ok(())
    }

    pub fn plugins(&self) -> impl Iterator<Item = &'_ dyn Plugin> + '_ {
        self.plugins.iter().map(|plugin| plugin as &dyn Plugin)
    }
}

#[derive(Debug)]
struct PluginProxy {
    lib: Library,
    ctx: NonNull<c_void>,
}

impl Drop for PluginProxy {
    fn drop(&mut self) {
        unsafe {
            if let Ok(release_plugin) = self
                .lib
                .get::<unsafe extern "C" fn(*mut c_void)>("release_plugin".as_ref())
            {
                release_plugin(self.ctx.as_ptr());
            }
        }
    }
}

impl Plugin for PluginProxy {
    fn name<'a>(&'a self) -> &'a str {
        unsafe {
            let name = self
                .lib
                .get::<unsafe extern "C" fn(*mut c_void) -> &'a str>("name".as_ref())
                .expect("missing symbol");
            name(self.ctx.as_ptr())
        }
    }

    fn operator<'a>(&'a self) -> &'a str {
        unsafe {
            let operator = self
                .lib
                .get::<unsafe extern "C" fn(*mut c_void) -> &'a str>("operator".as_ref())
                .expect("missing symbol");
            operator(self.ctx.as_ptr())
        }
    }

    fn calc(&self, lhs: u32, rhs: u32) -> u32 {
        unsafe {
            let calc = self
                .lib
                .get::<unsafe extern "C" fn(*mut c_void, u32, u32) -> u32>("calc".as_ref())
                .expect("missing symbol");
            calc(self.ctx.as_ptr(), lhs, rhs)
        }
    }
}
