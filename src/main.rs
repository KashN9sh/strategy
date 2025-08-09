use anyhow::Result;
use glam::{IVec2, Vec2};
mod types; use types::{TileKind, BuildingKind, Building, Resources};
mod world; use world::World;
mod atlas; use atlas::{TileAtlas, BuildingAtlas};
mod render { pub mod tiles; }
mod ui;
mod input;
use pixels::{Pixels, SurfaceTexture};
use std::time::Instant;
use noise::NoiseFn;
use rand::{rngs::StdRng, Rng, SeedableRng};
 
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
// use image::GenericImageView; // не нужен
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

// размеры базового тайла перенесены в atlas::TILE_W/H
// Размер тайла в пикселях задаётся через атлас (half_w/half_h)
// Размер чанка в тайлах
// use world::{World, CHUNK_W, CHUNK_H};

// Перенесено в crate::types


// методы World вынесены в модуль world

// генерация чанков вынесена в модуль world

// --------------- Config / Save ---------------

type Config = input::Config;

type InputConfig = input::InputConfig;

// code_from_str перенесён в модуль input

fn load_or_create_config(path: &str) -> Result<(Config, InputConfig)> {
    if Path::new(path).exists() {
        let data = fs::read_to_string(path)?;
        #[derive(Deserialize)]
        struct FileCfg { config: Config, input: InputConfig }
        let parsed: FileCfg = toml::from_str(&data)?;
        Ok((parsed.config, parsed.input))
    } else {
        let config = Config { base_step_ms: 33.0, ui_scale_base: 1.6 };
        let input = InputConfig {
            move_up: "W".into(),
            move_down: "S".into(),
            move_left: "A".into(),
            move_right: "D".into(),
            zoom_in: "E".into(),
            zoom_out: "Q".into(),
            toggle_pause: "SPACE".into(),
            speed_0_5x: "DIGIT1".into(),
            speed_1x: "DIGIT2".into(),
            speed_2x: "DIGIT3".into(),
            speed_3x: "DIGIT4".into(),
            build_lumberjack: "Z".into(),
            build_house: "X".into(),
            toggle_road_mode: "R".into(),
            reset_new_seed: "T".into(),
            reset_same_seed: "N".into(),
            save_game: "F5".into(),
            load_game: "F9".into(),
        };
        #[derive(Serialize)]
        struct FileCfg<'a> { config: &'a Config, input: &'a InputConfig }
        let toml_text = toml::to_string_pretty(&FileCfg { config: &config, input: &input })?;
        fs::write(path, toml_text)?;
        Ok((config, input))
    }
}

type ResolvedInput = input::ResolvedInput;

// ResolvedInput::from реализован в модуле input

#[derive(Serialize, Deserialize)]
struct SaveData {
    seed: u64,
    resources: Resources,
    buildings: Vec<SaveBuilding>,
    cam_x: f32,
    cam_y: f32,
    zoom: f32,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct SaveBuilding { kind: BuildingKind, x: i32, y: i32, timer_ms: i32 }

impl SaveData {
    fn from_runtime(seed: u64, res: &Resources, buildings: &Vec<Building>, cam_px: Vec2, zoom: f32) -> Self {
        let buildings = buildings
            .iter()
            .map(|b| SaveBuilding { kind: b.kind, x: b.pos.x, y: b.pos.y, timer_ms: b.timer_ms })
            .collect();
        SaveData { seed, resources: *res, buildings, cam_x: cam_px.x, cam_y: cam_px.y, zoom }
    }

