use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};

// ============================================================================
// 1. HIPERCUBO 4D (TESERACTO Y POLITOPOS)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Polytope4D {
    Tesseract,
    Cell16,
    Cell5,
}

impl Default for Polytope4D {
    fn default() -> Self {
        Self::Tesseract
    }
}

impl Polytope4D {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tesseract => "Teseracto (8-celdas / Hipercubo)",
            Self::Cell16 => "16-celdas (Hiperoctaedro)",
            Self::Cell5 => "5-celdas (Pentácoro / 4-símplex)",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TesseractParams {
    pub polytope: Polytope4D,
    pub speed_xw: f32,
    pub speed_yw: f32,
    pub speed_zw: f32,
    pub speed_xy: f32,
    pub projection_distance: f32,
    pub scale: f32,
}

impl Default for TesseractParams {
    fn default() -> Self {
        Self {
            polytope: Polytope4D::Tesseract,
            speed_xw: 0.75,
            speed_yw: 0.45,
            speed_zw: 0.30,
            speed_xy: 0.25,
            projection_distance: 3.1,
            scale: 5.5,
        }
    }
}

fn polytope_geometry(kind: Polytope4D) -> (Vec<Vec4>, Vec<(usize, usize)>) {
    match kind {
        Polytope4D::Tesseract => {
            let mut verts = Vec::with_capacity(16);
            for i in 0..16 {
                let x = if (i & 1) != 0 { 1.0 } else { -1.0 };
                let y = if (i & 2) != 0 { 1.0 } else { -1.0 };
                let z = if (i & 4) != 0 { 1.0 } else { -1.0 };
                let w = if (i & 8) != 0 { 1.0 } else { -1.0 };
                verts.push(Vec4::new(x, y, z, w));
            }
            let mut edges = Vec::with_capacity(32);
            for i in 0..16usize {
                for bit in 0..4 {
                    let j = i ^ (1 << bit);
                    if i < j {
                        edges.push((i, j));
                    }
                }
            }
            (verts, edges)
        }
        Polytope4D::Cell16 => {
            let s = 1.45;
            let verts = vec![
                Vec4::new(s, 0.0, 0.0, 0.0),
                Vec4::new(-s, 0.0, 0.0, 0.0),
                Vec4::new(0.0, s, 0.0, 0.0),
                Vec4::new(0.0, -s, 0.0, 0.0),
                Vec4::new(0.0, 0.0, s, 0.0),
                Vec4::new(0.0, 0.0, -s, 0.0),
                Vec4::new(0.0, 0.0, 0.0, s),
                Vec4::new(0.0, 0.0, 0.0, -s),
            ];
            let mut edges = Vec::with_capacity(24);
            for i in 0..8 {
                for j in (i + 1)..8 {
                    // Conectar todos los pares excepto los opuestos (0-1, 2-3, 4-5, 6-7)
                    if (i / 2) != (j / 2) {
                        edges.push((i, j));
                    }
                }
            }
            (verts, edges)
        }
        Polytope4D::Cell5 => {
            let a = 1.35;
            let verts = vec![
                Vec4::new(a, a, a, -a * 0.45),
                Vec4::new(a, -a, -a, -a * 0.45),
                Vec4::new(-a, a, -a, -a * 0.45),
                Vec4::new(-a, -a, a, -a * 0.45),
                Vec4::new(0.0, 0.0, 0.0, a * 1.8),
            ];
            let mut edges = Vec::with_capacity(10);
            for i in 0..5 {
                for j in (i + 1)..5 {
                    edges.push((i, j));
                }
            }
            (verts, edges)
        }
    }
}

#[inline]
fn rotate_plane(a: f32, b: f32, angle: f32) -> (f32, f32) {
    let (s, c) = angle.sin_cos();
    (a * c - b * s, a * s + b * c)
}

/// Genera las aristas proyectadas de un politopo 4D rotando en el espacio tetradimensional.
pub fn generate_tesseract_wireframe(params: &TesseractParams, time: f32) -> Vec<(Vec3, Vec3, f32)> {
    let (verts, edges) = polytope_geometry(params.polytope);
    let axw = time * params.speed_xw;
    let ayw = time * params.speed_yw;
    let azw = time * params.speed_zw;
    let axy = time * params.speed_xy;
    let dist = params.projection_distance.max(2.1);

    let projected: Vec<(Vec3, f32)> = verts
        .into_iter()
        .map(|mut v| {
            let (x, w) = rotate_plane(v.x, v.w, axw);
            v.x = x;
            v.w = w;

            let (y, w) = rotate_plane(v.y, v.w, ayw);
            v.y = y;
            v.w = w;

            let (z, w) = rotate_plane(v.z, v.w, azw);
            v.z = z;
            v.w = w;

            let (x, y) = rotate_plane(v.x, v.y, axy);
            v.x = x;
            v.y = y;

            let w_factor = 1.0 / (dist - v.w).max(0.45);
            let p3 = Vec3::new(v.x, v.y, v.z) * (w_factor * params.scale);
            let depth_w = ((v.w + 1.5) / 3.0).clamp(0.25, 1.0);
            (p3, depth_w)
        })
        .collect();

    let mut segments = Vec::with_capacity(edges.len() + 12);
    for (i, j) in edges {
        let (pa, wa) = projected[i];
        let (pb, wb) = projected[j];
        segments.push((pa, pb, (wa + wb) * 0.5));
    }

    segments
}

// ============================================================================
// 2. TÚNEL HIPERESPACIAL TIPO TEMPEST
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TunnelParams {
    pub sides: usize,
    pub rings: usize,
    pub radius: f32,
    pub speed: f32,
    pub twist: f32,
    pub sway_amplitude: f32,
}

impl Default for TunnelParams {
    fn default() -> Self {
        Self {
            sides: 10,
            rings: 34,
            radius: 6.5,
            speed: 14.0,
            twist: 0.14,
            sway_amplitude: 2.2,
        }
    }
}

/// Genera un túnel vectorial estilo arcade años 80 con anillos avanzando y torsión senoidal.
pub fn generate_tunnel_wireframe(params: &TunnelParams, time: f32) -> Vec<(Vec3, Vec3, f32)> {
    let sides = params.sides.clamp(4, 24);
    let rings = params.rings.clamp(8, 64);
    let spacing = 2.4;
    let total_depth = rings as f32 * spacing;
    let scroll = (time * params.speed) % spacing;

    let mut ring_verts: Vec<Vec<Vec3>> = Vec::with_capacity(rings);

    for r in 0..rings {
        let z = 6.0 - (r as f32 * spacing) + scroll;
        let depth_t = ((6.0 - z) / total_depth).clamp(0.0, 1.0);
        let world_z = (r as f32 * spacing) - scroll + time * params.speed;

        let cx = (world_z * 0.075).sin() * params.sway_amplitude * depth_t;
        let cy = (world_z * 0.055).cos() * params.sway_amplitude * 0.75 * depth_t;
        let angle_offset = world_z * params.twist + time * 0.35;
        let pulse = 1.0 + 0.12 * (world_z * 0.22).sin();
        let rad = params.radius * pulse;

        let mut pts = Vec::with_capacity(sides);
        for s in 0..sides {
            let theta = angle_offset + (s as f32 / sides as f32) * std::f32::consts::TAU;
            pts.push(Vec3::new(cx + rad * theta.cos(), cy + rad * theta.sin(), z));
        }
        ring_verts.push(pts);
    }

    let mut segments = Vec::with_capacity(rings * sides * 2);
    for r in 0..rings {
        let fade = 1.0 - (r as f32 / (rings - 1) as f32).powf(1.25);
        let intensity = (0.14 + 0.86 * fade).clamp(0.14, 1.0);
        for s in 0..sides {
            let next_s = (s + 1) % sides;
            segments.push((ring_verts[r][s], ring_verts[r][next_s], intensity));
            if r + 1 < rings {
                segments.push((ring_verts[r][s], ring_verts[r + 1][s], intensity * 0.8));
            }
        }
    }

    segments
}

// ============================================================================
// 3. CIUDAD WIREFRAME Y HORIZONTE SYNTHWAVE
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CityParams {
    pub speed: f32,
    pub avenue_width: f32,
    pub blocks_depth: usize,
    pub max_building_height: f32,
    pub show_retro_sun: bool,
}

impl Default for CityParams {
    fn default() -> Self {
        Self {
            speed: 12.0,
            avenue_width: 6.5,
            blocks_depth: 22,
            max_building_height: 15.0,
            show_retro_sun: true,
        }
    }
}

#[inline]
fn hash_u32(mut x: u32) -> f32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    (x as f32) / (u32::MAX as f32)
}

