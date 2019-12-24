use calc::Loader;
use std::{env, path::Path};

fn main() -> anyhow::Result<()> {
    init_ld_library_path()?;

    let mut loader = Loader::new();
    loader.load("libcalc_plugin_add.so")?;
    loader.load("libcalc_plugin_mul.so")?;

    for plugin in loader.plugins() {
        println!("Plugin: {}", plugin.name());
        println!("Calc: 1 {} 2 = {}", plugin.operator(), plugin.calc(1, 2));
    }

    Ok(())
}

fn init_ld_library_path() -> anyhow::Result<()> {
    let plugins_path = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(path) => Path::new(&path).join("target/debug"),
        None => return Ok(()),
    };

    if let Some(path) = env::var_os("LD_LIBRARY_PATH") {
        let mut paths: Vec<_> = env::split_paths(&path).collect();
        paths.push(plugins_path);
        let new_path = env::join_paths(paths)?;
        env::set_var("LD_LIBRARY_PATH", &new_path);
    }

    Ok(())
}
