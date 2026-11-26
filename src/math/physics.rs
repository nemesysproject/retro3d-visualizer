use std::collections::VecDeque;

use glam::Vec3;
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::{integrate, Integrator};

// ============================================================================
// 4. GALERÍA DE ATRACTORES CAÓTICOS (AIZAWA, RÖSSLER, THOMAS)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChaoticAttractorKind {
    Aizawa,
    Rossler,
    Thomas,
}

impl Default for ChaoticAttractorKind {
    fn default() -> Self {
        Self::Aizawa
    }
}

impl ChaoticAttractorKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Aizawa => "Atractor de Aizawa (Toroide caótico)",
            Self::Rossler => "Atractor de Rössler (Espiral plegada)",
            Self::Thomas => "Atractor de Thomas (Simetría cíclica)",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GalleryParams {
    pub kind: ChaoticAttractorKind,
    pub dt: f32,
    pub particle_count: usize,
    pub trail_length: usize,
    pub integrator: Integrator,
    pub param_a: f32,
    pub param_b: f32,
}

impl Default for GalleryParams {
    fn default() -> Self {
        Self {
            kind: ChaoticAttractorKind::Aizawa,
            dt: 0.012,
            particle_count: 52,
            trail_length: 170,
            integrator: Integrator::Rk4,
            param_a: 0.95,
            param_b: 0.20,
        }
    }
}

#[derive(Debug, Clone)]
struct TrailParticle {
    pos: Vec3,
    trail: VecDeque<Vec3>,
}

#[derive(Debug, Clone)]
pub struct AttractorGallerySystem {
    pub params: GalleryParams,
    particles: Vec<TrailParticle>,
    last_kind: ChaoticAttractorKind,
}

impl AttractorGallerySystem {
    pub fn new(params: GalleryParams) -> Self {
        let mut sys = Self {
            last_kind: params.kind,
            params,
            particles: Vec::new(),
        };
        sys.reset();
        sys
    }

    pub fn reset(&mut self) {
        self.last_kind = self.params.kind;
        let mut rng = rand::thread_rng();
        let count = self.params.particle_count.clamp(1, 256);
        let trail_len = self.params.trail_length.clamp(12, 360);
        self.particles.clear();

        for _ in 0..count {
            let init = match self.params.kind {
                ChaoticAttractorKind::Aizawa => Vec3::new(
                    0.1 + rng.gen_range(-0.15..0.15),
                    rng.gen_range(-0.15..0.15),
                    rng.gen_range(-0.15..0.15),
                ),
                ChaoticAttractorKind::Rossler => Vec3::new(
                    rng.gen_range(-2.0..2.0),
                    rng.gen_range(-2.0..2.0),
                    rng.gen_range(0.1..1.5),
                ),
                ChaoticAttractorKind::Thomas => Vec3::new(
                    rng.gen_range(-1.8..1.8),
                    rng.gen_range(-1.8..1.8),
                    rng.gen_range(-1.8..1.8),
                ),
            };
            let mut trail = VecDeque::with_capacity(trail_len + 1);
            trail.push_back(init);
            self.particles.push(TrailParticle { pos: init, trail });
        }
        self.step(30);
    }

    fn derivative(kind: ChaoticAttractorKind, p: Vec3, a: f32, b: f32) -> Vec3 {
        match kind {
            ChaoticAttractorKind::Aizawa => {
                // a ≈ 0.95, b ≈ 0.7, c = 0.6, d = 3.5, e = 0.25, f = 0.1
                let c = 0.6;
                let d = 3.5;
                let e = 0.25;
                let f = 0.1;
                let b_eff = 0.5 + b; // 0.7 por defecto cuando b=0.20
                Vec3::new(
                    (p.z - b_eff) * p.x - d * p.y,
                    d * p.x + (p.z - b_eff) * p.y,
                    c + a * p.z
                        - (p.z * p.z * p.z) / 3.0
                        - (p.x * p.x + p.y * p.y) * (1.0 + e * p.z)
                        + f * p.z * p.x * p.x * p.x,
                )
            }
            ChaoticAttractorKind::Rossler => {
                let a_r = b.clamp(0.05, 0.45);
                let b_r = 0.2;
                let c_r = 5.7 * (a / 0.95).clamp(0.5, 1.8);
                Vec3::new(-p.y - p.z, p.x + a_r * p.y, b_r + p.z * (p.x - c_r))
            }
            ChaoticAttractorKind::Thomas => {
                let b_t = b.clamp(0.05, 0.35);
                Vec3::new(
                    p.y.sin() - b_t * p.x,
                    p.z.sin() - b_t * p.y,
                    p.x.sin() - b_t * p.z,
                )
            }
        }
    }