fn push_wire_box(
    segments: &mut Vec<(Vec3, Vec3, f32)>,
    center_x: f32,
    center_z: f32,
    half_w: f32,
    half_d: f32,
    height: f32,
    intensity: f32,
) {
    let b0 = Vec3::new(center_x - half_w, 0.0, center_z - half_d);
    let b1 = Vec3::new(center_x + half_w, 0.0, center_z - half_d);
    let b2 = Vec3::new(center_x + half_w, 0.0, center_z + half_d);
    let b3 = Vec3::new(center_x - half_w, 0.0, center_z + half_d);

    let t0 = Vec3::new(b0.x, height, b0.z);
    let t1 = Vec3::new(b1.x, height, b1.z);
    let t2 = Vec3::new(b2.x, height, b2.z);
    let t3 = Vec3::new(b3.x, height, b3.z);

    // Base y techo
    segments.push((b0, b1, intensity * 0.7));
    segments.push((b1, b2, intensity * 0.7));
    segments.push((b2, b3, intensity * 0.7));
    segments.push((b3, b0, intensity * 0.7));

    segments.push((t0, t1, intensity));
    segments.push((t1, t2, intensity));
    segments.push((t2, t3, intensity));
    segments.push((t3, t0, intensity));

    // Pilares verticales
    segments.push((b0, t0, intensity * 0.9));
    segments.push((b1, t1, intensity * 0.9));
    segments.push((b2, t2, intensity * 0.9));
    segments.push((b3, t3, intensity * 0.9));
}

