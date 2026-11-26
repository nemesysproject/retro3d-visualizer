pub mod path;

use glam::Vec3;
use serde::{Deserialize, Serialize};
use three_d::{vec3, Camera as ThreeDCamera};

use self::path::CameraPath;

/// Modo de control de la cámara 3D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraControlMode {
    Orbital,
    FreeFlight,
    AutoPath,
}

impl Default for CameraControlMode {
    fn default() -> Self {
        Self::Orbital
    }
}

/// Estado instantáneo de teclas de vuelo libre (WASD + Q/E).
#[derive(Debug, Default, Clone, Copy)]
pub struct FlightInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

impl FlightInput {
    pub fn any_active(self) -> bool {
        self.forward || self.backward || self.left || self.right || self.up || self.down
    }
}

/// Controlador de cámara retro compatible con modo orbital, vuelo libre y rutas automáticas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetroCamera {
    pub mode: CameraControlMode,
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub move_speed: f32,
    pub path: CameraPath,
}

impl Default for RetroCamera {
    fn default() -> Self {
        let mut cam = Self {
            mode: CameraControlMode::Orbital,
            position: [0.0, 4.0, 22.0],
            target: [0.0, 0.0, 0.0],
            yaw: 0.0,
            pitch: 0.18,
            distance: 22.5,
            move_speed: 14.0,
            path: CameraPath::lorenz_orbit(),
        };
        cam.recompute_orbital_position();
        cam
    }
}

impl RetroCamera {
    pub fn position_vec(&self) -> Vec3 {
        Vec3::from_array(self.position)
    }

    pub fn target_vec(&self) -> Vec3 {
        Vec3::from_array(self.target)
    }

    /// Restablece posición, orientación y ruta predeterminada según el modo activo.
    pub fn reset_for_scene(&mut self, scene_name: &str) {
        match scene_name {
            "terrain" => {
                self.mode = CameraControlMode::FreeFlight;
                self.position = [0.0, 3.4, 10.5];
                self.target = [0.0, 1.5, -24.0];
                self.yaw = -std::f32::consts::FRAC_PI_2;
                self.pitch = -0.06;
                self.distance = 34.5;
                self.path = CameraPath::terrain_flythrough();
            }
            "tunnel" => {
                self.mode = CameraControlMode::FreeFlight;
                self.position = [0.0, 0.0, 9.0];
                self.target = [0.0, 0.0, -30.0];
                self.yaw = -std::f32::consts::FRAC_PI_2;
                self.pitch = 0.0;
                self.distance = 39.0;
                self.path = CameraPath::terrain_flythrough();
            }
            "city" => {
                self.mode = CameraControlMode::FreeFlight;
                self.position = [0.0, 4.8, 11.5];
                self.target = [0.0, 3.2, -32.0];
                self.yaw = -std::f32::consts::FRAC_PI_2;
                self.pitch = -0.03;
                self.distance = 43.5;
                self.path = CameraPath::terrain_flythrough();
            }
            "equation" => {
                self.mode = CameraControlMode::Orbital;
                self.target = [0.0, 0.0, 0.0];
                self.yaw = 0.78;
                self.pitch = 0.52;
                self.distance = 19.0;
                self.recompute_orbital_position();
                self.path = CameraPath::equation_showcase();
            }
            "tesseract" | "lissajous" | "threebody" => {
                self.mode = CameraControlMode::Orbital;
                self.target = [0.0, 0.0, 0.0];
                self.yaw = 0.55;
                self.pitch = 0.30;
                self.distance = 18.0;
                self.recompute_orbital_position();
                self.path = CameraPath::lorenz_orbit();
            }
            "topology" | "dipole" | "attractors" => {
                self.mode = CameraControlMode::Orbital;
                self.target = [0.0, 0.0, 0.0];
                self.yaw = 0.45;
                self.pitch = 0.34;
                self.distance = 21.0;
                self.recompute_orbital_position();
                self.path = CameraPath::lorenz_orbit();
            }
            _ => {
                self.mode = CameraControlMode::Orbital;
                self.target = [0.0, 0.0, 0.0];
                self.yaw = 0.35;
                self.pitch = 0.22;
                self.distance = 22.0;
                self.recompute_orbital_position();
                self.path = CameraPath::lorenz_orbit();
            }
        }
    }

    fn recompute_orbital_position(&mut self) {
        let target = self.target_vec();
        let cp = self.pitch.cos();
        let offset = Vec3::new(
            self.distance * cp * self.yaw.sin(),
            self.distance * self.pitch.sin(),
            self.distance * cp * self.yaw.cos(),
        );
        self.position = (target + offset).to_array();
    }

