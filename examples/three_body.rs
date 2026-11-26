use retro3d_visualizer::app::{AppMode, LaunchConfig};

/// Demo independiente del problema gravitacional de los 3 cuerpos (integrador RK4).
/// Ejecutar con: `cargo run --release --example three_body`
fn main() {
    retro3d_visualizer::run_visualizer(LaunchConfig {
        title: "Retro3D — Three-Body Gravitational Choreography".to_string(),
        mode: AppMode::ThreeBody,
        preset_path: None,
        show_hud_initially: true,
    });
}
