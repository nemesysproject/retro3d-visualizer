use retro3d_visualizer::app::{AppMode, LaunchConfig};

/// Demo independiente del recorrido de terreno vectorial estilo años 80.
/// Ejecutar con: `cargo run --release --example wireframe_trench`
fn main() {
    retro3d_visualizer::run_visualizer(LaunchConfig {
        title: "Retro3D — 80s Wireframe Trench Flythrough".to_string(),
        mode: AppMode::Terrain,
        preset_path: None,
        show_hud_initially: true,
    });
}
