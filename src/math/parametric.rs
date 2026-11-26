use glam::Vec3;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Familia de superficies matemáticas z = f(x, y) disponibles en el modo ecuación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquationKind {
    Ripple,
    WaveGrid,
    Saddle,
    Interference,
}

impl Default for EquationKind {
    fn default() -> Self {
        Self::Ripple
    }
}

impl EquationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ripple => "sin(r) / r  (Onda radial)",
            Self::WaveGrid => "sin(x) * cos(y)  (Malla armónica)",
            Self::Saddle => "x^2 - y^2  (Paraboloide hiperbólico)",
            Self::Interference => "sin(x+t) + cos(y-t)  (Interferencia)",
        }
    }
}

/// Parámetros del graficador de funciones z = f(x, y).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EquationParams {
    pub kind: EquationKind,
    pub grid_size: usize,
    pub extent: f32,
    pub amplitude: f32,
    pub frequency: f32,
    pub animate_phase: bool,
}

impl Default for EquationParams {
    fn default() -> Self {
        Self {
            kind: EquationKind::Ripple,
            grid_size: 36,
            extent: 12.0,
            amplitude: 2.8,
            frequency: 0.75,
            animate_phase: true,
        }
    }
}

/// Parámetros del terreno vectorial estilo años 80 (trinchera wireframe).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerrainParams {
    pub grid_width: usize,
    pub grid_depth: usize,
    pub cell_size: f32,
    pub speed: f32,
    pub trench_width: f32,
    pub mountain_height: f32,
}

impl Default for TerrainParams {
    fn default() -> Self {
        Self {
            grid_width: 36,
            grid_depth: 52,
            cell_size: 1.3,
            speed: 9.0,
            trench_width: 4.5,
            mountain_height: 7.0,
        }
    }
}

/// Evalúa la altura z = f(x, y) para la superficie seleccionada.
#[inline]
pub fn evaluate_equation(kind: EquationKind, x: f32, y: f32, phase: f32, freq: f32, amp: f32) -> f32 {
    let sx = x * freq;
    let sy = y * freq;
    match kind {
        EquationKind::Ripple => {
            let r = (sx * sx + sy * sy).sqrt();
            if r < 0.001 {
                amp * phase.cos()
            } else {
                amp * (r - phase).sin() / (1.0 + 0.35 * r)
            }
        }
        EquationKind::WaveGrid => amp * (sx + phase * 0.5).sin() * (sy - phase * 0.4).cos(),
        EquationKind::Saddle => {
            let wave = 1.0 + 0.25 * phase.sin();
            amp * 0.08 * wave * (sx * sx - sy * sy)
        }
        EquationKind::Interference => {
            let r1 = ((sx - 2.2).powi(2) + (sy - 1.5).powi(2)).sqrt();
            let r2 = ((sx + 2.2).powi(2) + (sy + 1.5).powi(2)).sqrt();
            amp * 0.5 * ((r1 * 1.4 - phase).sin() + (r2 * 1.4 - phase * 1.1).cos())
        }
    }
}

/// Genera los segmentos wireframe de la superficie matemática z = f(x, y) (mapeada a Y-up).
pub fn generate_equation_wireframe(params: &EquationParams, time: f32) -> Vec<(Vec3, Vec3, f32)> {
    let n = params.grid_size.clamp(8, 80);
    let half = params.extent * 0.5;
    let step = params.extent / (n - 1) as f32;
    let phase = if params.animate_phase { time * 1.8 } else { 0.0 };

    let eval_row = |row: usize| -> Vec<Vec3> {
        let y_math = -half + row as f32 * step;
        (0..n)
            .map(|col| {
                let x_math = -half + col as f32 * step;
                let z_math = evaluate_equation(
                    params.kind,
                    x_math,
                    y_math,
                    phase,
                    params.frequency,
                    params.amplitude,
                );
                // Mapeo a espacio 3D Y-up: (x, z_math, y_math)
                Vec3::new(x_math, z_math, y_math)
            })
            .collect()
    };

    #[cfg(not(target_arch = "wasm32"))]
    let grid: Vec<Vec<Vec3>> = (0..n).into_par_iter().map(eval_row).collect();

    #[cfg(target_arch = "wasm32")]
    let grid: Vec<Vec<Vec3>> = (0..n).map(eval_row).collect();

    let mut segments = Vec::with_capacity(2 * n * (n - 1));
    let amp_norm = params.amplitude.abs().max(0.5);

    for r in 0..n {
        for c in 0..n {
            let p = grid[r][c];
            let h_factor = ((p.y / amp_norm) * 0.5 + 0.5).clamp(0.2, 1.0);
            if c + 1 < n {
                let right = grid[r][c + 1];
                segments.push((p, right, h_factor));
            }
            if r + 1 < n {
                let down = grid[r + 1][c];
                segments.push((p, down, h_factor));
            }
        }
    }

    segments
}

