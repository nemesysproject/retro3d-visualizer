pub mod postprocess;
pub mod wireframe;

use glam::Vec3;
use serde::{Deserialize, Serialize};
use three_d::{Camera, ClearState, Context, RenderTarget, Viewport};

use self::postprocess::CrtPostProcessor;
use self::wireframe::WireframeRenderer;

/// Paletas monocromáticas retro especificadas para el visualizador.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetroPalette {
    /// Verde Fósforo (#00FF41)
    PhosphorGreen,
    /// Ámbar (#FFB000)
    Amber,
    /// Cian Vectorial (#00FFFF)
    VectorCyan,
}

impl Default for RetroPalette {
    fn default() -> Self {
        Self::PhosphorGreen
    }
}

impl RetroPalette {
    pub fn label(self) -> &'static str {
        match self {
            Self::PhosphorGreen => "Verde Fósforo (#00FF41)",
            Self::Amber => "Ámbar (#FFB000)",
            Self::VectorCyan => "Cian Vectorial (#00FFFF)",
        }
    }

    pub fn rgb_f32(self) -> [f32; 3] {
        match self {
            Self::PhosphorGreen => [0.0, 1.0, 0.255],
            Self::Amber => [1.0, 0.690, 0.0],
            Self::VectorCyan => [0.0, 1.0, 1.0],
        }
    }

    pub fn rgb_u8(self) -> [u8; 3] {
        match self {
            Self::PhosphorGreen => [0x00, 0xFF, 0x41],
            Self::Amber => [0xFF, 0xB0, 0x00],
            Self::VectorCyan => [0x00, 0xFF, 0xFF],
        }
    }
}

/// Parámetros estéticos de renderizado ajustables en tiempo real.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VisualConfig {
    pub palette: RetroPalette,
    pub line_width: f32,
    pub glow_intensity: f32,
    pub crt_enabled: bool,
    pub show_reference_grid: bool,
}

impl Default for VisualConfig {
    fn default() -> Self {
        Self {
            palette: RetroPalette::PhosphorGreen,
            line_width: 2.2,
            glow_intensity: 0.95,
            crt_enabled: true,
            show_reference_grid: true,
        }
    }
}

/// Orquestador del pipeline gráfico 3D retro (wireframe + postprocesado CRT).
pub struct RetroRenderer {
    pub wireframe: WireframeRenderer,
    pub postprocess: CrtPostProcessor,
}

impl RetroRenderer {
    pub fn new(context: &Context, viewport: Viewport) -> Result<Self, String> {
        let wireframe = WireframeRenderer::new(context)?;
        let postprocess = CrtPostProcessor::new(context, viewport.width, viewport.height)?;
        Ok(Self {
            wireframe,
            postprocess,
        })
    }

    /// Renderiza todos los segmentos en el buffer offscreen y aplica el filtro CRT sobre `screen`.
    pub fn render_frame(
        &mut self,
        context: &Context,
        screen: &RenderTarget<'_>,
        camera: &Camera,
        viewport: Viewport,
        segments: &[(Vec3, Vec3, f32)],
        visual: &VisualConfig,
        time: f32,
    ) {
        self.postprocess.ensure_size(context, viewport);
        self.wireframe.upload_segments(segments);

        let color = visual.palette.rgb_f32();
        let offscreen_vp = Viewport::new_at_origo(viewport.width.max(1), viewport.height.max(1));

        {
            let target = self.postprocess.offscreen_target();
            target.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0));
            let _ = target.write::<std::convert::Infallible>(|| {
                self.wireframe.render(
                    camera,
                    offscreen_vp,
                    color,
                    visual.line_width,
                    visual.glow_intensity,
                );
                Ok(())
            });
        }

        self.postprocess.composite_to(
            screen,
            viewport,
            color,
            visual.crt_enabled,
            visual.glow_intensity,
            time,
        );
    }
}
