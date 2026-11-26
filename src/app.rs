use std::path::Path;

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::camera::{FlightInput, RetroCamera};
use crate::math::demoscene::{
    generate_city_wireframe, generate_tesseract_wireframe, generate_tunnel_wireframe, CityParams,
    TesseractParams, TunnelParams,
};
use crate::math::lorenz::{LorenzParams, LorenzSystem};
use crate::math::parametric::{
    generate_equation_wireframe, generate_terrain_wireframe, EquationParams, TerrainParams,
};
use crate::math::physics::{
    generate_dipole_wireframe, generate_lissajous_wireframe, generate_topology_wireframe,
    AttractorGallerySystem, DipoleParams, GalleryParams, LissajousParams, ThreeBodyParams,
    ThreeBodySystem, TopologyParams,
};
use crate::render::wireframe::WireframeRenderer;
use crate::render::VisualConfig;

/// Modos de visualización soportados por la aplicación y CLI (`--mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    Lorenz,
    Terrain,
    Equation,
    Tesseract,
    Tunnel,
    City,
    Attractors,
    ThreeBody,
    Lissajous,
    Topology,
    Dipole,
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Lorenz
    }
}

impl AppMode {
    pub const ALL: [Self; 11] = [
        Self::Lorenz,
        Self::Terrain,
        Self::Equation,
        Self::Tesseract,
        Self::Tunnel,
        Self::City,
        Self::Attractors,
        Self::ThreeBody,
        Self::Lissajous,
        Self::Topology,
        Self::Dipole,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lorenz => "lorenz",
            Self::Terrain => "terrain",
            Self::Equation => "equation",
            Self::Tesseract => "tesseract",
            Self::Tunnel => "tunnel",
            Self::City => "city",
            Self::Attractors => "attractors",
            Self::ThreeBody => "threebody",
            Self::Lissajous => "lissajous",
            Self::Topology => "topology",
            Self::Dipole => "dipole",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Lorenz => "Atractor de Lorenz",
            Self::Terrain => "Terreno Wireframe 80s",
            Self::Equation => "Superficie z = f(x, y)",
            Self::Tesseract => "Hipercubo 4D (Teseracto)",
            Self::Tunnel => "Túnel Hiperespacial",
            Self::City => "Ciudad Wireframe 80s",
            Self::Attractors => "Galería de Atractores",
            Self::ThreeBody => "Problema de los 3 Cuerpos",
            Self::Lissajous => "Osciloscopio Lissajous 3D",
            Self::Topology => "Superficies Topológicas",
            Self::Dipole => "Campo Magnético Dipolar",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "lorenz" => Some(Self::Lorenz),
            "terrain" => Some(Self::Terrain),
            "equation" => Some(Self::Equation),
            "tesseract" | "tesseract_4d" => Some(Self::Tesseract),
            "tunnel" | "vector_tunnel" => Some(Self::Tunnel),
            "city" | "cyber_city" => Some(Self::City),
            "attractors" | "attractors_gallery" => Some(Self::Attractors),
            "threebody" | "three_body" => Some(Self::ThreeBody),
            "lissajous" | "lissajous_3d" => Some(Self::Lissajous),
            "topology" | "klein_mobius" => Some(Self::Topology),
            "dipole" | "dipole_field" => Some(Self::Dipole),
            _ => None,
        }
    }
}

/// Configuración de lanzamiento obtenida desde argumentos CLI o ejemplos.
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub title: String,
    pub mode: AppMode,
    pub preset_path: Option<String>,
    pub show_hud_initially: bool,
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            title: "Retro3D Visualizer — Vector Graphics Terminal".to_string(),
            mode: AppMode::Lorenz,
            preset_path: None,
            show_hud_initially: true,
        }
    }
}

impl LaunchConfig {
    /// Parsea los argumentos CLI:
    /// `--mode <modo>` y `--preset <archivo.json>`.
    pub fn from_cli_args() -> Self {
        let mut config = Self::default();
        let mut args = std::env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--mode" => {
                    if let Some(val) = args.next() {
                        if let Some(mode) = AppMode::parse(&val) {
                            config.mode = mode;
                        } else {
                            eprintln!(
                                "Modo desconocido '{val}'. Opciones: lorenz, terrain, equation, tesseract, tunnel, city, attractors, threebody, lissajous, topology, dipole."
                            );
                        }
                    }
                }
                "--preset" => {
                    if let Some(path) = args.next() {
                        config.preset_path = Some(path);
                    }
                }
                "--help" | "-h" => {
                    println!("Retro3D Visualizer");
                    println!("Uso:");
                    println!("  retro3d-visualizer [--mode <modo>] [--preset <archivo.json>]");
                    println!("Modos: lorenz, terrain, equation, tesseract, tunnel, city, attractors, threebody, lissajous, topology, dipole");
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        config
    }
}

