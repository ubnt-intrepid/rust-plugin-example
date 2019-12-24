use calc::Plugin;

calc::export_plugin!(PluginMul);

#[derive(Default)]
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
