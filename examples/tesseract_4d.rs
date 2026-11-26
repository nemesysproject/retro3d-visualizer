use retro3d_visualizer::app::{AppMode, LaunchConfig};

/// Demo independiente de proyección 4D → 3D (Hipercubo / Teseracto y politopos).
/// Ejecutar con: `cargo run --release --example tesseract_4d`
fn main() {
    retro3d_visualizer::run_visualizer(LaunchConfig {
        title: "Retro3D — 4D Tesseract & Polytopes".to_string(),
        mode: AppMode::Tesseract,
        preset_path: None,
        show_hud_initially: true,
    });
}
