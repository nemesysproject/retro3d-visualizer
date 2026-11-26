use three_d::egui;

use crate::app::{AppMode, AppState};
use crate::camera::CameraControlMode;
use crate::math::demoscene::Polytope4D;
use crate::math::parametric::EquationKind;
use crate::math::physics::{ChaoticAttractorKind, ThreeBodyPreset, TopologyKind};
use crate::math::Integrator;
use crate::render::RetroPalette;

/// Dibuja el panel lateral de parámetros y telemetría estilo terminal CRT.
pub fn draw_hud(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_hud {
        return;
    }

    let [r, g, b] = state.visual.palette.rgb_u8();
    let accent = egui::Color32::from_rgb(r, g, b);
    let dim_accent = egui::Color32::from_rgb(r / 3, g / 3, b / 3);
    let bg_fill = egui::Color32::from_rgba_unmultiplied(6, 12, 8, 225);

    let mut style = (*ctx.style()).clone();
    style.override_font_id = Some(egui::FontId::monospace(13.0));
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = bg_fill;
    style.visuals.window_fill = bg_fill;
    style.visuals.override_text_color = Some(accent);
    style.visuals.selection.bg_fill = dim_accent;
    style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, accent);
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, dim_accent);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, dim_accent);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.2_f32, accent);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5_f32, accent);
    ctx.set_style(style);

    egui::SidePanel::left("retro_control_panel")
        .resizable(true)
        .default_width(325.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("RETRO3D // TERMINAL");
                ui.separator();

                // Telemetría en tiempo real
                let cam_pos = state.camera.position;
                ui.label(format!(
                    "FPS: {:>5.1}  |  Estado: {}",
                    state.fps,
                    if state.paused { "PAUSA" } else { "ACTIVO" }
                ));
                ui.label(format!("Segmentos: {}", state.last_segment_count));
                ui.label(format!(
                    "Cam: ({:.1}, {:.1}, {:.1})",
                    cam_pos[0], cam_pos[1], cam_pos[2]
                ));
                ui.separator();

                // Selector de modo principal (11 escenas)
                ui.label("MODO DE VISUALIZACIÓN");
                let mut selected_mode = state.mode;
                egui::ComboBox::from_id_source("scene_mode_selector")
                    .selected_text(state.mode.label())
                    .width(285.0)
                    .show_ui(ui, |ui| {
                        for mode in AppMode::ALL {
                            ui.selectable_value(&mut selected_mode, mode, mode.label());
                        }
                    });
                if selected_mode != state.mode {
                    state.set_mode(selected_mode);
                }

                ui.add_space(6.0);
                ui.label("CONTROL DE CÁMARA");
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(
                        &mut state.camera.mode,
                        CameraControlMode::Orbital,
                        "Orbital",
                    );
                    ui.selectable_value(
                        &mut state.camera.mode,
                        CameraControlMode::FreeFlight,
                        "Vuelo Libre",
                    );
                    ui.selectable_value(
                        &mut state.camera.mode,
                        CameraControlMode::AutoPath,
                        "Ruta Auto",
                    );
                });
                ui.separator();

                // Parámetros específicos del sistema activo
                match state.mode {
                    AppMode::Lorenz => {
                        ui.label("CONSTANTES DE LORENZ");
                        ui.add(
                            egui::Slider::new(&mut state.lorenz.params.sigma, 1.0..=30.0)
                                .text("σ (Sigma)"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.lorenz.params.rho, 5.0..=60.0)
                                .text("ρ (Rho)"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.lorenz.params.beta, 0.2..=8.0)
                                .text("β (Beta)"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.lorenz.params.dt, 0.001..=0.02)
                                .fixed_decimals(4)
                                .text("Δt"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.lorenz.params.particle_count, 1..=256)
                                .text("Partículas"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.lorenz.params.trail_length, 20..=320)
                                .text("Estela"),
                        );
                        ui.horizontal(|ui| {
                            ui.label("Integrador:");
                            ui.selectable_value(
                                &mut state.lorenz.params.integrator,
                                Integrator::Rk4,
                                "RK4",
                            );
                            ui.selectable_value(
                                &mut state.lorenz.params.integrator,
                                Integrator::Euler,
                                "Euler",
                            );
                        });
                    }
                    AppMode::Terrain => {
                        ui.label("PARÁMETROS DE TERRENO 80s");
                        ui.add(
                            egui::Slider::new(&mut state.terrain.speed, 0.0..=28.0)
                                .text("Velocidad"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.terrain.trench_width, 1.5..=12.0)
                                .text("Trinchera"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.terrain.mountain_height, 1.0..=16.0)
                                .text("Relieve"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.terrain.grid_width, 16..=60)
                                .text("Ancho malla"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.terrain.grid_depth, 24..=80)
                                .text("Profundidad"),
                        );
                    }
                    AppMode::Equation => {
                        ui.label("SUPERFICIE z = f(x, y)");
                        egui::ComboBox::from_label("Ecuación")
                            .selected_text(state.equation.kind.label())
                            .show_ui(ui, |ui| {
                                for kind in [
                                    EquationKind::Ripple,
                                    EquationKind::WaveGrid,
                                    EquationKind::Saddle,
                                    EquationKind::Interference,
                                ] {
                                    ui.selectable_value(
                                        &mut state.equation.kind,
                                        kind,
                                        kind.label(),
                                    );
                                }
                            });
                        ui.add(
                            egui::Slider::new(&mut state.equation.amplitude, 0.2..=6.0)
                                .text("Amplitud"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.equation.frequency, 0.2..=2.5)
                                .text("Frecuencia"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.equation.grid_size, 12..=64)
                                .text("Resolución"),
                        );
                        ui.checkbox(&mut state.equation.animate_phase, "Animar fase");
                    }
                    AppMode::Tesseract => {
                        ui.label("PROYECCIÓN 4D → 3D");
                        egui::ComboBox::from_label("Politopo")
                            .selected_text(state.tesseract.polytope.label())
                            .show_ui(ui, |ui| {
                                for p in [Polytope4D::Tesseract, Polytope4D::Cell16, Polytope4D::Cell5] {
                                    ui.selectable_value(&mut state.tesseract.polytope, p, p.label());
                                }
                            });
                        ui.add(
                            egui::Slider::new(&mut state.tesseract.speed_xw, -2.5..=2.5)
                                .text("Rotación XW"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.tesseract.speed_yw, -2.5..=2.5)
                                .text("Rotación YW"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.tesseract.speed_zw, -2.5..=2.5)
                                .text("Rotación ZW"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.tesseract.projection_distance, 2.2..=6.0)
                                .text("Distancia 4D"),
                        );
                    }
                    AppMode::Tunnel => {
                        ui.label("TÚNEL HIPERESPACIAL");
                        ui.add(
                            egui::Slider::new(&mut state.tunnel.sides, 4..=20).text("Lados"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.tunnel.rings, 12..=56).text("Anillos"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.tunnel.speed, 0.0..=36.0)
                                .text("Velocidad"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.tunnel.twist, -0.4..=0.4)
                                .text("Torsión"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.tunnel.sway_amplitude, 0.0..=6.0)
                                .text("Curvatura"),
                        );
                    }
                    AppMode::City => {
                        ui.label("CIUDAD WIREFRAME 80s");
                        ui.add(
                            egui::Slider::new(&mut state.city.speed, 0.0..=32.0)
                                .text("Velocidad"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.city.avenue_width, 3.5..=14.0)
                                .text("Avenida"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.city.max_building_height, 4.0..=26.0)
                                .text("Altura torres"),
                        );
                        ui.checkbox(&mut state.city.show_retro_sun, "Sol retro horizonte");
                    }
                    AppMode::Attractors => {
                        ui.label("GALERÍA DE ATRACTORES");
                        egui::ComboBox::from_label("Sistema")
                            .selected_text(state.attractors.params.kind.label())
                            .show_ui(ui, |ui| {
                                for k in [
                                    ChaoticAttractorKind::Aizawa,
                                    ChaoticAttractorKind::Rossler,
                                    ChaoticAttractorKind::Thomas,
                                ] {
                                    ui.selectable_value(
                                        &mut state.attractors.params.kind,
                                        k,
                                        k.label(),
                                    );
                                }
                            });
                        ui.add(
                            egui::Slider::new(&mut state.attractors.params.param_a, 0.4..=1.6)
                                .text("Parámetro A"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.attractors.params.param_b, 0.08..=0.35)
                                .text("Parámetro B"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.attractors.params.particle_count, 4..=180)
                                .text("Partículas"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.attractors.params.trail_length, 30..=300)
                                .text("Estela"),
                        );
                    }
                    AppMode::ThreeBody => {
                        ui.label("PROBLEMA DE LOS 3 CUERPOS");
                        egui::ComboBox::from_label("Órbita")
                            .selected_text(state.three_body.params.preset.label())
                            .show_ui(ui, |ui| {
                                for p in [
                                    ThreeBodyPreset::FigureEight,
                                    ThreeBodyPreset::LagrangeDance,
                                    ThreeBodyPreset::ChaoticSlingshot,
                                ] {
                                    ui.selectable_value(
                                        &mut state.three_body.params.preset,
                                        p,
                                        p.label(),
                                    );
                                }
                            });
                        ui.add(
                            egui::Slider::new(&mut state.three_body.params.gravity, 0.4..=2.0)
                                .text("Gravedad G"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.three_body.params.dt, 0.001..=0.015)
                                .fixed_decimals(4)
                                .text("Δt"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.three_body.params.trail_length, 40..=500)
                                .text("Estela"),
                        );
                    }
                    AppMode::Lissajous => {
                        ui.label("OSCILOSCOPIO LISSAJOUS 3D");
                        ui.add(
                            egui::Slider::new(&mut state.lissajous.freq_x, 1.0..=10.0)
                                .text("Frecuencia X"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.lissajous.freq_y, 1.0..=10.0)
                                .text("Frecuencia Y"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.lissajous.freq_z, 1.0..=10.0)
                                .text("Frecuencia Z"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.lissajous.phase_x, 0.0..=6.28)
                                .text("Fase X"),
                        );
                        ui.checkbox(&mut state.lissajous.animate_phase, "Deriva de fase");
                    }
                    AppMode::Topology => {
                        ui.label("SUPERFICIES TOPOLÓGICAS");
                        egui::ComboBox::from_label("Forma")
                            .selected_text(state.topology.kind.label())
                            .show_ui(ui, |ui| {
                                for k in [
                                    TopologyKind::KleinBottle,
                                    TopologyKind::MobiusStrip,
                                    TopologyKind::Torus,
                                ] {
                                    ui.selectable_value(&mut state.topology.kind, k, k.label());
                                }
                            });
                        ui.add(
                            egui::Slider::new(&mut state.topology.u_steps, 18..=72)
                                .text("Segmentos U"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.topology.v_steps, 10..=40)
                                .text("Segmentos V"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.topology.twist_speed, -2.0..=2.0)
                                .text("Rotación"),
                        );
                    }
                    AppMode::Dipole => {
                        ui.label("CAMPO MAGNÉTICO DIPOLAR");
                        ui.add(
                            egui::Slider::new(&mut state.dipole.radial_planes, 4..=24)
                                .text("Planos radiales"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.dipole.shells, 1..=5).text("Capas flujo"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.dipole.flow_speed, 0.0..=2.0)
                                .text("Vel. pulsos"),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.dipole.field_scale, 4.0..=14.0)
                                .text("Escala campo"),
                        );
                    }
                }

                ui.separator();

                // Estética Visual
                ui.label("ESTÉTICA VISUAL");
                for palette in [
                    RetroPalette::PhosphorGreen,
                    RetroPalette::Amber,
                    RetroPalette::VectorCyan,
                ] {
                    ui.radio_value(&mut state.visual.palette, palette, palette.label());
                }
                ui.add(
                    egui::Slider::new(&mut state.visual.line_width, 1.0..=6.0)
                        .text("Grosor línea"),
                );
                ui.add(
                    egui::Slider::new(&mut state.visual.glow_intensity, 0.0..=2.2)
                        .text("Intensidad brillo"),
                );
                ui.checkbox(&mut state.visual.crt_enabled, "Filtro CRT / Fósforo (F1)");
                ui.checkbox(&mut state.visual.show_reference_grid, "Grilla de referencia");

                ui.separator();

                // Acciones y Exportación
                ui.label("EXPORTACIÓN Y ACCIONES");
                ui.horizontal(|ui| {
                    if ui
                        .button(if state.paused {
                            "▶ Reanudar"
                        } else {
                            "⏸ Pausar"
                        })
                        .clicked()
                    {
                        state.paused = !state.paused;
                    }
                    if ui.button("↺ Reiniciar (R)").clicked() {
                        state.reset_current();
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("💾 Guardar JSON").clicked() {
                        state.request_save_preset = true;
                    }
                    if ui.button("📷 Captura limpia PNG").clicked() {
                        state.request_screenshot = true;
                    }
                });

                if !state.status_message.is_empty() {
                    ui.add_space(4.0);
                    ui.label(format!("> {}", state.status_message));
                }

                ui.separator();
                ui.label("CONTROLES RÁPIDOS");
                ui.small("Clic Izq: Rotar | Clic Der: Pan | Rueda: Zoom");
                ui.small("WASD + Q/E: Vuelo libre");
                ui.small("Espacio: Pausa | R: Reiniciar");
                ui.small("H: Ocultar HUD | F1: Filtro CRT");
            });
        });
}