    pub fn step(&mut self, substeps: usize) {
        if self.last_kind != self.params.kind {
            self.reset();
            return;
        }
        let target = self.params.particle_count.clamp(1, 256);
        let trail_len = self.params.trail_length.clamp(12, 360);
        if self.particles.len() != target {
            self.reset();
            return;
        }

        let kind = self.params.kind;
        let method = self.params.integrator;
        let a = self.params.param_a;
        let b = self.params.param_b;
        let dt = match kind {
            ChaoticAttractorKind::Aizawa => self.params.dt.clamp(0.002, 0.02),
            ChaoticAttractorKind::Rossler => (self.params.dt * 2.2).clamp(0.005, 0.04),
            ChaoticAttractorKind::Thomas => (self.params.dt * 5.0).clamp(0.01, 0.12),
        };

        let update_fn = |particle: &mut TrailParticle| {
            for _ in 0..substeps {
                let next = integrate(method, particle.pos, dt, |p| Self::derivative(kind, p, a, b));
                if next.is_finite() && next.length_squared() < 1.0e5 {
                    particle.pos = next;
                } else {
                    particle.pos = Vec3::new(0.1, 0.05, 0.1);
                    particle.trail.clear();
                }
                particle.trail.push_back(particle.pos);
                while particle.trail.len() > trail_len {
                    particle.trail.pop_front();
                }
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        self.particles.par_iter_mut().for_each(update_fn);
        #[cfg(target_arch = "wasm32")]
        self.particles.iter_mut().for_each(update_fn);
    }

    pub fn line_segments(&self) -> Vec<(Vec3, Vec3, f32)> {
        let mut segments = Vec::with_capacity(self.particles.len() * self.params.trail_length);
        let transform = |v: Vec3| match self.params.kind {
            ChaoticAttractorKind::Aizawa => Vec3::new(v.x * 5.2, (v.z - 0.65) * 5.2, v.y * 5.2),
            ChaoticAttractorKind::Rossler => Vec3::new(v.x * 0.55, (v.z - 6.0) * 0.55, v.y * 0.55),
            ChaoticAttractorKind::Thomas => v * 2.4,
        };

        for p in &self.particles {
            let len = p.trail.len();
            if len < 2 {
                continue;
            }
            let denom = (len - 1) as f32;
            for i in 0..(len - 1) {
                let age = (i + 1) as f32 / denom;
                segments.push((
                    transform(p.trail[i]),
                    transform(p.trail[i + 1]),
                    0.15 + 0.85 * age * age,
                ));
            }
        }
        segments
    }
}

// ============================================================================
// 5. PROBLEMA GRAVITACIONAL DE LOS 3 CUERPOS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreeBodyPreset {
    FigureEight,
    LagrangeDance,
    ChaoticSlingshot,
}

impl Default for ThreeBodyPreset {
    fn default() -> Self {
        Self::FigureEight
    }
}

impl ThreeBodyPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::FigureEight => "Coreografía en Ocho (Estable)",
            Self::LagrangeDance => "Triángulo de Lagrange 3D",
            Self::ChaoticSlingshot => "Tirón Gravitacional Caótico",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThreeBodyParams {
    pub preset: ThreeBodyPreset,
    pub gravity: f32,
    pub dt: f32,
    pub trail_length: usize,
    pub softening: f32,
}

impl Default for ThreeBodyParams {
    fn default() -> Self {
        Self {
            preset: ThreeBodyPreset::FigureEight,
            gravity: 1.0,
            dt: 0.006,
            trail_length: 280,
            softening: 0.04,
        }
    }
}

#[derive(Debug, Clone)]
struct Body {
    pos: Vec3,
    vel: Vec3,
    mass: f32,
    trail: VecDeque<Vec3>,
}

#[derive(Debug, Clone)]
pub struct ThreeBodySystem {
    pub params: ThreeBodyParams,
    bodies: Vec<Body>,
    last_preset: ThreeBodyPreset,
}

impl ThreeBodySystem {
    pub fn new(params: ThreeBodyParams) -> Self {
        let mut sys = Self {
            last_preset: params.preset,
            params,
            bodies: Vec::new(),
        };
        sys.reset();
        sys
    }

