use retro3d_visualizer::app::{AppMode, LaunchConfig};

/// Demo independiente de la galería de atractores caóticos (Aizawa, Rössler, Thomas).
/// Ejecutar con: `cargo run --release --example attractors_gallery`
fn main() {
    retro3d_visualizer::run_visualizer(LaunchConfig {
        title: "Retro3D — Chaotic Attractors Gallery".to_string(),
        mode: AppMode::Attractors,
        preset_path: None,
        show_hud_initially: true,
    });
}
