use anyhow::Result;
use glam::Vec2;
mod types; use types::Resources;
mod world;
mod atlas; use atlas::BuildingAtlas;
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
    
    if let Ok(img) = image::open("assets/spritesheet.png") {
        let img = img.to_rgba8();
        let (iw, ih) = img.dimensions();
        let cell_w = 32u32; let cell_h = 32u32;
        let cols = (iw / cell_w).max(1); let rows = (ih / cell_h).max(1);
        let cell_rgba = |cx: u32, cy: u32| -> Vec<u8> {
            let x0 = cx * cell_w; let y0 = cy * cell_h;
            let mut out = vec![0u8; (cell_w * cell_h * 4) as usize];
            for y in 0..cell_h as usize {
                let src = ((y0 as usize + y) * iw as usize + x0 as usize) * 4;
                let dst = y * cell_w as usize * 4;
                out[dst..dst + cell_w as usize * 4].copy_from_slice(&img.as_raw()[src..src + cell_w as usize * 4]);
            }
            out
        };
        // row 2 — вариации травы, последний ряд — вода (первая ячейка чистая, остальные — кромки)
        let grass_row = 2u32.min(rows-1);
        let mut grass_variants_raw: Vec<Vec<u8>> = Vec::new();
        let grass_cols = cols.min(3);
        for cx in 0..grass_cols { grass_variants_raw.push(cell_rgba(cx, grass_row)); }
        let water_row = rows-1;
        let water_full = cell_rgba(0, water_row);
        let mut water_edges_raw: Vec<Vec<u8>> = Vec::new();
        for cx in 1..=7 { if cx < cols { water_edges_raw.push(cell_rgba(cx, water_row)); } }
        let clay_row = 1u32.min(rows-1);
        let clay_cols = cols.min(3);
        let mut clay_variants_raw: Vec<Vec<u8>> = Vec::new();
        for cx in 0..clay_cols { clay_variants_raw.push(cell_rgba(cx, clay_row)); }
        let def0 = grass_variants_raw.get(0).cloned().unwrap_or_else(|| cell_rgba(0,0));
        let def1 = grass_variants_raw.get(1).cloned().unwrap_or_else(|| def0.clone());
        let def2 = water_full.clone();
        game_state.atlas.base_loaded = true;
        game_state.atlas.base_w = cell_w as i32;
        game_state.atlas.base_h = cell_h as i32;
        game_state.atlas.base_grass = def0;
        let forest_tile = if rows > 3 && cols > 7 { cell_rgba(7, 3) } else { def1.clone() };
        game_state.atlas.base_forest = forest_tile;
        game_state.atlas.base_water = def2;
        let dep_row = 5u32.min(rows-1);
        let dep_cx = 6u32.min(cols-1);
        let dep_tile = cell_rgba(dep_cx, dep_row);
        game_state.atlas.base_clay = dep_tile.clone();
        game_state.atlas.base_stone = dep_tile.clone();
        game_state.atlas.base_iron = dep_tile.clone();
        game_state.atlas.grass_variants = grass_variants_raw;
        game_state.atlas.clay_variants = clay_variants_raw;
        game_state.atlas.water_edges = water_edges_raw;
    } else if let Ok(img) = image::open("assets/tiles.png") {
        let img = img.to_rgba8();
        let (iw, ih) = img.dimensions();
        let tile_w = (iw / 6) as i32;
        let tile_h = ih as i32;
        let slice_rgba = |index: u32| -> Vec<u8> {
            let x0 = (index * tile_w as u32) as usize;
            let mut out = vec![0u8; (tile_w * tile_h * 4) as usize];
            for y in 0..tile_h as usize {
                let src = ((y as u32) * iw as u32 + x0 as u32) as usize * 4;
                let dst = y * tile_w as usize * 4;
                out[dst..dst + tile_w as usize * 4].copy_from_slice(&img.as_raw()[src..src + tile_w as usize * 4]);
            }
            out
        };
        game_state.atlas.base_loaded = true;
        game_state.atlas.base_w = tile_w;
        game_state.atlas.base_h = tile_h;
        game_state.atlas.base_grass = slice_rgba(0);
        game_state.atlas.base_forest = slice_rgba(1);
        game_state.atlas.base_water = slice_rgba(2);
        game_state.atlas.base_clay = slice_rgba(3);
        game_state.atlas.base_stone = slice_rgba(4);
        game_state.atlas.base_iron = slice_rgba(5);
    }
    if let Ok(img) = image::open("assets/buildings.png") {
        let img = img.to_rgba8();
        let (iw, ih) = img.dimensions();
        let base_w = if game_state.atlas.base_loaded { game_state.atlas.base_w } else { 64 } as u32;
        let cols = (iw / base_w).max(1);
        let mut sprites = Vec::new();
        for i in 0..cols {
            let x0 = (i * base_w) as usize;
            let mut out = vec![0u8; base_w as usize * ih as usize * 4];
            for y in 0..ih as usize {
                let src = (y * iw as usize + x0) * 4;
                let dst = y * base_w as usize * 4;
                out[dst..dst + base_w as usize * 4].copy_from_slice(&img.as_raw()[src..src + base_w as usize * 4]);
            }
            sprites.push(out);
        }
        println!("Загружено {} спрайтов зданий из buildings.png", sprites.len());
        game_state.building_atlas = Some(BuildingAtlas { w: base_w as i32, h: ih as i32 });
    }
    if let Ok(img) = image::open("assets/trees.png") {
        let img = img.to_rgba8();
        let (iw, ih) = img.dimensions();
        let base_w = if game_state.atlas.base_loaded { game_state.atlas.base_w } else { 64 } as u32;
        let cols = (iw / base_w).max(1);
        let mut sprites = Vec::new();
        for i in 0..cols {
            let x0 = (i * base_w) as usize;
            let mut out = vec![0u8; base_w as usize * ih as usize * 4];
            for y in 0..ih as usize {
                let src = (y * iw as usize + x0) * 4; let dst = y * base_w as usize * 4;
                out[dst..dst + base_w as usize * 4].copy_from_slice(&img.as_raw()[src..src + base_w as usize * 4]);
            }
            sprites.push(out);
        }
        game_state.tree_atlas = Some(atlas::TreeAtlas { w: base_w as i32, h: ih as i32 });
    }
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
                        let depot_total_wood: i32 = game_state.warehouses.iter().map(|w| w.wood).sum();
                        let total_visible_wood = game_state.resources.wood + depot_total_wood;
                        let visible = Resources {
                            wood: total_visible_wood,
                            stone: game_state.resources.stone + game_state.warehouses.iter().map(|w| w.stone).sum::<i32>(),
                            clay: game_state.resources.clay + game_state.warehouses.iter().map(|w| w.clay).sum::<i32>(),
                            bricks: game_state.resources.bricks + game_state.warehouses.iter().map(|w| w.bricks).sum::<i32>(),
                            wheat: game_state.resources.wheat + game_state.warehouses.iter().map(|w| w.wheat).sum::<i32>(),
                            flour: game_state.resources.flour + game_state.warehouses.iter().map(|w| w.flour).sum::<i32>(),
                            bread: game_state.resources.bread + game_state.warehouses.iter().map(|w| w.bread).sum::<i32>(),
                            fish: game_state.resources.fish + game_state.warehouses.iter().map(|w| w.fish).sum::<i32>(),
                            gold: game_state.resources.gold + game_state.warehouses.iter().map(|w| w.gold).sum::<i32>(),
                            iron_ore: game_state.resources.iron_ore + game_state.warehouses.iter().map(|w| w.iron_ore).sum::<i32>(),
                            iron_ingots: game_state.resources.iron_ingots + game_state.warehouses.iter().map(|w| w.iron_ingots).sum::<i32>(),
                        };
                        let mut idle=0; let mut working=0; let mut sleeping=0; let mut hauling=0; let mut fetching=0;
                        for c in &game_state.citizens {
                            use types::CitizenState::*;
                            match c.state {
                                Idle => idle+=1,
                                Working => working+=1,
                                Sleeping => sleeping+=1,
                                GoingToDeposit => hauling+=1,
                                GoingToFetch => fetching+=1,
                                GoingToWork | GoingHome => idle+=1,
                            }
                        }
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
