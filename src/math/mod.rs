pub mod demoscene;
pub mod lorenz;
pub mod parametric;
pub mod physics;

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Método de integración numérica para sistemas dinámicos 3D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Integrator {
    Euler,
    Rk4,
}

impl Default for Integrator {
    fn default() -> Self {
        Self::Rk4
    }
}

/// Integración explícita de Euler de primer orden.
#[inline]
pub fn integrate_euler<F>(pos: Vec3, dt: f32, deriv: F) -> Vec3
where
    F: Fn(Vec3) -> Vec3,
{
    pos + deriv(pos) * dt
}

/// Integración clásica de Runge-Kutta de cuarto orden (RK4).
#[inline]
pub fn integrate_rk4<F>(pos: Vec3, dt: f32, deriv: F) -> Vec3
where
    F: Fn(Vec3) -> Vec3,
{
    let k1 = deriv(pos);
    let k2 = deriv(pos + k1 * (0.5 * dt));
    let k3 = deriv(pos + k2 * (0.5 * dt));
    let k4 = deriv(pos + k3 * dt);
    pos + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * (dt / 6.0)
}

/// Despacha el método de integración seleccionado.
#[inline]
pub fn integrate<F>(method: Integrator, pos: Vec3, dt: f32, deriv: F) -> Vec3
where
    F: Fn(Vec3) -> Vec3,
{
    match method {
        Integrator::Euler => integrate_euler(pos, dt, deriv),
        Integrator::Rk4 => integrate_rk4(pos, dt, deriv),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rk4_is_more_accurate_than_euler_for_rotation() {
        let deriv = |p: Vec3| Vec3::new(-p.y, p.x, 0.0);
        let dt = 0.05;
        let mut p_euler = Vec3::new(1.0, 0.0, 0.0);
        let mut p_rk4 = Vec3::new(1.0, 0.0, 0.0);

        for _ in 0..100 {
            p_euler = integrate_euler(p_euler, dt, deriv);
            p_rk4 = integrate_rk4(p_rk4, dt, deriv);
        }

        let err_euler = (p_euler.length() - 1.0).abs();
        let err_rk4 = (p_rk4.length() - 1.0).abs();
        assert!(err_rk4 < err_euler);
        assert!(err_rk4 < 1e-4);
    }
}
