use glam::Vec3 as GlamVec3;
use three_d::{
    vec2, vec3, Blend, Camera, Context, DepthTest, Program, RenderStates, VertexBuffer, Viewport,
    WriteMask,
};

const WIREFRAME_VERT: &str = include_str!("../../assets/shaders/wireframe.vert");
const WIREFRAME_FRAG: &str = include_str!("../../assets/shaders/wireframe.frag");

/// Renderizador de líneas vectoriales 3D con grosor real en espacio de pantalla
/// y brillo estilo osciloscopio / monitor vectorial retro.
pub struct WireframeRenderer {
    program: Program,
    pos_buffer: VertexBuffer,
    other_buffer: VertexBuffer,
    meta_buffer: VertexBuffer,
    vertex_count: u32,
}

impl WireframeRenderer {
    pub fn new(context: &Context) -> Result<Self, String> {
        let program = Program::from_source(context, WIREFRAME_VERT, WIREFRAME_FRAG)
            .map_err(|e| format!("Error compilando shader wireframe: {e:?}"))?;

        Ok(Self {
            program,
            pos_buffer: VertexBuffer::new(context),
            other_buffer: VertexBuffer::new(context),
            meta_buffer: VertexBuffer::new(context),
            vertex_count: 0,
        })
    }

    /// Sube una lista de segmentos `(inicio, fin, intensidad)` como quads de pantalla.
    pub fn upload_segments(&mut self, segments: &[(GlamVec3, GlamVec3, f32)]) {
        let count = segments.len();
        let mut positions = Vec::with_capacity(count * 6);
        let mut others = Vec::with_capacity(count * 6);
        let mut metas = Vec::with_capacity(count * 6);

        for &(a, b, intensity) in segments {
            let va = vec3(a.x, a.y, a.z);
            let vb = vec3(b.x, b.y, b.z);
            let inten = intensity.clamp(0.05, 1.0);

            // Triángulo 1: (a, -1), (a, +1), (b, -1)
            positions.push(va);
            others.push(vb);
            metas.push(vec2(-1.0, inten));

            positions.push(va);
            others.push(vb);
            metas.push(vec2(1.0, inten));

            positions.push(vb);
            others.push(va);
            metas.push(vec2(1.0, inten));

            // Triángulo 2: (a, +1), (b, +1), (b, -1)
            positions.push(va);
            others.push(vb);
            metas.push(vec2(1.0, inten));

            positions.push(vb);
            others.push(va);
            metas.push(vec2(-1.0, inten));

            positions.push(vb);
            others.push(va);
            metas.push(vec2(1.0, inten));
        }

        self.pos_buffer.fill(&positions);
        self.other_buffer.fill(&others);
        self.meta_buffer.fill(&metas);
        self.vertex_count = positions.len() as u32;
    }

    /// Dibuja todos los segmentos cargados con mezcla aditiva estilo fósforo.
    pub fn render(
        &self,
        camera: &Camera,
        viewport: Viewport,
        color: [f32; 3],
        line_width: f32,
        glow: f32,
    ) {
        if self.vertex_count == 0 {
            return;
        }

        let view_proj = camera.projection() * camera.view();
        self.program.use_uniform("u_view_proj", view_proj);
        self.program.use_uniform(
            "u_viewport",
            vec2(viewport.width as f32, viewport.height as f32),
        );
        self.program.use_uniform("u_line_width", line_width);
        self.program
            .use_uniform("u_color", vec3(color[0], color[1], color[2]));
        self.program.use_uniform("u_glow", glow);

        self.program.use_vertex_attribute("a_pos", &self.pos_buffer);
        self.program
            .use_vertex_attribute("a_other", &self.other_buffer);
        self.program
            .use_vertex_attribute("a_meta", &self.meta_buffer);

        let render_states = RenderStates {
            write_mask: WriteMask::COLOR,
            depth_test: DepthTest::LessOrEqual,
            blend: Blend::ADD,
            ..Default::default()
        };

        self.program
            .draw_arrays(render_states, viewport, self.vertex_count);
    }

    /// Genera una grilla de referencia en el plano XZ para orientar el espacio 3D.
    pub fn reference_grid(half_extent: f32, divisions: usize, y_level: f32) -> Vec<(GlamVec3, GlamVec3, f32)> {
        let n = divisions.max(2);
        let step = (half_extent * 2.0) / n as f32;
        let mut lines = Vec::with_capacity((n + 1) * 2);

        for i in 0..=n {
            let coord = -half_extent + i as f32 * step;
            let is_axis = (coord.abs()) < step * 0.1;
            let intensity = if is_axis { 0.45 } else { 0.2 };
            lines.push((
                GlamVec3::new(-half_extent, y_level, coord),
                GlamVec3::new(half_extent, y_level, coord),
                intensity,
            ));
            lines.push((
                GlamVec3::new(coord, y_level, -half_extent),
                GlamVec3::new(coord, y_level, half_extent),
                intensity,
            ));
        }
        lines
    }
}
