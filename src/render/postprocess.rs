use three_d::{
    vec2, vec3, ClearState, Context, DepthTexture2D, Interpolation, Program, RenderStates,
    RenderTarget, Texture2D, VertexBuffer, Viewport, Wrapping,
};

const FULLSCREEN_VERT: &str = r#"in vec2 a_pos;
out vec2 v_uv;
void main() {
    v_uv = a_pos * 0.5 + 0.5;
    gl_Position = vec4(a_pos, 0.0, 1.0);
}
"#;

const CRT_FRAG: &str = include_str!("../../assets/shaders/crt_postprocess.frag");

/// Pipeline de postprocesado CRT: buffer offscreen + scanlines + resplandor + curvatura.
pub struct CrtPostProcessor {
    program: Program,
    quad_buffer: VertexBuffer,
    color_tex: Texture2D,
    depth_tex: DepthTexture2D,
    width: u32,
    height: u32,
}

impl CrtPostProcessor {
    pub fn new(context: &Context, width: u32, height: u32) -> Result<Self, String> {
        let program = Program::from_source(context, FULLSCREEN_VERT, CRT_FRAG)
            .map_err(|e| format!("Error compilando shader CRT: {e:?}"))?;

        let quad_vertices = vec![
            vec2(-1.0, -1.0),
            vec2(1.0, -1.0),
            vec2(1.0, 1.0),
            vec2(-1.0, -1.0),
            vec2(1.0, 1.0),
            vec2(-1.0, 1.0),
        ];
        let quad_buffer = VertexBuffer::new_with_data(context, &quad_vertices);

        let w = width.max(1);
        let h = height.max(1);
        let (color_tex, depth_tex) = Self::create_targets(context, w, h);

        Ok(Self {
            program,
            quad_buffer,
            color_tex,
            depth_tex,
            width: w,
            height: h,
        })
    }

    fn create_targets(context: &Context, width: u32, height: u32) -> (Texture2D, DepthTexture2D) {
        let color_tex = Texture2D::new_empty::<[u8; 4]>(
            context,
            width,
            height,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let depth_tex = DepthTexture2D::new::<f32>(
            context,
            width,
            height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        (color_tex, depth_tex)
    }

    pub fn ensure_size(&mut self, context: &Context, viewport: Viewport) {
        let w = viewport.width.max(1);
        let h = viewport.height.max(1);
        if w != self.width || h != self.height {
            let (c, d) = Self::create_targets(context, w, h);
            self.color_tex = c;
            self.depth_tex = d;
            self.width = w;
            self.height = h;
        }
    }

    /// Retorna el `RenderTarget` offscreen para renderizar la escena 3D antes del filtro CRT.
    pub fn offscreen_target(&mut self) -> RenderTarget<'_> {
        RenderTarget::new(
            self.color_tex.as_color_target(None),
            self.depth_tex.as_depth_target(),
        )
    }

    /// Dibuja el cuadro offscreen sobre el destino indicado aplicando el shader CRT/fósforo.
    pub fn composite_to(
        &self,
        target: &RenderTarget<'_>,
        viewport: Viewport,
        phosphor_color: [f32; 3],
        crt_enabled: bool,
        glow_intensity: f32,
        time: f32,
    ) {
        target.clear(ClearState::color_and_depth(0.01, 0.015, 0.012, 1.0, 1.0));
        let _ = target.write::<std::convert::Infallible>(|| {
            self.program.use_texture("u_scene", &self.color_tex);
            self.program.use_uniform(
                "u_resolution",
                vec2(viewport.width as f32, viewport.height as f32),
            );
            self.program.use_uniform(
                "u_phosphor_color",
                vec3(phosphor_color[0], phosphor_color[1], phosphor_color[2]),
            );
            self.program
                .use_uniform("u_crt_enabled", if crt_enabled { 1.0f32 } else { 0.0f32 });
            self.program.use_uniform("u_glow_intensity", glow_intensity);
            self.program.use_uniform("u_time", time);
            self.program.use_vertex_attribute("a_pos", &self.quad_buffer);

            self.program
                .draw_arrays(RenderStates::default(), viewport, 6);
            Ok(())
        });
    }

    /// Captura la imagen final sin HUD en formato PNG (solo en nativo).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_clean_screenshot(
        &self,
        context: &Context,
        viewport: Viewport,
        phosphor_color: [f32; 3],
        crt_enabled: bool,
        glow_intensity: f32,
        time: f32,
        path: &std::path::Path,
    ) -> Result<(), String> {
        let w = viewport.width.max(1);
        let h = viewport.height.max(1);
        let mut capture_tex = Texture2D::new_empty::<[u8; 4]>(
            context,
            w,
            h,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let mut capture_depth = DepthTexture2D::new::<f32>(
            context,
            w,
            h,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let rt = RenderTarget::new(
            capture_tex.as_color_target(None),
            capture_depth.as_depth_target(),
        );
        let cap_vp = Viewport::new_at_origo(w, h);
        self.composite_to(&rt, cap_vp, phosphor_color, crt_enabled, glow_intensity, time);

        let pixels: Vec<[u8; 4]> = rt.read_color();
        // Voltear verticalmente (OpenGL tiene origen inferior-izquierdo, PNG superior-izquierdo)
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for row in (0..h as usize).rev() {
            let start = row * w as usize;
            let end = start + w as usize;
            for px in &pixels[start..end] {
                rgba.extend_from_slice(px);
            }
        }

        let file = std::fs::File::create(path)
            .map_err(|e| format!("No se pudo crear {}: {e}", path.display()))?;
        let writer = std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(writer, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut png_writer = encoder
            .write_header()
            .map_err(|e| format!("Error escribiendo cabecera PNG: {e}"))?;
        png_writer
            .write_image_data(&rgba)
            .map_err(|e| format!("Error escribiendo datos PNG: {e}"))?;

        Ok(())
    }
}
