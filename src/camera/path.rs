use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Fotograma clave para rutas de cámara preprogramadas.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CameraKeyframe {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub duration: f32,
}

impl CameraKeyframe {
    pub fn new(position: Vec3, target: Vec3, duration: f32) -> Self {
        Self {
            position: position.to_array(),
            target: target.to_array(),
            duration: duration.max(0.25),
        }
    }
}

/// Ruta de cámara interpolada suavemente para demos automáticos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraPath {
    pub keyframes: Vec<CameraKeyframe>,
    pub looping: bool,
    #[serde(skip)]
    pub elapsed: f32,
}

impl Default for CameraPath {
    fn default() -> Self {
        Self::lorenz_orbit()
    }
}

impl CameraPath {
    pub fn lorenz_orbit() -> Self {
        Self {
            keyframes: vec![
                CameraKeyframe::new(Vec3::new(0.0, 3.0, 22.0), Vec3::ZERO, 5.0),
                CameraKeyframe::new(Vec3::new(20.0, 7.0, 10.0), Vec3::ZERO, 5.0),
                CameraKeyframe::new(Vec3::new(14.0, -4.0, -18.0), Vec3::ZERO, 5.0),
                CameraKeyframe::new(Vec3::new(-18.0, 5.0, -12.0), Vec3::ZERO, 5.0),
                CameraKeyframe::new(Vec3::new(-16.0, 9.0, 14.0), Vec3::ZERO, 5.0),
            ],
            looping: true,
            elapsed: 0.0,
        }
    }

    pub fn terrain_flythrough() -> Self {
        Self {
            keyframes: vec![
                CameraKeyframe::new(Vec3::new(0.0, 3.2, 10.0), Vec3::new(0.0, 1.5, -24.0), 4.0),
                CameraKeyframe::new(Vec3::new(2.2, 2.4, 9.5), Vec3::new(-1.0, 1.8, -26.0), 4.0),
                CameraKeyframe::new(Vec3::new(-2.2, 3.8, 10.5), Vec3::new(1.2, 1.4, -25.0), 4.0),
                CameraKeyframe::new(Vec3::new(0.0, 5.5, 11.0), Vec3::new(0.0, 1.2, -24.0), 4.0),
            ],
            looping: true,
            elapsed: 0.0,
        }
    }

    pub fn equation_showcase() -> Self {
        Self {
            keyframes: vec![
                CameraKeyframe::new(Vec3::new(12.0, 9.0, 12.0), Vec3::ZERO, 4.5),
                CameraKeyframe::new(Vec3::new(-12.0, 11.0, 11.0), Vec3::ZERO, 4.5),
                CameraKeyframe::new(Vec3::new(-13.0, 6.5, -12.0), Vec3::ZERO, 4.5),
                CameraKeyframe::new(Vec3::new(13.0, 8.0, -11.0), Vec3::ZERO, 4.5),
            ],
            looping: true,
            elapsed: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Avanza el tiempo de la ruta y retorna `(posición, objetivo)`.
    pub fn sample(&mut self, dt: f32) -> (Vec3, Vec3) {
        if self.keyframes.is_empty() {
            return (Vec3::new(0.0, 4.0, 18.0), Vec3::ZERO);
        }
        if self.keyframes.len() == 1 {
            let kf = self.keyframes[0];
            return (Vec3::from_array(kf.position), Vec3::from_array(kf.target));
        }

        let total_duration: f32 = self.keyframes.iter().map(|k| k.duration).sum();
        self.elapsed += dt.max(0.0);
        if self.looping && total_duration > 0.0 {
            self.elapsed %= total_duration;
        } else {
            self.elapsed = self.elapsed.min(total_duration);
        }

        let mut acc = 0.0;
        let count = self.keyframes.len();
        for i in 0..count {
            let curr = self.keyframes[i];
            let next_idx = if i + 1 < count {
                i + 1
            } else if self.looping {
                0
            } else {
                i
            };
            let next = self.keyframes[next_idx];

            if self.elapsed <= acc + curr.duration || i + 1 == count {
                let local_t = ((self.elapsed - acc) / curr.duration).clamp(0.0, 1.0);
                // Smoothstep cúbico para transiciones suaves sin sacudidas
                let s = local_t * local_t * (3.0 - 2.0 * local_t);
                let pos = Vec3::from_array(curr.position).lerp(Vec3::from_array(next.position), s);
                let target = Vec3::from_array(curr.target).lerp(Vec3::from_array(next.target), s);
                return (pos, target);
            }
            acc += curr.duration;
        }

        let last = self.keyframes[count - 1];
        (Vec3::from_array(last.position), Vec3::from_array(last.target))
    }
}
