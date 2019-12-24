use calc::Plugin;

calc::export_plugin!(PluginAdd);

#[derive(Default)]
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