    pub fn reset(&mut self) {
        self.last_preset = self.params.preset;
        let len = self.params.trail_length.clamp(20, 600);
        let init: Vec<(Vec3, Vec3, f32)> = match self.params.preset {
            ThreeBodyPreset::FigureEight => vec![
                (
                    Vec3::new(0.970_004_36, -0.243_087_53, 0.0),
                    Vec3::new(0.466_203_68, 0.432_365_73, 0.08),
                    1.0,
                ),
                (
                    Vec3::new(-0.970_004_36, 0.243_087_53, 0.0),
                    Vec3::new(0.466_203_68, 0.432_365_73, -0.08),
                    1.0,
                ),
                (
                    Vec3::ZERO,
                    Vec3::new(-0.932_407_37, -0.864_731_46, 0.0),
                    1.0,
                ),
            ],
            ThreeBodyPreset::LagrangeDance => {
                let r = 1.15;
                let v = 0.65;
                (0..3)
                    .map(|k| {
                        let a = k as f32 * std::f32::consts::TAU / 3.0;
                        let pos = Vec3::new(r * a.cos(), 0.25 * (k as f32 - 1.0), r * a.sin());
                        let vel = Vec3::new(-v * a.sin(), 0.15 * a.cos(), v * a.cos());
                        (pos, vel, 1.0)
                    })
                    .collect()
            }
            ThreeBodyPreset::ChaoticSlingshot => vec![
                (Vec3::new(-1.3, 0.2, 0.4), Vec3::new(0.25, 0.42, -0.18), 1.4),
                (Vec3::new(1.1, -0.4, -0.3), Vec3::new(-0.35, 0.15, 0.48), 1.1),
                (Vec3::new(0.2, 0.9, -0.5), Vec3::new(0.10, -0.55, -0.25), 0.9),
            ],
        };

        self.bodies = init
            .into_iter()
            .map(|(pos, vel, mass)| {
                let mut trail = VecDeque::with_capacity(len + 1);
                trail.push_back(pos);
                Body {
                    pos,
                    vel,
                    mass,
                    trail,
                }
            })
            .collect();
    }