/// Snapshot serializable a JSON para `--preset <archivo.json>` y botón de exportación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreset {
    pub mode: AppMode,
    pub lorenz: LorenzParams,
    pub equation: EquationParams,
    pub terrain: TerrainParams,
    #[serde(default)]
    pub tesseract: TesseractParams,
    #[serde(default)]
    pub tunnel: TunnelParams,
    #[serde(default)]
    pub city: CityParams,
    #[serde(default)]
    pub attractors: GalleryParams,
    #[serde(default)]
    pub three_body: ThreeBodyParams,
    #[serde(default)]
    pub lissajous: LissajousParams,
    #[serde(default)]
    pub topology: TopologyParams,
    #[serde(default)]
    pub dipole: DipoleParams,
    pub visual: VisualConfig,
    pub camera: RetroCamera,
}

/// Estado global de la aplicación en tiempo de ejecución.
pub struct AppState {
    pub mode: AppMode,
    pub lorenz: LorenzSystem,
    pub equation: EquationParams,
    pub terrain: TerrainParams,
    pub tesseract: TesseractParams,
    pub tunnel: TunnelParams,
    pub city: CityParams,
    pub attractors: AttractorGallerySystem,
    pub three_body: ThreeBodySystem,
    pub lissajous: LissajousParams,
    pub topology: TopologyParams,
    pub dipole: DipoleParams,
    pub visual: VisualConfig,
    pub camera: RetroCamera,
    pub flight_input: FlightInput,

    pub paused: bool,
    pub show_hud: bool,
    pub elapsed_time: f32,
    pub terrain_scroll: f32,
    pub city_scroll: f32,
    pub fps: f32,
    pub last_segment_count: usize,

    pub status_message: String,
    pub request_screenshot: bool,
    pub request_save_preset: bool,
}

impl AppState {
    pub fn from_launch_config(config: &LaunchConfig) -> Self {
        let mut camera = RetroCamera::default();
        camera.reset_for_scene(config.mode.as_str());

        let mut state = Self {
            mode: config.mode,
            lorenz: LorenzSystem::new(LorenzParams::default()),
            equation: EquationParams::default(),
            terrain: TerrainParams::default(),
            tesseract: TesseractParams::default(),
            tunnel: TunnelParams::default(),
            city: CityParams::default(),
            attractors: AttractorGallerySystem::new(GalleryParams::default()),
            three_body: ThreeBodySystem::new(ThreeBodyParams::default()),
            lissajous: LissajousParams::default(),
            topology: TopologyParams::default(),
            dipole: DipoleParams::default(),
            visual: VisualConfig::default(),
            camera,
            flight_input: FlightInput::default(),
            paused: false,
            show_hud: config.show_hud_initially,
            elapsed_time: 0.0,
            terrain_scroll: 0.0,
            city_scroll: 0.0,
            fps: 60.0,
            last_segment_count: 0,
            status_message: format!("Modo activo: {}", config.mode.label()),
            request_screenshot: false,
            request_save_preset: false,
        };

        if let Some(ref preset_path) = config.preset_path {
            match state.load_preset_from_file(Path::new(preset_path)) {
                Ok(()) => {
                    state.status_message = format!("Preset cargado: {preset_path}");
                }
                Err(err) => {
                    eprintln!("Error cargando preset '{preset_path}': {err}");
                    state.status_message = format!("Error cargando preset: {err}");
                }
            }
        }

        state
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        if self.mode != mode {
            self.mode = mode;
            self.camera.reset_for_scene(mode.as_str());
            self.status_message = format!("Modo cambiado a: {}", mode.label());
        }
    }

    /// Reinicia trayectoria y cámara (`R`).
    pub fn reset_current(&mut self) {
        self.camera.reset_for_scene(self.mode.as_str());
        match self.mode {
            AppMode::Lorenz => self.lorenz.reset(),
            AppMode::Terrain => self.terrain_scroll = 0.0,
            AppMode::City => self.city_scroll = 0.0,
            AppMode::Attractors => self.attractors.reset(),
            AppMode::ThreeBody => self.three_body.reset(),
            AppMode::Equation
            | AppMode::Tesseract
            | AppMode::Tunnel
            | AppMode::Lissajous
            | AppMode::Topology
            | AppMode::Dipole => self.elapsed_time = 0.0,
        }
        self.status_message = "Trayectoria y cámara reiniciadas.".to_string();
    }