    /// Rotación con clic izquierdo + arrastrar (orbital o mirada en vuelo libre).
    pub fn handle_left_drag(&mut self, dx: f32, dy: f32) {
        let sensitivity = 0.0065;
        match self.mode {
            CameraControlMode::Orbital => {
                self.yaw -= dx * sensitivity;
                self.pitch = (self.pitch + dy * sensitivity).clamp(-1.45, 1.45);
                self.recompute_orbital_position();
            }
            CameraControlMode::FreeFlight => {
                self.yaw += dx * sensitivity;
                self.pitch = (self.pitch - dy * sensitivity).clamp(-1.45, 1.45);
                let forward = self.free_flight_forward();
                self.target = (self.position_vec() + forward * 10.0).to_array();
            }
            CameraControlMode::AutoPath => {
                // Interacción manual cambia suavemente a orbital
                self.mode = CameraControlMode::Orbital;
                self.sync_spherical_from_vectors();
                self.handle_left_drag(dx, dy);
            }
        }
    }

    /// Desplazamiento lateral/vertical con clic derecho + arrastrar (Pan).
    pub fn handle_right_drag(&mut self, dx: f32, dy: f32) {
        let pos = self.position_vec();
        let target = self.target_vec();
        let forward = (target - pos).normalize_or_zero();
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();

        let pan_scale = (self.distance * 0.0022).max(0.01);
        let delta = (-right * dx + up * dy) * pan_scale;
        self.position = (pos + delta).to_array();
        self.target = (target + delta).to_array();
    }

    /// Acercamiento/alejamiento con la rueda del ratón.
    pub fn handle_zoom(&mut self, scroll_y: f32) {
        let factor = (1.0 - scroll_y * 0.08).clamp(0.5, 1.5);
        match self.mode {
            CameraControlMode::Orbital => {
                self.distance = (self.distance * factor).clamp(2.5, 120.0);
                self.recompute_orbital_position();
            }
            CameraControlMode::FreeFlight => {
                let forward = self.free_flight_forward();
                let step = scroll_y * 1.2;
                let new_pos = self.position_vec() + forward * step;
                self.position = new_pos.to_array();
                self.target = (new_pos + forward * 10.0).to_array();
            }
            CameraControlMode::AutoPath => {
                self.distance = (self.distance * factor).clamp(2.5, 120.0);
            }
        }
    }

    fn free_flight_forward(&self) -> Vec3 {
        let cp = self.pitch.cos();
        Vec3::new(cp * self.yaw.cos(), self.pitch.sin(), cp * self.yaw.sin()).normalize_or_zero()
    }

    fn sync_spherical_from_vectors(&mut self) {
        let offset = self.position_vec() - self.target_vec();
        self.distance = offset.length().max(1.0);
        self.pitch = (offset.y / self.distance).clamp(-0.99, 0.99).asin();
        self.yaw = offset.x.atan2(offset.z);
    }

    /// Actualiza el vuelo libre (WASD + Q/E) o la ruta automática.
    pub fn update(&mut self, dt: f32, input: FlightInput) {
        if input.any_active() {
            if self.mode == CameraControlMode::AutoPath {
                self.mode = CameraControlMode::FreeFlight;
                let dir = (self.target_vec() - self.position_vec()).normalize_or_zero();
                self.pitch = dir.y.clamp(-0.99, 0.99).asin();
                self.yaw = dir.z.atan2(dir.x);
            }

            let forward = match self.mode {
                CameraControlMode::FreeFlight => self.free_flight_forward(),
                _ => (self.target_vec() - self.position_vec()).normalize_or_zero(),
            };
            let right = forward.cross(Vec3::Y).normalize_or_zero();
            let mut velocity = Vec3::ZERO;

            if input.forward {
                velocity += forward;
            }
            if input.backward {
                velocity -= forward;
            }
            if input.right {
                velocity += right;
            }
            if input.left {
                velocity -= right;
            }
            if input.up {
                velocity += Vec3::Y;
            }
            if input.down {
                velocity -= Vec3::Y;
            }

            if velocity.length_squared() > 0.0 {
                let delta = velocity.normalize() * self.move_speed * dt;
                self.position = (self.position_vec() + delta).to_array();
                self.target = (self.target_vec() + delta).to_array();
            }
        } else if self.mode == CameraControlMode::AutoPath {
            let (pos, target) = self.path.sample(dt);
            self.position = pos.to_array();
            self.target = target.to_array();
        }
    }

    /// Sincroniza la cámara interna con la cámara de `three-d`.
    pub fn apply_to(&self, camera: &mut ThreeDCamera) {
        let p = self.position;
        let t = self.target;
        camera.set_view(vec3(p[0], p[1], p[2]), vec3(t[0], t[1], t[2]), vec3(0.0, 1.0, 0.0));
    }
}