/// Genera una ciudad wireframe en movimiento infinito con avenida central y sol retro en el horizonte.
pub fn generate_city_wireframe(params: &CityParams, scroll_z: f32) -> Vec<(Vec3, Vec3, f32)> {
    let blocks = params.blocks_depth.clamp(8, 36);
    let block_spacing = 4.6;
    let snapped_idx = (scroll_z / block_spacing).floor() as i32;
    let frac_z = scroll_z - (snapped_idx as f32 * block_spacing);
    let horizon_z = 8.0 - (blocks as f32 * block_spacing);

    let mut segments = Vec::with_capacity(blocks * 64 + 120);

    // Grilla de avenida central
    let half_road = params.avenue_width * 0.5;
    for lane in -2_i32..=2_i32 {
        let x = lane as f32 * (half_road * 0.5);
        let inten = if lane.abs() == 2 { 0.85 } else { 0.35 };
        segments.push((Vec3::new(x, 0.0, 10.0), Vec3::new(x, 0.0, horizon_z), inten));
    }

    for r in 0..blocks {
        let z = 8.0 - (r as f32 * block_spacing) + frac_z;
        let depth_fade = 1.0 - (r as f32 / (blocks - 1) as f32).powf(1.3);
        let inten = (0.14 + 0.86 * depth_fade).clamp(0.14, 1.0);

        // Líneas transversales de la avenida
        segments.push((
            Vec3::new(-half_road, 0.0, z),
            Vec3::new(half_road, 0.0, z),
            inten * 0.55,
        ));

        let world_row = (snapped_idx + r as i32) as u32;
        // Dos filas de edificios a cada lado de la avenida
        for side in [-1.0_f32, 1.0_f32] {
            for col in 0..2u32 {
                let seed = world_row
                    .wrapping_mul(374_761_393)
                    .wrapping_add(col.wrapping_mul(668_265_263))
                    .wrapping_add(if side > 0.0 { 17 } else { 91 });
                let h_rand = hash_u32(seed);
                let w_rand = hash_u32(seed.wrapping_add(13));
                let height = 2.2 + h_rand.powf(1.2) * params.max_building_height;
                let half_w = 1.1 + w_rand * 0.85;
                let half_d = 1.2 + hash_u32(seed.wrapping_add(29)) * 0.7;
                let cx = side * (half_road + 2.4 + col as f32 * 4.8);
                push_wire_box(&mut segments, cx, z, half_w, half_d, height, inten);
            }
        }
    }

    // Sol retro con franjas horizontales sobre el horizonte
    if params.show_retro_sun {
        let sun_z = horizon_z - 8.0;
        let sun_cy = 12.5;
        let sun_r = 11.5;
        let circle_steps = 40;
        for i in 0..circle_steps {
            let a0 = (i as f32 / circle_steps as f32) * std::f32::consts::TAU;
            let a1 = ((i + 1) as f32 / circle_steps as f32) * std::f32::consts::TAU;
            let p0 = Vec3::new(sun_r * a0.cos(), sun_cy + sun_r * a0.sin(), sun_z);
            let p1 = Vec3::new(sun_r * a1.cos(), sun_cy + sun_r * a1.sin(), sun_z);
            if p0.y > 0.5 && p1.y > 0.5 {
                segments.push((p0, p1, 0.95));
            }
        }
        // Franjas características en la mitad inferior del sol
        for stripe in 0..8 {
            let frac = -0.82 + stripe as f32 * 0.21;
            let y = sun_cy + sun_r * frac;
            if y > 0.8 {
                let dx = sun_r * (1.0 - frac * frac).max(0.0).sqrt();
                segments.push((
                    Vec3::new(-dx, y, sun_z),
                    Vec3::new(dx, y, sun_z),
                    0.75,
                ));
            }
        }
    }

    segments
}
