use retro3d_visualizer::app::{AppMode, LaunchConfig};

/// Demo independiente del osciloscopio armónico Lissajous 3D.
/// Ejecutar con: `cargo run --release --example lissajous_3d`
fn main() {
    retro3d_visualizer::run_visualizer(LaunchConfig {
        title: "Retro3D — 3D Lissajous Oscilloscope".to_string(),
        mode: AppMode::Lissajous,
        preset_path: None,
        show_hud_initially: true,
    });
}