    fn to_buildings(&self) -> Vec<Building> {
        self.buildings
            .iter()
            .map(|s| Building { kind: s.kind, pos: IVec2::new(s.x, s.y), timer_ms: s.timer_ms })
            .collect()
    }
}

fn save_game(data: &SaveData) -> Result<()> {
    let txt = serde_json::to_string_pretty(data)?;
    fs::write("save.json", txt)?;
    Ok(())
}

fn load_game() -> Result<SaveData> {
    let txt = fs::read_to_string("save.json")?;
    let data: SaveData = serde_json::from_str(&txt)?;
    Ok(data)
}

fn main() -> Result<()> {
    run()
}

fn run() -> Result<()> {
    use std::rc::Rc;
    let event_loop = EventLoop::new()?;
    let window = Rc::new(WindowBuilder::new()
        .with_title("Strategy Isometric Prototype")
        .with_inner_size(LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)?);

    let size = window.inner_size();
    let surface_texture = SurfaceTexture::new(size.width, size.height, &*window);
    let mut pixels = Pixels::new(size.width, size.height, surface_texture)?;

    // Конфиг
    let (config, input) = load_or_create_config("config.toml")?;
    let input = ResolvedInput::from(&input);

    // Камера в пикселях мира (изометрических)
    let mut cam_px = Vec2::new(0.0, 0.0);
    let mut zoom: f32 = 2.0; // влияет на размеры тайла (через атлас)
    let mut last_frame = Instant::now();
    let mut accumulator_ms: f32 = 0.0;
    let mut paused = false;
    let mut speed_mult: f32 = 1.0; // 0.5, 1, 2, 3

    // Процедурная генерация: бесконечный мир чанков
    let mut rng = StdRng::seed_from_u64(42);
    let mut seed: u64 = rng.random();
    let mut world = World::new(seed);

    // Состояние игры
    let mut hovered_tile: Option<IVec2> = None;
    let mut selected_building: BuildingKind = BuildingKind::Lumberjack;
    let mut buildings: Vec<Building> = Vec::new();
    let mut resources = Resources { wood: 20, gold: 100 };
    let mut atlas = TileAtlas::new();
    let mut road_mode = false;
    let mut building_atlas: Option<BuildingAtlas> = None;
    // Попытаемся загрузить атлас из assets/tiles.png (ожидаем 3 тайла в строку: grass, forest, water)
    if let Ok(img) = image::open("assets/tiles.png") {
        let img = img.to_rgba8();
        let (iw, ih) = img.dimensions();
        // делим по 3 спрайта по ширине
        let tile_w = (iw / 3) as i32;
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
        atlas.base_loaded = true;
        atlas.base_w = tile_w;
        atlas.base_h = tile_h;
        atlas.base_grass = slice_rgba(0);
        atlas.base_forest = slice_rgba(1);
        atlas.base_water = slice_rgba(2);
    }
    // buildings.png: N спрайтов по горизонтали, ширина = base_w (или 64), высота любая
    if let Ok(img) = image::open("assets/buildings.png") {
        let img = img.to_rgba8();
        let (iw, ih) = img.dimensions();
        let base_w = if atlas.base_loaded { atlas.base_w } else { 64 } as u32;
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
        building_atlas = Some(BuildingAtlas { sprites, w: base_w as i32, h: ih as i32 });
    }
    let mut water_anim_time: f32 = 0.0;
    let mut show_grid = false;
    let mut show_forest_overlay = false;
    let mut show_ui = true;
    let mut cursor_xy = IVec2::new(0, 0);
    let mut fps_ema: f32 = 60.0;
    let mut show_ui = true;

    let mut width_i32 = size.width as i32;
    let mut height_i32 = size.height as i32;

    let window = window.clone();
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed {
                        let key = event.physical_key;
                        if key == PhysicalKey::Code(KeyCode::Escape) { elwt.exit(); }

                        if key == PhysicalKey::Code(input.move_up) { cam_px.y -= 80.0; }
                        if key == PhysicalKey::Code(input.move_down) { cam_px.y += 80.0; }
                        if key == PhysicalKey::Code(input.move_left) { cam_px.x -= 80.0; }
                        if key == PhysicalKey::Code(input.move_right) { cam_px.x += 80.0; }
                        if key == PhysicalKey::Code(input.zoom_out) { zoom = (zoom * 0.9).max(0.5); }
                        if key == PhysicalKey::Code(input.zoom_in) { zoom = (zoom * 1.1).min(8.0); }
                        if key == PhysicalKey::Code(input.toggle_pause) { paused = !paused; }
                        if key == PhysicalKey::Code(input.speed_0_5x) { speed_mult = 0.5; }
                        if key == PhysicalKey::Code(input.speed_1x) { speed_mult = 1.0; }
                        if key == PhysicalKey::Code(input.speed_2x) { speed_mult = 2.0; }
                        if key == PhysicalKey::Code(input.speed_3x) { speed_mult = 3.0; }
                        if key == PhysicalKey::Code(KeyCode::KeyG) { show_grid = !show_grid; }
                        if key == PhysicalKey::Code(KeyCode::KeyH) { show_forest_overlay = !show_forest_overlay; }
                        if key == PhysicalKey::Code(KeyCode::KeyU) { show_ui = !show_ui; }
                        if key == PhysicalKey::Code(input.toggle_road_mode) { road_mode = !road_mode; }
                        if key == PhysicalKey::Code(input.build_lumberjack) { selected_building = BuildingKind::Lumberjack; }
                        if key == PhysicalKey::Code(input.build_house) { selected_building = BuildingKind::House; }
                        if key == PhysicalKey::Code(input.reset_new_seed) { seed = rng.random(); world.reset_noise(seed); buildings.clear(); resources = Resources { wood: 20, gold: 100 }; }
                        if key == PhysicalKey::Code(input.reset_same_seed) { world.reset_noise(seed); buildings.clear(); resources = Resources { wood: 20, gold: 100 }; }
                        if key == PhysicalKey::Code(input.save_game) { let _ = save_game(&SaveData::from_runtime(seed, &resources, &buildings, cam_px, zoom)); }
                        if key == PhysicalKey::Code(input.load_game) {
                            if let Ok(save) = load_game() {
                                seed = save.seed;
                                world.reset_noise(seed);
                                buildings = save.to_buildings();
                                resources = save.resources;
                                cam_px = Vec2::new(save.cam_x, save.cam_y);
                                zoom = save.zoom;
                                // восстановим отметку occupied
                                world.occupied.clear();
                                for b in &buildings { world.occupy(b.pos); }
                            }
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let mx = position.x as i32;
                    let my = position.y as i32;
                    cursor_xy = IVec2::new(mx, my);
                    let cam_snap = Vec2::new(cam_px.x.round(), cam_px.y.round());
                    hovered_tile = screen_to_tile_px(mx, my, width_i32, height_i32, cam_snap, atlas.half_w, atlas.half_h);
                }
                WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                    // ЛКМ — попытка построить
                    if button == winit::event::MouseButton::Left {
                        if show_ui {
                            let ui_s = ui::ui_scale(height_i32, config.ui_scale_base);
                            let bar_h = ui::ui_bar_height(height_i32, ui_s);
                            if cursor_xy.y >= 0 && cursor_xy.y < bar_h {
                                let pad = 8 * ui_s; let icon_size = 10 * ui_s; let by = pad + icon_size + 8 * ui_s; let btn_w = 90 * ui_s; let btn_h = 18 * ui_s;
                                if ui::point_in_rect(cursor_xy.x, cursor_xy.y, pad, by, btn_w, btn_h) { selected_building = BuildingKind::Lumberjack; return; }
                                if ui::point_in_rect(cursor_xy.x, cursor_xy.y, pad + btn_w + 6 * ui_s, by, btn_w, btn_h) { selected_building = BuildingKind::House; return; }
                            }
                        }
                        if let Some(tp) = hovered_tile {
                            if road_mode {
                                // Переключение дороги по клику
                                let on = !world.is_road(tp);
                                world.set_road(tp, on);
                                return;
                            }
                            // кликаем по тайлу под курсором, но убедимся, что используем те же snapped-пиксели камеры,
                            // чтобы не было рассинхрона между рендером и хитом
                            let tile_kind = world.get_tile(tp.x, tp.y);
                            if !world.is_occupied(tp) && tile_kind != TileKind::Water {
                                let cost = building_cost(selected_building);
                                if resources.wood >= cost.wood && resources.gold >= cost.gold {
                                    resources.wood -= cost.wood;
                                    resources.gold -= cost.gold;
                                    world.occupy(tp);
                                    buildings.push(Building { kind: selected_building, pos: tp, timer_ms: 0 });
                                    println!("Построено {:?} на {:?}. Ресурсы: wood={}, gold={}", selected_building, tp, resources.wood, resources.gold);
                                }
                            }
                        }
                    }
                }
                WindowEvent::Resized(new_size) => {
                    width_i32 = new_size.width as i32;
                    height_i32 = new_size.height as i32;
                    pixels.resize_surface(new_size.width, new_size.height).ok();
                    pixels.resize_buffer(new_size.width, new_size.height).ok();
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let factor = match delta {
                        MouseScrollDelta::LineDelta(_, y) => if y > 0.0 { 1.1 } else { 0.9 },
                        MouseScrollDelta::PixelDelta(p) => if p.y > 0.0 { 1.1 } else { 0.9 },
                    };
                    zoom = (zoom * factor).clamp(0.5, 8.0);
                }
                WindowEvent::RedrawRequested => {
                    let frame = pixels.frame_mut();
                    clear(frame, [12, 18, 24, 255]);

                    // Центр экрана
                    let screen_center = IVec2::new(width_i32 / 2, height_i32 / 2);

                    // Обновим атлас для текущего зума
                    atlas.ensure_zoom(zoom);

                    // Границы видимых тайлов через инверсию проекции
                    let (min_tx, min_ty, max_tx, max_ty) = visible_tile_bounds_px(width_i32, height_i32, cam_px, atlas.half_w, atlas.half_h);
                    // Закажем генерацию колец чанков
                    world.schedule_ring(min_tx, min_ty, max_tx, max_ty);
                    // Интегрируем готовые чанки (non-blocking)
                    world.integrate_ready_chunks();

                    // Рисуем тайлы быстрым блитом
                    let water_frame = ((water_anim_time / 120.0) as usize) % atlas.water_frames.len().max(1);
                    let cam_snap = Vec2::new(cam_px.x.round(), cam_px.y.round());
                    for my in min_ty..=max_ty {
                        for mx in min_tx..=max_tx {
                            let kind = world.get_tile(mx, my);
                            let world_x = (mx - my) * atlas.half_w - cam_snap.x as i32;
                            let world_y = (mx + my) * atlas.half_h - cam_snap.y as i32;
                            let screen_pos = screen_center + IVec2::new(world_x, world_y);
                            atlas.blit(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, kind, water_frame);

                            // сетка
                            if show_grid { render::tiles::draw_iso_outline(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [20, 20, 20, 255]); }

                            // дороги (простая заливка поверх тайла)
                            if world.is_road(IVec2::new(mx, my)) { render::tiles::draw_iso_tile_tinted(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [120, 110, 90, 200]); }

                            // оверлей плотности леса (простая функция от шума)
                            if show_forest_overlay { let n = world.fbm.get([mx as f64, my as f64]) as f32; let v = ((n + 1.0) * 0.5 * 255.0) as u8; render::tiles::draw_iso_outline(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [v, 50, 50, 255]); }
                        }
                    }

                    // Подсветка ховера
                    if let Some(tp) = hovered_tile {
                        let world_x = (tp.x - tp.y) * atlas.half_w - cam_snap.x as i32;
                        let world_y = (tp.x + tp.y) * atlas.half_h - cam_snap.y as i32;
                        let screen_pos = screen_center + IVec2::new(world_x, world_y);
                        render::tiles::draw_iso_outline(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [240, 230, 80, 255]);
                    }

                    // Отрисуем здания по глубине
                    buildings.sort_by_key(|b| b.pos.x + b.pos.y);
                    for b in buildings.iter() {
                        let mx = b.pos.x;
                        let my = b.pos.y;
                        if mx < min_tx || my < min_ty || mx > max_tx || my > max_ty { continue; }
                        let world_x = (mx - my) * atlas.half_w - cam_snap.x as i32;
                        let world_y = (mx + my) * atlas.half_h - cam_snap.y as i32;
                        let screen_pos = screen_center + IVec2::new(world_x, world_y);
                        if let Some(ba) = &building_atlas {
                            if let Some(idx) = building_sprite_index(b.kind) {
                                if idx < ba.sprites.len() {
                                    // Масштаб спрайта по текущей ширине тайла
                                    let tile_w_px = atlas.half_w * 2 + 1;
                                    let scale = tile_w_px as f32 / ba.w as f32;
                                    let draw_w = (ba.w as f32 * scale).round() as i32;
                                    let draw_h = (ba.h as f32 * scale).round() as i32;
                                    let top_left_x = screen_pos.x - draw_w / 2;
                                    // Привязываем нижний центр спрайта к нижней вершине ромба тайла
                                    let top_left_y = screen_pos.y + atlas.half_h/2 - draw_h;
                                    render::tiles::blit_sprite_alpha_scaled(frame, width_i32, height_i32, top_left_x, top_left_y, &ba.sprites[idx], ba.w, ba.h, draw_w, draw_h);
                                    continue;
                                }
                            }
                        }
                        let color = match b.kind { BuildingKind::Lumberjack => [140, 90, 40, 255], BuildingKind::House => [180, 180, 180, 255] };
                        render::tiles::draw_building(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, color);
                    }

                    // UI наложение
                    if show_ui { ui::draw_ui(frame, width_i32, height_i32, &resources, selected_building, fps_ema, speed_mult, paused, config.ui_scale_base); }

                    if let Err(err) = pixels.render() {
                        eprintln!("pixels.render() failed: {err}");
                        elwt.exit();
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                // фиксированный тик с ускорением
                let now = Instant::now();
                let frame_ms = (now - last_frame).as_secs_f32() * 1000.0;
                last_frame = now;
                // ограничим, чтобы не накапливалось слишком много
                let frame_ms = frame_ms.min(250.0);
                accumulator_ms += frame_ms;
                water_anim_time += frame_ms;
                if frame_ms > 0.0 { fps_ema = fps_ema * 0.9 + (1000.0 / frame_ms) * 0.1; }

                let base_step_ms = config.base_step_ms;
                let step_ms = (base_step_ms / speed_mult.max(0.0001)).max(1.0);
                let mut did_step = false;
                if !paused {
                    while accumulator_ms >= step_ms {
                        simulate(&mut buildings, &mut world, &mut resources, step_ms as i32);
                        accumulator_ms -= step_ms;
                        did_step = true;
                        // ограничим число шагов за кадр
                        if accumulator_ms > 10.0 * step_ms { accumulator_ms = 0.0; break; }
                    }
                }
                if did_step {
                    window.request_redraw();
                } else {
                    // всё равно перерисуем с периодичностью
                    window.request_redraw();
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}

fn clear(frame: &mut [u8], rgba: [u8; 4]) {
    for px in frame.chunks_exact_mut(4) {
        px.copy_from_slice(&rgba);
    }
}

// удалено: draw_iso_tile (перенесено в модуль render)

// удалено: draw_iso_tile_tinted (перенесено в модуль render)

// удалено: draw_iso_outline (перенесено в модуль render)

// удалено: draw_line (перенесено в модуль render)

// удалено: draw_building (перенесено в модуль render)

fn building_sprite_index(kind: BuildingKind) -> Option<usize> {
    match kind {
        BuildingKind::Lumberjack => Some(0),
        BuildingKind::House => Some(1),
    }
}

// удалено: blit_sprite_alpha (перенесено в модуль render)

// удалено: blit_sprite_alpha_scaled (перенесено в модуль render)

// UI helpers переехали в модуль ui

// draw_ui перенесён в модуль ui

// fill_rect перенесён в модуль ui

// draw_button перенесён в модуль ui

// draw_text_mini перенесён в модуль ui

// draw_glyph_3x5 перенесён в модуль ui

// draw_number перенесён в модуль ui

// point_in_rect перенесён в модуль ui

fn screen_to_tile_px(mx: i32, my: i32, sw: i32, sh: i32, cam_px: Vec2, half_w: i32, half_h: i32) -> Option<IVec2> {
    // экран -> мир (в пикселях изометрии)
    let dx = (mx - sw / 2) as f32 + cam_px.x;
    let dy = (my - sh / 2) as f32 + cam_px.y;
    let a = half_w as f32;
    let b = half_h as f32;
    // обратное к: screen_x = (x - y)*a, screen_y = (x + y)*b
    let tx = 0.5 * (dy / b + dx / a);
    let ty = 0.5 * (dy / b - dx / a);
    let ix = tx.floor() as i32;
    let iy = ty.floor() as i32;
    Some(IVec2::new(ix, iy))
}

fn visible_tile_bounds_px(sw: i32, sh: i32, cam_px: Vec2, half_w: i32, half_h: i32) -> (i32, i32, i32, i32) {
    // по четырём углам экрана
    let corners = [
        (0, 0),
        (sw, 0),
        (0, sh),
        (sw, sh),
    ];
    let mut min_tx = i32::MAX;
    let mut min_ty = i32::MAX;
    let mut max_tx = i32::MIN;
    let mut max_ty = i32::MIN;
    for (x, y) in corners {
            if let Some(tp) = screen_to_tile_px(x, y, sw, sh, cam_px, half_w, half_h) {
            min_tx = min_tx.min(tp.x);
            min_ty = min_ty.min(tp.y);
            max_tx = max_tx.max(tp.x);
            max_ty = max_ty.max(tp.y);
        }
    }
    // запас; не ограничиваем картой, чтобы рисовать воду вне карты
    if min_tx == i32::MAX { return (-64, -64, 64, 64); }
    (min_tx - 4, min_ty - 4, max_tx + 4, max_ty + 4)
}

fn building_cost(kind: BuildingKind) -> Resources {
    match kind {
        BuildingKind::Lumberjack => Resources { wood: 5, gold: 10 },
        BuildingKind::House => Resources { wood: 10, gold: 15 },
    }
}

fn simulate(buildings: &mut Vec<Building>, world: &mut World, resources: &mut Resources, dt_ms: i32) {
    for b in buildings.iter_mut() {
        b.timer_ms += dt_ms;
        match b.kind {
            BuildingKind::Lumberjack => {
                // каждые 2с +1 дерево, если рядом есть лес
                if b.timer_ms >= 2000 {
                    b.timer_ms = 0;
                    if has_adjacent_forest(b.pos, world) {
                        resources.wood += 1;
                    }
                }
            }
            BuildingKind::House => {
                // каждые 5с -1 дерево, +1 золото
                if b.timer_ms >= 5000 {
                    b.timer_ms = 0;
                    if resources.wood > 0 { resources.wood -= 1; resources.gold += 1; }
                }
            }
        }
    }
}

fn has_adjacent_forest(p: IVec2, world: &mut World) -> bool {
    const NB: [(i32, i32); 4] = [(1,0),(-1,0),(0,1),(0,-1)];
    for (dx, dy) in NB {
        let x = p.x + dx;
        let y = p.y + dy;
        if world.get_tile(x, y) == TileKind::Forest { return true; }
    }
    false
}

