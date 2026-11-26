use retro3d_visualizer::app::{AppMode, LaunchConfig};

/// Demo independiente del campo magnético dipolar 3D con pulsos de flujo.
/// Ejecutar con: `cargo run --release --example dipole_field`
fn main() {
    retro3d_visualizer::run_visualizer(LaunchConfig {
        title: "Retro3D — 3D Magnetic Dipole Field".to_string(),
        mode: AppMode::Dipole,
        preset_path: None,
        show_hud_initially: true,
    });
}