    fn accelerations(positions: &[Vec3], masses: &[f32], g: f32, eps2: f32) -> Vec<Vec3> {
        let n = positions.len();
        let mut acc = vec![Vec3::ZERO; n];
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let diff = positions[j] - positions[i];
                let dist2 = diff.length_squared() + eps2;
                let inv_dist3 = 1.0 / (dist2 * dist2.sqrt());
                acc[i] += diff * (g * masses[j] * inv_dist3);
            }
        }
        acc
    }

    pub fn step(&mut self, substeps: usize) {
        if self.last_preset != self.params.preset {
            self.reset();
            return;
        }
        let n = self.bodies.len();
        let g = self.params.gravity;
        let dt = self.params.dt.clamp(0.001, 0.02);
        let eps2 = self.params.softening.max(0.01).powi(2);
        let trail_len = self.params.trail_length.clamp(20, 600);
        let masses: Vec<f32> = self.bodies.iter().map(|b| b.mass).collect();

        for _ in 0..substeps {
            let p0: Vec<Vec3> = self.bodies.iter().map(|b| b.pos).collect();
            let v0: Vec<Vec3> = self.bodies.iter().map(|b| b.vel).collect();

            let a1 = Self::accelerations(&p0, &masses, g, eps2);
            let p1: Vec<Vec3> = (0..n).map(|i| p0[i] + v0[i] * (0.5 * dt)).collect();
            let v1: Vec<Vec3> = (0..n).map(|i| v0[i] + a1[i] * (0.5 * dt)).collect();

            let a2 = Self::accelerations(&p1, &masses, g, eps2);
            let p2: Vec<Vec3> = (0..n).map(|i| p0[i] + v1[i] * (0.5 * dt)).collect();
            let v2: Vec<Vec3> = (0..n).map(|i| v0[i] + a2[i] * (0.5 * dt)).collect();

            let a3 = Self::accelerations(&p2, &masses, g, eps2);
            let p3: Vec<Vec3> = (0..n).map(|i| p0[i] + v2[i] * dt).collect();
            let v3: Vec<Vec3> = (0..n).map(|i| v0[i] + a3[i] * dt).collect();

            let a4 = Self::accelerations(&p3, &masses, g, eps2);

            for i in 0..n {
                self.bodies[i].pos =
                    p0[i] + (v0[i] + v1[i] * 2.0 + v2[i] * 2.0 + v3[i]) * (dt / 6.0);
                self.bodies[i].vel =
                    v0[i] + (a1[i] + a2[i] * 2.0 + a3[i] * 2.0 + a4[i]) * (dt / 6.0);

                if !self.bodies[i].pos.is_finite() || self.bodies[i].pos.length() > 30.0 {
                    self.reset();
                    return;
                }
                let p = self.bodies[i].pos;
                self.bodies[i].trail.push_back(p);
                while self.bodies[i].trail.len() > trail_len {
                    self.bodies[i].trail.pop_front();
                }
            }
        }
    }

    pub fn line_segments(&self) -> Vec<(Vec3, Vec3, f32)> {
        let scale = 5.2;
        let mut segments = Vec::with_capacity(self.bodies.len() * (self.params.trail_length + 12));

        for body in &self.bodies {
            let len = body.trail.len();
            if len >= 2 {
                let denom = (len - 1) as f32;
                for i in 0..(len - 1) {
                    let age = (i + 1) as f32 / denom;
                    segments.push((
                        body.trail[i] * scale,
                        body.trail[i + 1] * scale,
                        0.15 + 0.85 * age * age,
                    ));
                }
            }
            // Marcador octaédrico brillante para cada cuerpo
            let c = body.pos * scale;
            let r = 0.35 * body.mass.cbrt();
            let pts = [
                c + Vec3::X * r,
                c - Vec3::X * r,
                c + Vec3::Y * r,
                c - Vec3::Y * r,
                c + Vec3::Z * r,
                c - Vec3::Z * r,
            ];
            for &(a, b) in &[
                (0, 2), (0, 3), (0, 4), (0, 5),
                (1, 2), (1, 3), (1, 4), (1, 5),
                (2, 4), (4, 3), (3, 5), (5, 2),
            ] {
                segments.push((pts[a], pts[b], 1.0));
            }
        }
        segments
    }
}

// ============================================================================
// 6. OSCILOSCOPIO LISSAJOUS 3D
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LissajousParams {
    pub freq_x: f32,
    pub freq_y: f32,
    pub freq_z: f32,
    pub phase_x: f32,
    pub phase_y: f32,
    pub samples: usize,
    pub animate_phase: bool,
}

impl Default for LissajousParams {
    fn default() -> Self {
        Self {
            freq_x: 3.0,
            freq_y: 4.0,
            freq_z: 5.0,
            phase_x: 1.57,
            phase_y: 0.78,
            samples: 720,
            animate_phase: true,
        }
    }
}