/// Función determinista de relieve para la trinchera retro de los años 80.
#[inline]
fn terrain_height(world_x: f32, world_z: f32, trench_width: f32, mountain_height: f32) -> f32 {
    let abs_x = world_x.abs();
    // Valle/corredor central plano para el vuelo rasante
    let wall_factor = ((abs_x - trench_width) / 7.0).clamp(0.0, 1.0);
    let wall_curve = wall_factor * wall_factor * (3.0 - 2.0 * wall_factor);

    // Crestas geométricas estilo synthwave/vectorial
    let ridge_a = (world_x * 0.37 + world_z * 0.19).sin() * (world_z * 0.27).cos();
    let ridge_b = (world_x * 0.71 - world_z * 0.31).cos().abs();
    let ridge_c = ((world_x * 0.15).sin() * (world_z * 0.11).sin()).abs();

    let raw_peaks = (0.45 * ridge_a.abs() + 0.35 * ridge_b + 0.35 * ridge_c) * mountain_height;
    wall_curve * (raw_peaks + 1.2)
}

/// Genera una grilla wireframe de terreno infinito con desplazamiento en Z.
pub fn generate_terrain_wireframe(params: &TerrainParams, scroll_z: f32) -> Vec<(Vec3, Vec3, f32)> {
    let cols = params.grid_width.clamp(12, 80);
    let rows = params.grid_depth.clamp(16, 100);
    let cell = params.cell_size.max(0.4);

    let half_w = (cols - 1) as f32 * cell * 0.5;
    let snapped_z = (scroll_z / cell).floor() * cell;
    let frac_z = scroll_z - snapped_z;

    let build_row = |r: usize| -> Vec<Vec3> {
        // r = 0 cerca de la cámara, r = rows-1 hacia el horizonte negativo Z
        let local_z = 8.0 - (r as f32 * cell) + frac_z;
        let sample_z = snapped_z + (r as f32 * cell);
        (0..cols)
            .map(|c| {
                let x = -half_w + c as f32 * cell;
                let y = terrain_height(x, sample_z, params.trench_width, params.mountain_height);
                Vec3::new(x, y, local_z)
            })
            .collect()
    };

    #[cfg(not(target_arch = "wasm32"))]
    let grid: Vec<Vec<Vec3>> = (0..rows).into_par_iter().map(build_row).collect();

    #[cfg(target_arch = "wasm32")]
    let grid: Vec<Vec<Vec3>> = (0..rows).map(build_row).collect();

    let mut segments = Vec::with_capacity(2 * rows * cols);
    let max_depth = (rows - 1) as f32;

    for r in 0..rows {
        let depth_fade = 1.0 - (r as f32 / max_depth).powf(1.35);
        let depth_intensity = (0.12 + 0.88 * depth_fade).clamp(0.12, 1.0);
        for c in 0..cols {
            let p = grid[r][c];
            let peak_boost = (p.y / params.mountain_height.max(1.0)).clamp(0.0, 1.0) * 0.25;
            let intensity = (depth_intensity + peak_boost).min(1.0);
            if c + 1 < cols {
                segments.push((p, grid[r][c + 1], intensity));
            }
            if r + 1 < rows {
                segments.push((p, grid[r + 1][c], intensity));
            }
        }
    }

    segments
}
