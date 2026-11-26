use std::collections::VecDeque;

use glam::Vec3;
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::{integrate, Integrator};

/// Parámetros configurables del atractor caótico de Lorenz.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LorenzParams {
    pub sigma: f32,
    pub rho: f32,
    pub beta: f32,
    pub dt: f32,
    pub particle_count: usize,
    pub trail_length: usize,
    pub integrator: Integrator,
}

impl Default for LorenzParams {
    fn default() -> Self {
        Self {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
            dt: 0.008,
            particle_count: 48,
            trail_length: 160,
            integrator: Integrator::Rk4,
        }
    }
}

/// Partícula individual con historial de trayectoria para renderizado vectorial.
#[derive(Debug, Clone)]
pub struct LorenzParticle {
    pub position: Vec3,
    pub trail: VecDeque<Vec3>,
}

impl LorenzParticle {
    fn new(initial: Vec3, trail_length: usize) -> Self {
        let mut trail = VecDeque::with_capacity(trail_length + 1);
        trail.push_back(initial);
        Self {
            position: initial,
            trail,
        }
    }
}

/// Ecuaciones diferenciales del sistema de Lorenz:
/// dx/dt = sigma * (y - x)
/// dy/dt = x * (rho - z) - y
/// dz/dt = x * y - beta * z
#[inline]
pub fn lorenz_derivative(p: Vec3, sigma: f32, rho: f32, beta: f32) -> Vec3 {
    Vec3::new(
        sigma * (p.y - p.x),
        p.x * (rho - p.z) - p.y,
        p.x * p.y - beta * p.z,
    )
}

/// Simulador paralelo del atractor de Lorenz con múltiples partículas.
#[derive(Debug, Clone)]
pub struct LorenzSystem {
    pub params: LorenzParams,
    pub particles: Vec<LorenzParticle>,
    pub elapsed_steps: u64,
}

impl LorenzSystem {
    pub fn new(params: LorenzParams) -> Self {
        let mut system = Self {
            params,
            particles: Vec::new(),
            elapsed_steps: 0,
        };
        system.reset();
        system
    }

    /// Reinicia todas las partículas con pequeñas perturbaciones aleatorias
    /// para evidenciar la sensibilidad a condiciones iniciales.
    pub fn reset(&mut self) {
        let mut rng = rand::thread_rng();
        let count = self.params.particle_count.clamp(1, 512);
        let trail_len = self.params.trail_length.clamp(8, 512);
        self.particles.clear();
        self.particles.reserve(count);

        for i in 0..count {
            let spread = 0.35;
            let base = if i % 2 == 0 {
                Vec3::new(1.0, 1.0, 1.0)
            } else {
                Vec3::new(-1.0, -1.0, 1.0)
            };
            let jitter = Vec3::new(
                rng.gen_range(-spread..spread),
                rng.gen_range(-spread..spread),
                rng.gen_range(-spread..spread),
            );
            self.particles
                .push(LorenzParticle::new(base + jitter, trail_len));
        }

        self.elapsed_steps = 0;
        // Pre-calentamiento breve para que el atractor muestre estructura desde el primer cuadro
        self.step(24);
    }

    /// Ajusta dinámicamente la cantidad de partículas sin perder las trayectorias existentes.
    pub fn sync_particle_count(&mut self) {
        let target = self.params.particle_count.clamp(1, 512);
        let trail_len = self.params.trail_length.clamp(8, 512);

        if self.particles.len() > target {
            self.particles.truncate(target);
        } else if self.particles.len() < target {
            let mut rng = rand::thread_rng();
            while self.particles.len() < target {
                let initial = Vec3::new(
                    rng.gen_range(-2.0..2.0),
                    rng.gen_range(-2.0..2.0),
                    rng.gen_range(12.0..24.0),
                );
                self.particles
                    .push(LorenzParticle::new(initial, trail_len));
            }
        }

        for p in &mut self.particles {
            while p.trail.len() > trail_len {
                p.trail.pop_front();
            }
        }
    }

    /// Avanza todas las partículas en paralelo usando Rayon (en nativo).
    pub fn step(&mut self, substeps: usize) {
        self.sync_particle_count();
        let sigma = self.params.sigma;
        let rho = self.params.rho;
        let beta = self.params.beta;
        let dt = self.params.dt.clamp(0.0005, 0.03);
        let method = self.params.integrator;
        let trail_len = self.params.trail_length.clamp(8, 512);

        let update_particle = |particle: &mut LorenzParticle| {
            for _ in 0..substeps {
                let next = integrate(method, particle.position, dt, |p| {
                    lorenz_derivative(p, sigma, rho, beta)
                });
                if next.is_finite() && next.length_squared() < 1.0e6 {
                    particle.position = next;
                } else {
                    particle.position = Vec3::new(0.5, 0.5, 1.0);
                    particle.trail.clear();
                }
                particle.trail.push_back(particle.position);
                while particle.trail.len() > trail_len {
                    particle.trail.pop_front();
                }
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        self.particles.par_iter_mut().for_each(update_particle);

        #[cfg(target_arch = "wasm32")]
        self.particles.iter_mut().for_each(update_particle);

        self.elapsed_steps += substeps as u64;
    }

    /// Convierte las trayectorias en segmentos 3D centrados y escalados para visualización,
    /// acompañados de un factor de intensidad [0.15, 1.0] (más brillante en la cabeza).
    pub fn line_segments(&self) -> Vec<(Vec3, Vec3, f32)> {
        let total_segments: usize = self
            .particles
            .iter()
            .map(|p| p.trail.len().saturating_sub(1))
            .sum();
        let mut segments = Vec::with_capacity(total_segments);

        // Centrar el atractor alrededor del origen (el centro clásico está en z ≈ 25)
        // y remapear a coordenadas Y-up para el visor 3D.
        let scale = 0.32;
        let transform = |v: Vec3| Vec3::new(v.x * scale, (v.z - 25.0) * scale, v.y * scale);

        for particle in &self.particles {
            let len = particle.trail.len();
            if len < 2 {
                continue;
            }
            let denom = (len - 1) as f32;
            for i in 0..(len - 1) {
                let a = transform(particle.trail[i]);
                let b = transform(particle.trail[i + 1]);
                let age = (i + 1) as f32 / denom;
                let intensity = 0.15 + 0.85 * age * age;
                segments.push((a, b, intensity));
            }
        }

        segments
    }
}
