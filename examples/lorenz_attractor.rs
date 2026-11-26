use retro3d_visualizer::app::{AppMode, LaunchConfig};

/// Demo independiente del atractor caótico de Lorenz.
/// Ejecutar con: `cargo run --release --example lorenz_attractor`
fn main() {
    retro3d_visualizer::run_visualizer(LaunchConfig {
        title: "Retro3D — Lorenz Attractor Demo".to_string(),
        mode: AppMode::Lorenz,
        preset_path: None,
        show_hud_initially: true,
    });
}
