use anyhow::Result;
use glam::Vec2;
mod types;
mod world;
mod atlas;
mod ui;
mod ui_gpu;
mod input;
mod config;
mod save;
mod path;
mod jobs;
mod controls;
mod ui_interaction;
mod game;
mod palette;
mod gpu_renderer;
mod weather;
mod camera;
mod console;
mod game_state;
mod event_handler;
mod game_loop;
mod render_prep;
use gpu_renderer::GpuRenderer;
use std::time::Instant;
use rand::{rngs::StdRng, SeedableRng};
use std::sync::atomic::{AtomicI32, Ordering};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

static MINIMAP_CELL_PX: AtomicI32 = AtomicI32::new(0);

type ResolvedInput = input::ResolvedInput;

fn main() -> Result<()> {
    run()
}

fn run() -> Result<()> {
    use std::sync::Arc;
    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Strategy Isometric Prototype")
        .with_inner_size(LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)?);

    env_logger::init();

    let size = window.inner_size();
    let mut gpu_renderer = pollster::block_on(GpuRenderer::new(window.clone()))?;
    gpu_renderer.load_faces_texture()?;
    let (config, input) = config::load_or_create("config.toml")?;
    let input = ResolvedInput::from(&input);

    let mut camera = camera::Camera::new(Vec2::new(0.0, 0.0), 2.0);
    let mut rng_init = StdRng::seed_from_u64(42);
    let mut game_state = game_state::GameState::new(&mut rng_init, &config);
    
    // Загрузить все текстуры
    atlas::load_textures(
        &mut game_state.atlas,
        &mut game_state.building_atlas,
        &mut game_state.tree_atlas,
    );
    game_state.width_i32 = size.width as i32;
    game_state.height_i32 = size.height as i32;

    let window = window.clone();
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if event_handler::handle_keyboard_input(
                        event.physical_key,
                        &event.state,
                        elwt,
                        &mut game_state,
                        &mut camera,
                        &input,
                        &config,
                    ) {
                        return;
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    event_handler::handle_cursor_moved(position, &mut game_state, &camera);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if event_handler::handle_mouse_input(button, state, &mut game_state, &config, &mut gpu_renderer) {
                        return;
                    }
                }
                WindowEvent::Resized(new_size) => {
                    event_handler::handle_resize(new_size, &mut game_state, &mut gpu_renderer);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    event_handler::handle_mouse_wheel(delta, &mut camera);
                }
                WindowEvent::RedrawRequested => {
                    if MINIMAP_CELL_PX.load(Ordering::Relaxed) == 0 {
                        let s0 = ui::ui_scale(game_state.height_i32, config.ui_scale_base);
                        MINIMAP_CELL_PX.store(3 * s0, Ordering::Relaxed);
                    }

                    let (min_tx, min_ty, max_tx, max_ty) = render_prep::prepare_rendering_data(&mut game_state, &camera, &mut gpu_renderer);
                    
                    // TODO: Реализовать GPU версию draw_debug_path для отладочного пути
                    if game_state.show_ui {
                        let visible = types::total_resources(&game_state.warehouses, &game_state.resources);
                        let stats = types::count_citizen_states(&game_state.citizens);
                        let idle = stats.idle;
                        let working = stats.working;
                        let sleeping = stats.sleeping;
                        let hauling = stats.hauling;
                        let fetching = stats.fetching;
                        let day_progress = (game_state.world_clock_ms / game_loop::DAY_LENGTH_MS).clamp(0.0, 1.0);
                        let avg_hap: f32 = if game_state.citizens.is_empty() { 50.0 } else { game_state.citizens.iter().map(|c| c.happiness as i32).sum::<i32>() as f32 / game_state.citizens.len() as f32 };
                        let pop_show = game_state.citizens.len() as i32;
                        let (wlabel, wcol) = game_state.weather_system.ui_label_and_color();
                        let hovered_building = if let Some(tp) = game_state.hovered_tile {
                            game_state.buildings.iter().find(|b| b.pos == tp).cloned()
                        } else {
                            None
                        };
                        
                        for building in &mut game_state.buildings {
                            building.is_highlighted = if let Some(ref hovered) = hovered_building {
                                building.pos == hovered.pos
                            } else {
                                false
                            };
                        }
                        
                        let hovered_button = if hovered_building.is_none() {
                            ui_interaction::get_hovered_button(
                                game_state.cursor_xy,
                                game_state.width_i32,
                                game_state.height_i32,
                                &config,
                                game_state.ui_category,
                                game_state.ui_tab,
                                game_state.paused,
                                game_state.speed_mult,
                                game_state.tax_rate,
                                game_state.food_policy,
                            )
                        } else {
                            None
                        };
                        
                        let hovered_resource = if hovered_building.is_none() && hovered_button.is_none() {
                            ui_interaction::get_hovered_resource(
                                game_state.cursor_xy,
                                game_state.width_i32,
                                game_state.height_i32,
                                &config,
                                &visible,
                                visible.wood,
                                pop_show,
                                avg_hap,
                                game_state.tax_rate,
                                idle,
                                working,
                                sleeping,
                                hauling,
                                fetching,
                            )
                        } else {
                            None
                        };
                        
                        let intensity = game_state.weather_system.intensity();
                        gpu_renderer.update_weather(game_state.weather_system.current(), game_state.world_clock_ms / 1000.0, intensity);
                        gpu_renderer.update_building_particles(&game_state.buildings, game_state.world_clock_ms / 1000.0);
                        let wcol_f32 = [wcol[0] as f32 / 255.0, wcol[1] as f32 / 255.0, wcol[2] as f32 / 255.0, wcol[3] as f32 / 255.0];
                ui_gpu::draw_ui_gpu(
                    &mut gpu_renderer,
                    game_state.width_i32,
                    game_state.height_i32,
                    &visible,
                    visible.wood,
                    pop_show,
                    game_state.selected_building,
                    game_state.fps_ema,
                    game_state.speed_mult,
                    game_state.paused,
                    config.ui_scale_base,
                    game_state.ui_category,
                    day_progress,
                    idle,
                    working,
                    sleeping,
                    hauling,
                    fetching,
                    avg_hap,
                    game_state.tax_rate,
                    game_state.ui_tab,
                    game_state.food_policy,
                    wlabel,
                    wcol_f32,
                    &mut game_state.world,
                    &game_state.buildings,
                    camera.pos.x,
                    camera.pos.y,
                    MINIMAP_CELL_PX.load(Ordering::Relaxed).max(1),
                    game_state.cursor_xy.x as f32,
                    game_state.cursor_xy.y as f32,
                    hovered_building,
                    hovered_button,
                    hovered_resource,
                    game_state.console.open,
                    &game_state.console.input,
                    &game_state.console.log,
                    game_state.biome_debug_mode,
                    game_state.show_deposits,
                    camera.zoom,
                    game_state.atlas.half_w,
                    game_state.atlas.half_h,
                    min_tx,
                    min_ty,
                    max_tx,
                    max_ty,
                );
            } else {
                gpu_renderer.clear_ui();
            }
                let t = (game_state.world_clock_ms / game_loop::DAY_LENGTH_MS).clamp(0.0, 1.0);
                let angle = t * std::f32::consts::TAU;
                let daylight = 0.5 - 0.5 * angle.cos();
                let darkness = (1.0 - daylight).max(0.0);
                let night_strength = (darkness.powf(1.4) * 180.0).min(200.0) as u8;
                let night_alpha = if night_strength > 0 {
                    night_strength as f32 / 255.0
                } else {
                    0.0
                };
                gpu_renderer.update_night_overlay(night_alpha);
                    
                    if let Err(err) = gpu_renderer.render() {
                        eprintln!("gpu_renderer.render() failed: {err}");
                        elwt.exit();
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                let now = Instant::now();
                let frame_ms = (now - game_state.last_frame).as_secs_f32() * 1000.0;
                game_state.last_frame = now;
                let frame_ms = frame_ms.min(250.0);
                
                game_loop::update_game_state(&mut game_state, frame_ms, &config);

                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
