use retro3d_visualizer::app::{AppMode, LaunchConfig};

/// Demo independiente de vuelo sobre ciudad wireframe y sol retro en el horizonte.
/// Ejecutar con: `cargo run --release --example cyber_city`
fn main() {
    retro3d_visualizer::run_visualizer(LaunchConfig {
        title: "Retro3D — Cyber City Wireframe Flythrough".to_string(),
        mode: AppMode::City,
        preset_path: None,
        show_hud_initially: true,
    });
}