/// Genera una curva armónica de Lissajous 3D con haz luminoso móvil estilo osciloscopio.
pub fn generate_lissajous_wireframe(params: &LissajousParams, time: f32) -> Vec<(Vec3, Vec3, f32)> {
    let n = params.samples.clamp(120, 2000);
    let amp = 6.8;
    let drift = if params.animate_phase { time * 0.55 } else { 0.0 };
    let beam_head = (time * 0.35).fract();

    let mut pts = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = (i as f32 / n as f32) * std::f32::consts::TAU;
        let x = amp * (params.freq_x * t + params.phase_x + drift).sin();
        let y = amp * (params.freq_y * t + params.phase_y - drift * 0.6).sin();
        let z = amp * (params.freq_z * t).sin();
        pts.push(Vec3::new(x, y, z));
    }

    let mut segments = Vec::with_capacity(n);
    for i in 0..n {
        let u = i as f32 / n as f32;
        let dist_behind = (beam_head - u).rem_euclid(1.0);
        let phosphor = (1.0 - dist_behind).powf(2.0);
        let intensity = 0.25 + 0.75 * phosphor;
        segments.push((pts[i], pts[i + 1], intensity));
    }
    segments
}

// ============================================================================
// 7. SUPERFICIES TOPOLÓGICAS (BOTELLA DE KLEIN, CINTA DE MÖBIUS, TOROIDE)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopologyKind {
    KleinBottle,
    MobiusStrip,
    Torus,
}

impl Default for TopologyKind {
    fn default() -> Self {
        Self::KleinBottle
    }
}

impl TopologyKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::KleinBottle => "Botella de Klein (Figura-8)",
            Self::MobiusStrip => "Cinta de Möbius",
            Self::Torus => "Toroide Paramétrico",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TopologyParams {
    pub kind: TopologyKind,
    pub u_steps: usize,
    pub v_steps: usize,
    pub twist_speed: f32,
}

impl Default for TopologyParams {
    fn default() -> Self {
        Self {
            kind: TopologyKind::KleinBottle,
            u_steps: 44,
            v_steps: 22,
            twist_speed: 0.45,
        }
    }
}

/// Genera mallas wireframe paramétricas $(u, v) \mapsto (x, y, z)$ de superficies topológicas.
pub fn generate_topology_wireframe(params: &TopologyParams, time: f32) -> Vec<(Vec3, Vec3, f32)> {
    let nu = params.u_steps.clamp(16, 80);
    let nv = params.v_steps.clamp(8, 48);
    let rot = time * params.twist_speed;
    let (sr, cr) = rot.sin_cos();

    let mut grid = vec![vec![Vec3::ZERO; nv + 1]; nu + 1];

    for i in 0..=nu {
        let u_frac = i as f32 / nu as f32;
        let u = u_frac * std::f32::consts::TAU;
        for j in 0..=nv {
            let v_frac = j as f32 / nv as f32;
            let p = match params.kind {
                TopologyKind::KleinBottle => {
                    // Inmersión figura-8 de la botella de Klein
                    let v = v_frac * std::f32::consts::TAU;
                    let r = 2.6;
                    let half_u = u * 0.5;
                    let term = r + half_u.cos() * v.sin() - half_u.sin() * (2.0 * v).sin();
                    let x = term * u.cos();
                    let z = term * u.sin();
                    let y = half_u.sin() * v.sin() + half_u.cos() * (2.0 * v).sin();
                    Vec3::new(x, y, z) * 2.1
                }
                TopologyKind::MobiusStrip => {
                    let s = (v_frac - 0.5) * 3.4;
                    let r = 5.4;
                    let half_u = u * 0.5;
                    let x = (r + s * half_u.cos()) * u.cos();
                    let z = (r + s * half_u.cos()) * u.sin();
                    let y = s * half_u.sin();
                    Vec3::new(x, y, z)
                }
                TopologyKind::Torus => {
                    let v = v_frac * std::f32::consts::TAU;
                    let big_r = 5.2;
                    let small_r = 2.1 + 0.35 * (3.0 * u + time * 1.5).sin();
                    let x = (big_r + small_r * v.cos()) * u.cos();
                    let z = (big_r + small_r * v.cos()) * u.sin();
                    let y = small_r * v.sin();
                    Vec3::new(x, y, z)
                }
            };
            // Rotación lenta alrededor de Y para apreciar la topología
            grid[i][j] = Vec3::new(p.x * cr - p.z * sr, p.y, p.x * sr + p.z * cr);
        }
    }

    let mut segments = Vec::with_capacity(nu * nv * 2);
    for i in 0..nu {
        for j in 0..nv {
            let p = grid[i][j];
            let inten = ((p.y / 6.0) * 0.4 + 0.65).clamp(0.25, 1.0);
            segments.push((p, grid[i + 1][j], inten));
            segments.push((p, grid[i][j + 1], inten * 0.85));
        }
    }
    segments
}

