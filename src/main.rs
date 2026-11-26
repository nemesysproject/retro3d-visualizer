use retro3d_visualizer::app::LaunchConfig;

fn main() {
    let config = LaunchConfig::from_cli_args();
    retro3d_visualizer::run_visualizer(config);
}
