use retro3d_visualizer::app::{AppMode, LaunchConfig};

/// Demo independiente de superficies topológicas (Botella de Klein, Möbius, Toroide).
/// Ejecutar con: `cargo run --release --example klein_mobius`
fn main() {
    retro3d_visualizer::run_visualizer(LaunchConfig {
        title: "Retro3D — Klein Bottle & Mobius Topology".to_string(),
        mode: AppMode::Topology,
        preset_path: None,
        show_hud_initially: true,
    });
}
