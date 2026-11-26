pub mod app;
pub mod camera;
pub mod math;
pub mod render;
pub mod ui;

use three_d::{
    degrees, vec3, Camera, Event, FrameOutput, Key, MouseButton, Window, WindowSettings, GUI,
};

use crate::app::{AppState, LaunchConfig};
use crate::render::RetroRenderer;

/// Punto de entrada compartido para la aplicación principal y los ejemplos independientes.
pub fn run_visualizer(config: LaunchConfig) {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let window = Window::new(WindowSettings {
        title: config.title.clone(),
        max_size: Some((1440, 900)),
        ..Default::default()
    })
    .expect("No se pudo inicializar la ventana gráfica");

    let context = window.gl();
    let initial_viewport = window.viewport();

    let mut camera = Camera::new_perspective(
        initial_viewport,
        vec3(0.0, 4.0, 22.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(50.0),
        0.1,
        300.0,
    );

    let mut renderer =
        RetroRenderer::new(&context, initial_viewport).expect("Error inicializando RetroRenderer");
    let mut gui = GUI::new(&context);
    let mut state = AppState::from_launch_config(&config);
    let mut left_dragging = false;
    let mut right_dragging = false;
    let mut capture_counter: u32 = 1;

    window.render_loop(move |mut frame_input| {
        let mut gui_wants_pointer = false;
        let mut gui_wants_keyboard = false;
        let mut f1_pressed = false;

        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                f1_pressed = gui_context.input(|i| i.key_pressed(three_d::egui::Key::F1));
                ui::draw_hud(gui_context, &mut state);
                gui_wants_pointer = gui_context.wants_pointer_input();
                gui_wants_keyboard = gui_context.wants_keyboard_input();
            },
        );

        if f1_pressed {
            state.visual.crt_enabled = !state.visual.crt_enabled;
            state.status_message = if state.visual.crt_enabled {
                "Filtro CRT / Fósforo ACTIVADO.".to_string()
            } else {
                "Filtro CRT / Fósforo DESACTIVADO.".to_string()
            };
        }

        // Procesar eventos de teclado y ratón
        for event in &frame_input.events {
            match *event {
                Event::MousePress {
                    button, handled, ..
                } if !handled && !gui_wants_pointer => match button {
                    MouseButton::Left => left_dragging = true,
                    MouseButton::Right => right_dragging = true,
                    _ => {}
                },
                Event::MouseRelease { button, .. } => match button {
                    MouseButton::Left => left_dragging = false,
                    MouseButton::Right => right_dragging = false,
                    _ => {}
                },
                Event::MouseMotion { delta, handled, .. } if !handled && !gui_wants_pointer => {
                    if left_dragging {
                        state.camera.handle_left_drag(delta.0, delta.1);
                    } else if right_dragging {
                        state.camera.handle_right_drag(delta.0, delta.1);
                    }
                }
                Event::MouseWheel { delta, handled, .. } if !handled && !gui_wants_pointer => {
                    state.camera.handle_zoom(delta.1);
                }
                Event::KeyPress { kind, handled, .. } if !handled && !gui_wants_keyboard => {
                    match kind {
                        Key::W => state.flight_input.forward = true,
                        Key::S => state.flight_input.backward = true,
                        Key::A => state.flight_input.left = true,
                        Key::D => state.flight_input.right = true,
                        Key::Q => state.flight_input.down = true,
                        Key::E => state.flight_input.up = true,
                        Key::Space => {
                            state.paused = !state.paused;
                            state.status_message = if state.paused {
                                "Simulación en pausa.".to_string()
                            } else {
                                "Simulación reanudada.".to_string()
                            };
                        }
                        Key::R => state.reset_current(),
                        Key::H => state.show_hud = !state.show_hud,
                        _ => {}
                    }
                }
                Event::KeyRelease { kind, .. } => match kind {
                    Key::W => state.flight_input.forward = false,
                    Key::S => state.flight_input.backward = false,
                    Key::A => state.flight_input.left = false,
                    Key::D => state.flight_input.right = false,
                    Key::Q => state.flight_input.down = false,
                    Key::E => state.flight_input.up = false,
                    _ => {}
                },
                _ => {}
            }
        }

        // Actualizar simulación y cámara
        let dt_seconds = (frame_input.elapsed_time * 0.001) as f32;
        state.update(dt_seconds);

        camera.set_viewport(frame_input.viewport);
        state.camera.apply_to(&mut camera);

        let segments = state.build_scene_segments();
        let screen = frame_input.screen();
        renderer.render_frame(
            &context,
            &screen,
            &camera,
            frame_input.viewport,
            &segments,
            &state.visual,
            state.elapsed_time,
        );

        // Exportación de preset JSON
        if state.request_save_preset {
            state.request_save_preset = false;
            #[cfg(not(target_arch = "wasm32"))]
            {
                let filename = format!("preset_{}.json", state.mode.as_str());
                match state.save_preset_to_file(std::path::Path::new(&filename)) {
                    Ok(()) => {
                        state.status_message = format!("Preset guardado en {filename}");
                    }
                    Err(err) => {
                        state.status_message = format!("Error guardando JSON: {err}");
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                state.status_message = "Exportación JSON solo disponible en nativo.".to_string();
            }
        }

        // Captura de pantalla limpia sin HUD
        if state.request_screenshot {
            state.request_screenshot = false;
            #[cfg(not(target_arch = "wasm32"))]
            {
                let filename = format!(
                    "retro3d_{}_{:03}.png",
                    state.mode.as_str(),
                    capture_counter
                );
                capture_counter += 1;
                match renderer.postprocess.save_clean_screenshot(
                    &context,
                    frame_input.viewport,
                    state.visual.palette.rgb_f32(),
                    state.visual.crt_enabled,
                    state.visual.glow_intensity,
                    state.elapsed_time,
                    std::path::Path::new(&filename),
                ) {
                    Ok(()) => {
                        state.status_message = format!("Captura limpia guardada: {filename}");
                    }
                    Err(err) => {
                        state.status_message = format!("Error en captura: {err}");
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = capture_counter;
                state.status_message = "Captura PNG solo disponible en nativo.".to_string();
            }
        }

        // Renderizar interfaz HUD encima de la escena CRT
        if state.show_hud {
            screen
                .write(|| gui.render())
                .expect("Error renderizando interfaz egui");
        }

        FrameOutput::default()
    });
}
