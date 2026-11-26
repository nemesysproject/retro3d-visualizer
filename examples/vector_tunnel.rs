use retro3d_visualizer::app::{AppMode, LaunchConfig};

/// Demo independiente del túnel hiperespacial estilo arcade años 80 (Tempest).
/// Ejecutar con: `cargo run --release --example vector_tunnel`
fn main() {
    retro3d_visualizer::run_visualizer(LaunchConfig {
        title: "Retro3D — Hyperspace Vector Tunnel".to_string(),
        mode: AppMode::Tunnel,
        preset_path: None,
        show_hud_initially: true,
    });
}