    pub fn to_preset(&self) -> AppPreset {
        AppPreset {
            mode: self.mode,
            lorenz: self.lorenz.params,
            equation: self.equation,
            terrain: self.terrain,
            tesseract: self.tesseract,
            tunnel: self.tunnel,
            city: self.city,
            attractors: self.attractors.params,
            three_body: self.three_body.params,
            lissajous: self.lissajous,
            topology: self.topology,
            dipole: self.dipole,
            visual: self.visual,
            camera: self.camera.clone(),
        }
    }

    pub fn apply_preset(&mut self, preset: AppPreset) {
        self.mode = preset.mode;
        self.lorenz = LorenzSystem::new(preset.lorenz);
        self.equation = preset.equation;
        self.terrain = preset.terrain;
        self.tesseract = preset.tesseract;
        self.tunnel = preset.tunnel;
        self.city = preset.city;
        self.attractors = AttractorGallerySystem::new(preset.attractors);
        self.three_body = ThreeBodySystem::new(preset.three_body);
        self.lissajous = preset.lissajous;
        self.topology = preset.topology;
        self.dipole = preset.dipole;
        self.visual = preset.visual;
        self.camera = preset.camera;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_preset_from_file(&mut self, path: &Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("No se pudo leer {}: {e}", path.display()))?;
        let preset: AppPreset = serde_json::from_str(&content)
            .map_err(|e| format!("JSON inválido en {}: {e}", path.display()))?;
        self.apply_preset(preset);
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load_preset_from_file(&mut self, _path: &Path) -> Result<(), String> {
        Err("Carga desde disco no disponible en WebAssembly".to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_preset_to_file(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.to_preset())
            .map_err(|e| format!("Error serializando preset: {e}"))?;
        std::fs::write(path, json)
            .map_err(|e| format!("No se pudo escribir {}: {e}", path.display()))?;
        Ok(())
    }

    /// Actualiza simulación y cámara para un cuadro con paso `dt_seconds`.
    pub fn update(&mut self, dt_seconds: f32) {
        let dt = dt_seconds.clamp(0.0001, 0.1);
        let instant_fps = 1.0 / dt;
        self.fps = self.fps * 0.9 + instant_fps * 0.1;

        self.camera.update(dt, self.flight_input);

        if !self.paused {
            self.elapsed_time += dt;
            match self.mode {
                AppMode::Lorenz => self.lorenz.step(2),
                AppMode::Terrain => self.terrain_scroll += self.terrain.speed * dt,
                AppMode::City => self.city_scroll += self.city.speed * dt,
                AppMode::Attractors => self.attractors.step(2),
                AppMode::ThreeBody => self.three_body.step(4),
                AppMode::Equation
                | AppMode::Tesseract
                | AppMode::Tunnel
                | AppMode::Lissajous
                | AppMode::Topology
                | AppMode::Dipole => {}
            }
        }
    }

    /// Construye la geometría vectorial 3D del cuadro actual.
    pub fn build_scene_segments(&mut self) -> Vec<(Vec3, Vec3, f32)> {
        let mut segments = match self.mode {
            AppMode::Lorenz => self.lorenz.line_segments(),
            AppMode::Terrain => generate_terrain_wireframe(&self.terrain, self.terrain_scroll),
            AppMode::Equation => generate_equation_wireframe(&self.equation, self.elapsed_time),
            AppMode::Tesseract => generate_tesseract_wireframe(&self.tesseract, self.elapsed_time),
            AppMode::Tunnel => generate_tunnel_wireframe(&self.tunnel, self.elapsed_time),
            AppMode::City => generate_city_wireframe(&self.city, self.city_scroll),
            AppMode::Attractors => self.attractors.line_segments(),
            AppMode::ThreeBody => self.three_body.line_segments(),
            AppMode::Lissajous => generate_lissajous_wireframe(&self.lissajous, self.elapsed_time),
            AppMode::Topology => generate_topology_wireframe(&self.topology, self.elapsed_time),
            AppMode::Dipole => generate_dipole_wireframe(&self.dipole, self.elapsed_time),
        };

        let needs_grid = self.visual.show_reference_grid
            && !matches!(self.mode, AppMode::Terrain | AppMode::Tunnel | AppMode::City);
        if needs_grid {
            let y_level = match self.mode {
                AppMode::Lorenz | AppMode::Attractors => -8.5,
                AppMode::Equation => -4.5,
                _ => -7.5,
            };
            segments.extend(WireframeRenderer::reference_grid(14.0, 14, y_level));
        }

        self.last_segment_count = segments.len();
        segments
    }
}