// ============================================================================
// 8. CAMPO MAGNÉTICO DIPOLAR 3D
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DipoleParams {
    pub radial_planes: usize,
    pub shells: usize,
    pub flow_speed: f32,
    pub field_scale: f32,
}

impl Default for DipoleParams {
    fn default() -> Self {
        Self {
            radial_planes: 12,
            shells: 3,
            flow_speed: 0.45,
            field_scale: 8.5,
        }
    }
}

/// Genera líneas de flujo de un dipolo magnético 3D ($r = R \sin^2\theta$) con pulsos animados.
pub fn generate_dipole_wireframe(params: &DipoleParams, time: f32) -> Vec<(Vec3, Vec3, f32)> {
    let planes = params.radial_planes.clamp(4, 24);
    let shells = params.shells.clamp(1, 5);
    let steps = 44;
    let mut segments = Vec::with_capacity(planes * shells * steps + 80);

    // Bobina / solenoide central del dipolo
    for ring in -2..=2 {
        let y = ring as f32 * 0.45;
        let rad = 0.85;
        for k in 0..16 {
            let a0 = (k as f32 / 16.0) * std::f32::consts::TAU;
            let a1 = ((k + 1) as f32 / 16.0) * std::f32::consts::TAU;
            segments.push((
                Vec3::new(rad * a0.cos(), y, rad * a0.sin()),
                Vec3::new(rad * a1.cos(), y, rad * a1.sin()),
                0.95,
            ));
        }
    }

    // Líneas de campo dipolar r = R_shell * sin^2(theta)
    for p_idx in 0..planes {
        let phi = (p_idx as f32 / planes as f32) * std::f32::consts::TAU + time * 0.12;
        let (sp, cp) = phi.sin_cos();

        for s_idx in 0..shells {
            let r_max = params.field_scale * (0.45 + 0.35 * s_idx as f32);
            let pulse_phase = (time * params.flow_speed + s_idx as f32 * 0.27 + p_idx as f32 * 0.08).fract();

            let mut prev: Option<Vec3> = None;
            for i in 1..steps {
                let frac = i as f32 / steps as f32;
                let theta = frac * std::f32::consts::PI;
                let sin_t = theta.sin();
                let r = r_max * sin_t * sin_t;
                let rho = r * sin_t;
                let pt = Vec3::new(rho * cp, r * theta.cos(), rho * sp);

                if let Some(a) = prev {
                    // Pulso luminoso viajando de polo norte a polo sur
                    let dist = (frac - pulse_phase).rem_euclid(1.0);
                    let pulse = (1.0 - (dist * 4.5).min(1.0)).powi(2);
                    let inten = (0.28 + 0.72 * pulse).clamp(0.28, 1.0);
                    segments.push((a, pt, inten));
                }
                prev = Some(pt);
            }
        }
    }

    segments
}
