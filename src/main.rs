use anyhow::Result;
use glam::{IVec2, Vec2};
mod types; use types::{TileKind, BuildingKind, Building, Resources, Citizen, Job, JobKind, LogItem, WarehouseStore, CitizenState, ResourceKind};
mod world; use world::World;
mod atlas; use atlas::{TileAtlas, BuildingAtlas};
mod render { pub mod tiles; pub mod utils; pub mod map; }
mod ui;
mod input;
mod config;
mod save;
mod path;
mod jobs;
mod controls;
mod ui_interaction;
mod game;
mod palette;
use pixels::{Pixels, SurfaceTexture};
use std::time::Instant;
use rand::{rngs::StdRng, Rng, SeedableRng};
 
// use std::fs; // перенесено в config
// use std::path::Path; // перенесено в config
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

// типы конфига используются через модуль input/config

// code_from_str перенесён в модуль input

// переносено: загрузка/создание конфига в модуль config

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
    trees: Vec<SaveTree>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct SaveBuilding { kind: BuildingKind, x: i32, y: i32, timer_ms: i32, #[serde(default)] workers_target: i32 }

#[derive(Serialize, Deserialize, Clone, Copy)]
struct SaveTree { x: i32, y: i32, stage: u8, age_ms: i32 }

// перенос в save.rs

// перенос в save.rs

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
    let (config, input) = config::load_or_create("config.toml")?;
    let input = ResolvedInput::from(&input);

    // Камера в пикселях мира (изометрических)
    let mut cam_px = Vec2::new(0.0, 0.0);
    let mut zoom: f32 = 2.0; // влияет на размеры тайла (через атлас)
    let mut last_frame = Instant::now();
    let mut accumulator_ms: f32 = 0.0;
    let mut paused = false;
    // Экономика: налог (монет/жителя/день)
    let mut tax_rate: f32 = 2.0;
    let mut speed_mult: f32 = 1.0; // 0.5, 1, 2, 3

    // Процедурная генерация: бесконечный мир чанков
    let mut rng = StdRng::seed_from_u64(42);
    let mut seed: u64 = rng.random();
    let mut world = World::new(seed);

    // Состояние игры
    let mut hovered_tile: Option<IVec2> = None;
    let mut selected_building: BuildingKind = BuildingKind::Lumberjack;
    let mut ui_category: ui::UICategory = ui::UICategory::Forestry;
    let mut ui_tab: ui::UITab = ui::UITab::Build;
    let mut food_policy: crate::types::FoodPolicy = crate::types::FoodPolicy::Balanced;
    let mut buildings: Vec<Building> = Vec::new();
    let mut buildings_dirty: bool = true;
    let mut citizens: Vec<Citizen> = Vec::new();
    let mut jobs: Vec<Job> = Vec::new();
    let mut next_job_id: u64 = 1;
    let mut logs_on_ground: Vec<LogItem> = Vec::new();
    // стартовый пакет: немного дерева/еды/денег, и склад с запасом дерева
    let mut resources = Resources { wood: 60, gold: 200, bread: 10, fish: 10, ..Default::default() };
    let mut warehouses: Vec<WarehouseStore> = Vec::new();
    let mut population: i32 = 0;
    let mut atlas = TileAtlas::new();
    let mut road_atlas = atlas::RoadAtlas::new();
    let mut road_mode = false;
    let mut path_debug_mode = false;
    let mut path_sel_a: Option<IVec2> = None;
    let mut path_sel_b: Option<IVec2> = None;
    let mut last_path: Option<Vec<IVec2>> = None;
    let mut building_atlas: Option<BuildingAtlas> = None;
    let mut tree_atlas: Option<atlas::TreeAtlas> = None;
    // Попытаемся загрузить спрайтшит 32x32: assets/spritesheet.png
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
        // Жёсткая раскладка по описанию:
        // row 2 (индекс 2) — вариации травы (grass)
        // последний ряд (rows-1): вода — первая ячейка чистая вода, остальные — кромки
        let grass_row = 2u32.min(rows-1);
        // соберём все варианты травы из строки
        let mut grass_variants_raw: Vec<Vec<u8>> = Vec::new();
        let grass_cols = cols.min(3); // используем только первые три варианта травы
        for cx in 0..grass_cols { grass_variants_raw.push(cell_rgba(cx, grass_row)); }
        let water_row = rows-1;
        let water_full = cell_rgba(0, water_row);
        let mut water_edges_raw: Vec<Vec<u8>> = Vec::new();
        // тайлы 2..8 (1-based) → cx=1..=7
        for cx in 1..=7 { if cx < cols { water_edges_raw.push(cell_rgba(cx, water_row)); } }
        // Варианты глины: 2-я строка, первые 3 спрайта
        let clay_row = 1u32.min(rows-1);
        let clay_cols = cols.min(3);
        let mut clay_variants_raw: Vec<Vec<u8>> = Vec::new();
        for cx in 0..clay_cols { clay_variants_raw.push(cell_rgba(cx, clay_row)); }
        // База для ромбической маски (на случай фоллбека и оверлеев deposits)
        let def0 = grass_variants_raw.get(0).cloned().unwrap_or_else(|| cell_rgba(0,0));
        let def1 = grass_variants_raw.get(1).cloned().unwrap_or_else(|| def0.clone());
        let def2 = water_full.clone();
        atlas.base_loaded = true;
        atlas.base_w = cell_w as i32;
        atlas.base_h = cell_h as i32;
        atlas.base_grass = def0;
        // Лесная трава: 4-я линия, 8-й спрайт (1-based) → cy=3, cx=7 (0-based)
        let forest_tile = if rows > 3 && cols > 7 { cell_rgba(7, 3) } else { def1.clone() };
        atlas.base_forest = forest_tile;
        atlas.base_water = def2;
        // депозит-маркер: 6-я строка, 7-й спрайт (1-based) → cy=5, cx=6 (0-based); с защитой границ
        let dep_row = 5u32.min(rows-1);
        let dep_cx = 6u32.min(cols-1);
        let dep_tile = cell_rgba(dep_cx, dep_row);
        atlas.base_clay = dep_tile.clone();
        atlas.base_stone = dep_tile.clone();
        atlas.base_iron = dep_tile.clone();
        // сохраним вариации травы — будем использовать при рендере PNG-тайлов
        atlas.grass_variants = grass_variants_raw;
        atlas.clay_variants = clay_variants_raw;
        atlas.water_edges = water_edges_raw;
        // заглушки для deposits из старого tiles.png отсутствуют — оставим пустыми, оверлеи не обязательны
    } else if let Ok(img) = image::open("assets/tiles.png") {
        // Старый путь: 6 тайлов в строку: grass, forest, water, clay, stone, iron
        let img = img.to_rgba8();
        let (iw, ih) = img.dimensions();
        // делим по 6 спрайтов по ширине
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
        atlas.base_loaded = true;
        atlas.base_w = tile_w;
        atlas.base_h = tile_h;
        atlas.base_grass = slice_rgba(0);
        atlas.base_forest = slice_rgba(1);
        atlas.base_water = slice_rgba(2);
        atlas.base_clay = slice_rgba(3);
        atlas.base_stone = slice_rgba(4);
        atlas.base_iron = slice_rgba(5);
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
    // trees.png: N спрайтов по горизонтали (стадии роста 0..N-1), ширина = base_w (или 64), высота любая
    if let Ok(img) = image::open("assets/trees.png") {
        let img = img.to_rgba8();
        let (iw, ih) = img.dimensions();
        let base_w = if atlas.base_loaded { atlas.base_w } else { 64 } as u32;
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
        tree_atlas = Some(atlas::TreeAtlas { sprites, w: base_w as i32, h: ih as i32 });
    }
    let mut water_anim_time: f32 = 0.0;
    // Экономика: вчерашние итоги
    let mut last_tax_income: i32 = 0;
    let mut last_upkeep_cost: i32 = 0;
    let mut show_grid = false;
    let mut show_forest_overlay = false;
    let mut show_tree_stage_overlay = false;
    let mut show_ui = true;
    let mut cursor_xy = IVec2::new(0, 0);
    let mut fps_ema: f32 = 60.0;
    // удалён дубликат переменной show_ui
    // выбранный житель для ручного назначения на работу (отключено)
    let _selected_citizen: Option<usize> = None; // больше не используется для назначения
    // активная панель здания (по клику)
    let mut active_building_panel: Option<IVec2> = None;

    let mut width_i32 = size.width as i32;
    let mut height_i32 = size.height as i32;
    // День/ночь
    const DAY_LENGTH_MS: f32 = 120_000.0;
    const START_HOUR: f32 = 8.0; // старт в 08:00
    let mut world_clock_ms: f32 = DAY_LENGTH_MS * (START_HOUR / 24.0);
    // Предыдущее состояние дня/ночи для детекции рассвета
    let mut prev_is_day_flag: bool = {
        let t0 = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
        let angle0 = t0 * std::f32::consts::TAU;
        let daylight0 = 0.5 - 0.5 * angle0.cos();
        daylight0 > 0.25
    };

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
                        if key == PhysicalKey::Code(input.tax_up) { tax_rate = (tax_rate + config.tax_step).min(config.tax_max); }
                        if key == PhysicalKey::Code(input.tax_down) { tax_rate = (tax_rate - config.tax_step).max(config.tax_min); }
                        controls::handle_key_press(key, &input, &mut rng, &mut world, &mut buildings, &mut buildings_dirty, &mut citizens, &mut population, &mut resources, &mut selected_building, &mut show_grid, &mut show_forest_overlay, &mut show_tree_stage_overlay, &mut show_ui, &mut road_mode, &mut path_debug_mode, &mut path_sel_a, &mut path_sel_b, &mut last_path, &mut speed_mult, &mut seed);
                        if key == PhysicalKey::Code(input.save_game) { let _ = save::save_game(&save::SaveData::from_runtime(seed, &resources, &buildings, cam_px, zoom, &world)); }
                        if key == PhysicalKey::Code(input.load_game) {
                            if let Ok(save) = save::load_game() {
                                seed = save.seed;
                                world.reset_noise(seed);
                                buildings = save.to_buildings(); buildings_dirty = true;
                                citizens.clear(); population = 0; // пока не сохраняем жителей
                                resources = save.resources;
                                cam_px = Vec2::new(save.cam_x, save.cam_y);
                                zoom = save.zoom;
                                // восстановим отметку occupied
                                world.occupied.clear();
                                for b in &buildings { world.occupy(b.pos); }
                                // восстановим деревья
                                world.trees.clear(); world.removed_trees.clear();
                                for t in &save.trees { world.trees.insert((t.x, t.y), world::Tree { stage: t.stage, age_ms: t.age_ms }); }
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
                    if button == winit::event::MouseButton::Left {
                        if show_ui {
                            if ui_interaction::handle_left_click(
                                cursor_xy, width_i32, height_i32, &config, &atlas, hovered_tile,
                                &mut ui_category, &mut ui_tab, &mut tax_rate, &mut food_policy,
                                &mut selected_building, &mut active_building_panel,
                                &mut world, &mut buildings, &mut buildings_dirty, &mut citizens, &mut population,
                                &mut warehouses, &mut resources, &mut road_mode, &mut path_debug_mode,
                                &mut path_sel_a, &mut path_sel_b, &mut last_path,
                            ) { return; }
                        }
                        // остальная часть обработки ЛКМ остаётся прежней (клика по миру вне UI)
                        if let Some(tp) = hovered_tile {
                            if let Some(bh) = buildings.iter().find(|bb| bb.pos == tp) {
                                active_building_panel = match active_building_panel { Some(cur) if cur == bh.pos => None, _ => Some(bh.pos) }; return;
                            }
                            if road_mode { let on = !world.is_road(tp); world.set_road(tp, on); return; }
                            if path_debug_mode {
                                match (path_sel_a, path_sel_b) { (None, _) => { path_sel_a = Some(tp); last_path=None; }, (Some(_), None) => { path_sel_b = Some(tp); }, (Some(_), Some(_)) => { path_sel_a = Some(tp); path_sel_b=None; last_path=None; } }
                                if let (Some(a), Some(b)) = (path_sel_a, path_sel_b) { last_path = crate::path::astar(&world, a, b, 20_000); }
                                return;
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
                    // Подготовим процедурный атлас дорог под текущий масштаб
                    road_atlas.ensure_zoom(atlas.half_w, atlas.half_h);
                    // Закажем генерацию колец чанков
                    world.schedule_ring(min_tx, min_ty, max_tx, max_ty);
                    // Интегрируем готовые чанки (non-blocking)
                    world.integrate_ready_chunks();

                    let cam_snap = Vec2::new(cam_px.x.round(), cam_px.y.round());
                    let water_frame = ((water_anim_time / 120.0) as usize) % atlas.water_frames.len().max(1);
                    render::map::draw_terrain_and_overlays(
                        frame, width_i32, height_i32, &atlas, &mut world,
                        min_tx, min_ty, max_tx, max_ty,
                        screen_center, cam_snap,
                        water_frame,
                        show_grid,
                        show_forest_overlay,
                        false, // деревья рисуем вместе со зданиями после сортировки
                        &tree_atlas,
                        &road_atlas,
                    );

                    // Поленья на земле
                    for li in &logs_on_ground {
                        if li.carried { continue; }
                        let mx = li.pos.x; let my = li.pos.y;
                        if mx < min_tx || my < min_ty || mx > max_tx || my > max_ty { continue; }
                        let screen_pos = render::map::world_to_screen(&atlas, screen_center, cam_snap, mx, my);
                        render::tiles::draw_log(frame, width_i32, height_i32, screen_pos.x, screen_pos.y - atlas.half_h/4, atlas.half_w, atlas.half_h);
                    }

                    // Подсветка ховера (заливка + контур) — пересчёт под актуальную камеру
                    if let Some(tp) = screen_to_tile_px(cursor_xy.x, cursor_xy.y, width_i32, height_i32, cam_snap, atlas.half_w, atlas.half_h) {
                        let screen_pos = render::map::world_to_screen(&atlas, screen_center, cam_snap, tp.x, tp.y);
                        let hover_off = ((atlas.half_h as f32) * 0.5).round() as i32; // ~30px при max zoom
                        // мягкая внутренняя подсветка
                        render::tiles::draw_iso_tile_tinted(
                            frame, width_i32, height_i32,
                            screen_pos.x, screen_pos.y + hover_off,
                            atlas.half_w, atlas.half_h,
                            [240, 230, 80, 70],
                        );
                        render::tiles::draw_iso_outline(frame, width_i32, height_i32, screen_pos.x, screen_pos.y + hover_off, atlas.half_w, atlas.half_h, [240, 230, 80, 255]);
                    }

                    // Отрисуем здания и деревья вместе, отсортировав по глубине
                    if buildings_dirty {
                        buildings.sort_by_key(|b| b.pos.x + b.pos.y);
                        buildings_dirty = false;
                    }
                    // Диагональный проход (x+y): обеспечивает правильный painter's order
                    render::map::draw_structures_diagonal_scan(
                        frame, width_i32, height_i32, &atlas,
                        &world,
                        &buildings, &building_atlas,
                        &tree_atlas,
                        screen_center, cam_snap,
                        min_tx, min_ty, max_tx, max_ty,
                    );

                    render::map::draw_citizens(frame, width_i32, height_i32, &atlas, &citizens, &buildings, screen_center, cam_snap);

                    // Оверлей день/ночь
                    let t = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
                    let angle = t * std::f32::consts::TAU;
                    let daylight = 0.5 - 0.5 * angle.cos();
                    let darkness = (1.0 - daylight).max(0.0);
                    let night_strength = (darkness.powf(1.4) * 180.0).min(200.0) as u8;
                    if night_strength > 0 { overlay_tint(frame, width_i32, height_i32, [18, 28, 60, night_strength]); }

                    // Отрисовка найденного пути в дебаг-режиме
                    if let (true, Some(path)) = (path_debug_mode, &last_path) {
                        render::map::draw_debug_path(frame, width_i32, height_i32, &atlas, path, screen_center, cam_snap);
                    }

                    // UI наложение
                    if show_ui {
                        let depot_total_wood: i32 = warehouses.iter().map(|w| w.wood).sum();
                        let total_visible_wood = resources.wood + depot_total_wood;
                        // Показ ресурсов как сумма "на руках" + в складах
                        let visible = Resources {
                            wood: total_visible_wood,
                            stone: resources.stone + warehouses.iter().map(|w| w.stone).sum::<i32>(),
                            clay: resources.clay + warehouses.iter().map(|w| w.clay).sum::<i32>(),
                            bricks: resources.bricks + warehouses.iter().map(|w| w.bricks).sum::<i32>(),
                            wheat: resources.wheat + warehouses.iter().map(|w| w.wheat).sum::<i32>(),
                            flour: resources.flour + warehouses.iter().map(|w| w.flour).sum::<i32>(),
                            bread: resources.bread + warehouses.iter().map(|w| w.bread).sum::<i32>(),
                            fish: resources.fish + warehouses.iter().map(|w| w.fish).sum::<i32>(),
                            gold: resources.gold + warehouses.iter().map(|w| w.gold).sum::<i32>(),
                            iron_ore: resources.iron_ore + warehouses.iter().map(|w| w.iron_ore).sum::<i32>(),
                            iron_ingots: resources.iron_ingots + warehouses.iter().map(|w| w.iron_ingots).sum::<i32>(),
                        };
                        // Статусы жителей для UI
                        let mut idle=0; let mut working=0; let mut sleeping=0; let mut hauling=0; let mut fetching=0;
                        for c in &citizens {
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
                        let day_progress = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
                        // среднее счастье
                        let avg_hap: f32 = if citizens.is_empty() { 50.0 } else { citizens.iter().map(|c| c.happiness as i32).sum::<i32>() as f32 / citizens.len() as f32 };
                        // Жилищная вместимость/занятость
                        let mut housing_cap = 0; let mut housing_used = 0;
                        for b in &buildings { if b.kind == BuildingKind::House { housing_cap += b.capacity; } }
                        for c in &citizens { if buildings.iter().any(|b| b.kind == BuildingKind::House && b.pos == c.home) { housing_used += 1; } }
                        let pop_show = citizens.len() as i32;
                        ui::draw_ui(
                            frame, width_i32, height_i32,
                            &visible, total_visible_wood, pop_show, selected_building,
                            fps_ema, speed_mult, paused, config.ui_scale_base, ui_category, day_progress,
                            idle, working, sleeping, hauling, fetching, cursor_xy.x, cursor_xy.y,
                            avg_hap, tax_rate, ui_tab, food_policy,
                            last_tax_income, last_upkeep_cost,
                            housing_used, housing_cap,
                        );
                        // Если выбрана панель здания — рисуем её
                        if let Some(p) = active_building_panel {
                            if let Some(b) = buildings.iter().find(|bb| bb.pos == p) {
                                // считаем фактических работников
                                let workers_current = citizens.iter().filter(|c| c.workplace == Some(b.pos)).count() as i32;
                                let (prod, cons): (&[u8], Option<&[u8]>) = match b.kind {
                                    BuildingKind::StoneQuarry => (b"+ Stone", None),
                                    BuildingKind::ClayPit => (b"+ Clay", None),
                                    BuildingKind::IronMine => (b"+ Iron Ore", None),
                                    BuildingKind::WheatField => (b"+ Wheat", None),
                                    BuildingKind::Mill => (b"+ Flour", Some(b"- Wheat")),
                                    BuildingKind::Kiln => (b"+ Bricks", Some(b"- Clay, - Wood")),
                                    BuildingKind::Bakery => (b"+ Bread", Some(b"- Flour, - Wood")),
                                    BuildingKind::Fishery => (b"+ Fish", None),
                                    BuildingKind::Smelter => (b"+ Iron Ingot", Some(b"- Iron Ore, - Wood")),
                                    BuildingKind::Lumberjack => (b"+ Wood (via jobs)", None),
                                    BuildingKind::House | BuildingKind::Warehouse | BuildingKind::Forester => (b"", None),
                                };
                                ui::draw_building_panel(frame, width_i32, height_i32, ui::ui_scale(height_i32, config.ui_scale_base), b.kind, workers_current, b.workers_target, prod, cons);
                            }
                        }
                    }

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
                        // Подтянем готовые чанки перед генерацией задач, чтобы деревья из новых чанков
                        // были видны логике (особенно если камера сдвинулась).
                        world.integrate_ready_chunks();
                        game::simulate(&mut buildings, &mut world, &mut resources, &mut warehouses, step_ms as i32);
                        world.grow_trees(step_ms as i32);
                        world_clock_ms = (world_clock_ms + step_ms) % DAY_LENGTH_MS;

                        // День/ночь
                        let t = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
                        let angle = t * std::f32::consts::TAU;
                        let daylight = 0.5 - 0.5 * angle.cos();
                        let is_day = daylight > 0.25; // простой порог
                        // На рассвете (переход ночь→день) — кормление и доход
                        if !prev_is_day_flag && is_day {
                            let (income_y, upkeep_y) = game::economy_new_day(&mut citizens, &mut resources, &mut warehouses, &buildings, tax_rate, &config, food_policy);
                            last_tax_income = income_y;
                            last_upkeep_cost = upkeep_y;
                        }
                        prev_is_day_flag = is_day;

                        // Ночная рутина: идём домой и спим, сбрасываем работу
                        if !is_day {
                            for c in citizens.iter_mut() {
                                // отменяем активную задачу/перенос
                                c.assigned_job = None;
                                c.carrying_log = false;
                                if c.state != CitizenState::Sleeping {
                                    if c.pos != c.home && !c.moving {
                                        game::plan_path(&world, c, c.home);
                                        c.state = CitizenState::GoingHome;
                                    }
                                    if !c.moving && c.pos == c.home {
                                        c.state = CitizenState::Sleeping;
                                        if !c.manual_workplace { c.workplace = None; }
                                        c.work_timer_ms = 0;
                                        c.carrying = None;
                                    }
                                }
                            }
                            // Сбросим захваты задач, чтобы утром их можно было перераспределить
                            for j in jobs.iter_mut() { j.taken = false; }
                        }
                        // Утро: разбудить спящих и отменить возвращение домой, если день начался
                        if is_day {
                            for c in citizens.iter_mut() {
                                match c.state {
                                    CitizenState::Sleeping | CitizenState::GoingHome => {
                                        c.state = CitizenState::Idle;
                                        c.moving = false; // отменим путь домой, пусть берёт работу
                                    }
                                    _ => {}
                                }
                            }
                            // направим вручную закреплённых к месту работы
                            for c in citizens.iter_mut() {
                                if c.manual_workplace { if let Some(wp) = c.workplace { if !c.moving { game::plan_path(&world, c, wp); c.state = CitizenState::GoingToWork; } } }
                            }
                        }

                        // Дневная рутина рабочих по зданиям (кроме дома/склада)
                        if is_day {
                            // Назначение рабочих
                            for b in buildings.iter() {
                                match b.kind {
                                    BuildingKind::House | BuildingKind::Warehouse => {}
                                    _ => {
                                        // считаем сколько уже назначено на это здание
                                        let current = citizens.iter().filter(|c| c.workplace == Some(b.pos)).count() as i32;
                                        if current >= b.workers_target { continue; }
                                            if let Some((ci, _)) = citizens.iter()
                                                .enumerate()
                                            .filter(|(_, c)| matches!(c.state, CitizenState::Idle | CitizenState::Sleeping) && !c.moving && !c.manual_workplace)
                                                .min_by_key(|(_, c)| (c.pos.x - b.pos.x).abs() + (c.pos.y - b.pos.y).abs())
                                            {
                                                let c = &mut citizens[ci];
                                                if matches!(c.state, CitizenState::Sleeping) && c.pos != c.home { continue; }
                                                c.workplace = Some(b.pos);
                                                c.target = b.pos;
                                            game::plan_path(&world, c, b.pos);
                                                c.moving = true;
                                                c.progress = 0.0;
                                                c.state = CitizenState::GoingToWork;
                                        }
                                    }
                                }
                            }
                            // Снижение числа работников: если фактических больше цели — освободим лишних (кроме ручных)
                            for b in buildings.iter() {
                                if matches!(b.kind, BuildingKind::House | BuildingKind::Warehouse) { continue; }
                                let mut assigned: Vec<usize> = citizens.iter()
                                    .enumerate()
                                    .filter(|(_, c)| c.workplace == Some(b.pos) && !c.manual_workplace)
                                    .map(|(i, _)| i)
                                    .collect();
                                let over = (assigned.len() as i32 - b.workers_target).max(0) as usize;
                                if over > 0 {
                                    // снимем часть: берём тех, кто дальше всего от здания
                                    assigned.sort_by_key(|&i| {
                                        let c = &citizens[i];
                                        (c.pos.x - b.pos.x).abs() + (c.pos.y - b.pos.y).abs()
                                    });
                                    for &i in assigned.iter().rev().take(over) {
                                        let c = &mut citizens[i];
                                        c.workplace = None;
                                        if matches!(c.state, CitizenState::GoingToWork | CitizenState::Working) {
                                            c.state = CitizenState::Idle;
                                            c.moving = false;
                                        }
                                    }
                                }
                            }
                        }

                        // 1) генерация задач лесорубками — ограничиваем по числу назначенных работников на лесорубку
                        if is_day {
                            for b in buildings.iter() {
                                if b.kind == BuildingKind::Lumberjack {
                                    // сколько работников закреплено на этой лесорубке
                                    let workers_here = citizens.iter().filter(|c| c.workplace == Some(b.pos) && c.fed_today).count() as i32;
                                    if workers_here <= 0 { continue; }
                                    // лимит задач = работников_here; считаем только Chop-задачи рядом
                                    let active_tasks_here = jobs.iter().filter(|j| match j.kind { JobKind::ChopWood { pos } => (pos.x - b.pos.x).abs() + (pos.y - b.pos.y).abs() <= 48, _ => false }).count() as i32;
                                    if active_tasks_here >= workers_here { continue; }
                                    // ищем ближайшее зрелое дерево; если нет в радиусе 24, расширяем до 32
                                    let search = |rad: i32| -> Option<IVec2> {
                                    let mut best: Option<(i32, IVec2)> = None;
                                        for dy in -rad..=rad { for dx in -rad..=rad {
                                        let np = IVec2::new(b.pos.x + dx, b.pos.y + dy);
                                        if matches!(world.tree_stage(np), Some(2)) {
                                            let d = dx.abs() + dy.abs();
                                            if best.map(|(bd, _)| d < bd).unwrap_or(true) { best = Some((d, np)); }
                                        }
                                    }}
                                        best.map(|(_,p)| p)
                                    };
                                    // посчитаем количество stage2 в базовом радиусе для дебага
                                    let mut _stage2_cnt = 0;
                                    for dy in -24..=24 { for dx in -24..=24 {
                                        if matches!(world.tree_stage(IVec2::new(b.pos.x+dx, b.pos.y+dy)), Some(2)) { _stage2_cnt += 1; }
                                    }}
                                    if let Some(np) = search(24).or_else(|| search(32)).or_else(|| search(48)).or_else(|| search(64)) {
                                        let already = jobs.iter().any(|j| match j.kind { JobKind::ChopWood { pos } => pos==np, JobKind::HaulWood { from, .. } => from==np });
                                        if !already {
                                            jobs.push(Job { id: { let id=next_job_id; next_job_id+=1; id }, kind: JobKind::ChopWood { pos: np }, taken: false, done: false });
                                            // println!("Лесорубка {:?}: публикуем ChopWood {:?} (stage2 в радиусе 24: {})", b.pos, np, stage2_cnt);
                                        }
                                    } else {
                                        // println!("Лесорубка {:?}: нет зрелых деревьев (stage2 в радиусе 24: {})", b.pos, stage2_cnt);
                                    }
                                }
                            }
                            // Гарантировать задачи на перенос для всех поленьев на земле
                            if !warehouses.is_empty() {
                                for li in logs_on_ground.iter() {
                                    if li.carried { continue; }
                                    let already = jobs.iter().any(|j| match j.kind { JobKind::HaulWood { from, .. } => from == li.pos, _ => false });
                                    if !already {
                                        if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - li.pos.x).abs() + (w.pos.y - li.pos.y).abs()).map(|w| w.pos) {
                                            jobs.push(Job { id: { let id=next_job_id; next_job_id+=1; id }, kind: JobKind::HaulWood { from: li.pos, to: dst }, taken: false, done: false });
                                        }
                                    }
                                }
                            }
                            // 2) назначение задач: ближайший к задаче свободный житель (только те, кто не назначен на работу)
                            // Назначаем только накормленных работников
                            jobs::assign_jobs_nearest_worker(&mut citizens, &mut jobs, &world, &buildings);
                            // 3) выполнение задач
                            jobs::process_jobs(&mut citizens, &mut jobs, &mut logs_on_ground, &mut warehouses, &mut resources, &buildings, &mut world, &mut next_job_id);
                        }
                        // перемещение жителей по пути (A*)
                        for c in citizens.iter_mut() {
                            if !c.moving {
                                c.idle_timer_ms += step_ms as i32;
                                c.progress = 0.0;
                                // если стоим > 5 секунд с назначенной задачей — сбросим её
                                if c.idle_timer_ms > 5000 { c.assigned_job = None; c.carrying_log = false; c.idle_timer_ms = 0; }
                                // смена состояний при прибытии
                                match c.state {
                                    CitizenState::GoingToWork => {
                                        // Не пускаем работать, если не накормлен
                                        if c.fed_today { c.state = CitizenState::Working; } else { c.state = CitizenState::Idle; }
                                    }
                                    CitizenState::GoingHome => { if c.pos == c.home { c.state = CitizenState::Sleeping; } }
                                    CitizenState::GoingToDeposit => {
                                        if let Some((kind, amt)) = c.carrying.take() {
                                            // кладём в ближайший склад (цель уже склад)
                                            if let Some(w) = warehouses.iter_mut().find(|w| w.pos == c.pos) {
                                                match kind {
                                                    ResourceKind::Wood => w.wood += amt,
                                                    ResourceKind::Stone => w.stone += amt,
                                                    ResourceKind::Clay => w.clay += amt,
                                                    ResourceKind::Bricks => w.bricks += amt,
                                                    ResourceKind::Wheat => w.wheat += amt,
                                                    ResourceKind::Flour => w.flour += amt,
                                                    ResourceKind::Bread => w.bread += amt,
                                                    ResourceKind::Fish => w.fish += amt,
                                                    ResourceKind::Gold => w.gold += amt,
                                                    ResourceKind::IronOre => w.iron_ore += amt,
                                                    ResourceKind::IronIngot => w.iron_ingots += amt,
                                                }
                                            }
                                            // возвращаемся к работе
                                            if let Some(wp) = c.workplace { game::plan_path(&world, c, wp); c.state = CitizenState::GoingToWork; }
                                        }
                                    }
                                    CitizenState::GoingToFetch => {
                                        if let Some(req) = c.pending_input.take() {
                                            // попытка забрать 1 ед. ресурса
                                            if let Some(w) = warehouses.iter_mut().find(|w| w.pos == c.pos) {
                                                let taken = match req {
                                                    ResourceKind::Wood => { if w.wood > 0 { w.wood -= 1; true } else { false } },
                                                    ResourceKind::Stone => { if w.stone > 0 { w.stone -= 1; true } else { false } },
                                                    ResourceKind::Clay => { if w.clay > 0 { w.clay -= 1; true } else { false } },
                                                    ResourceKind::Bricks => { if w.bricks > 0 { w.bricks -= 1; true } else { false } },
                                                    ResourceKind::Wheat => { if w.wheat > 0 { w.wheat -= 1; true } else { false } },
                                                    ResourceKind::Flour => { if w.flour > 0 { w.flour -= 1; true } else { false } },
                                                    ResourceKind::Bread => { if w.bread > 0 { w.bread -= 1; true } else { false } },
                                                    ResourceKind::Fish => { if w.fish > 0 { w.fish -= 1; true } else { false } },
                                                    ResourceKind::Gold => { if w.gold > 0 { w.gold -= 1; true } else { false } },
                                                    ResourceKind::IronOre => { if w.iron_ore > 0 { w.iron_ore -= 1; true } else { false } },
                                                    ResourceKind::IronIngot => { if w.iron_ingots > 0 { w.iron_ingots -= 1; true } else { false } },
                                                };
                                                if taken {
                                                    c.carrying = Some((req, 1));
                                                    if let Some(wp) = c.workplace { game::plan_path(&world, c, wp); c.state = CitizenState::GoingToWork; }
                                                } else {
                                                    c.state = CitizenState::Working; // ресурса нет
                                                }
                                            }
                                        }
                                    }
                                    CitizenState::Sleeping => {}
                                    _ => {}
                                }
                            } else {
                                c.idle_timer_ms = 0;
                                // если дорога пустая — идём к следующей точке пути
                                if c.pos == c.target {
                                    // достигнута вершина пути
                                    if c.path_index + 1 < c.path.len() { c.path_index += 1; c.target = c.path[c.path_index]; c.progress = 0.0; }
                                    else { c.moving = false; c.progress = 0.0; }
                                } else {
                                    // скорость шага зависит от целевой клетки: дорога быстрее, трава медленнее, лес ещё медленнее
                                    let mut _step_time_ms: f32 = 300.0; // базовая скорость (нравится на дорогах)
                                    if world.is_road(c.target) {
                                        _step_time_ms = 300.0;
                                    } else {
                                        use crate::types::TileKind::*;
                                        match world.get_tile(c.target.x, c.target.y) {
                                            Grass => _step_time_ms = 450.0,
                                            Forest => _step_time_ms = 600.0,
                                            Water => _step_time_ms = 300.0,
                                        }
                                    }
                                    c.progress += (step_ms / _step_time_ms) as f32;
                                    if c.progress >= 1.0 { c.pos = c.target; c.progress = 0.0; }
                                }
                            }
                        }

                        // Производство при работе у здания
                        if is_day {
                            for c in citizens.iter_mut() {
                                if !matches!(c.state, CitizenState::Working) { continue; }
                                if !c.fed_today { c.state = CitizenState::Idle; continue; }
                                let Some(wp) = c.workplace else { continue; };
                                if c.pos != wp { continue; }
                                if let Some(b) = buildings.iter().find(|b| b.pos == wp) {
                                    c.work_timer_ms += step_ms as i32;
                                    match b.kind {
                                        BuildingKind::StoneQuarry => {
                                            if c.carrying.is_none() && c.work_timer_ms >= 4000 {
                                                c.work_timer_ms = 0;
                                                if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                    c.carrying = Some((ResourceKind::Stone, 1));
                                                    game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                }
                                            }
                                        }
                                        BuildingKind::ClayPit => {
                                            if c.carrying.is_none() && c.work_timer_ms >= 4000 {
                                                c.work_timer_ms = 0;
                                                if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                    c.carrying = Some((ResourceKind::Clay, 1));
                                                    game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                }
                                            }
                                        }
                                        BuildingKind::IronMine => {
                                            if c.carrying.is_none() && c.work_timer_ms >= 5000 {
                                                c.work_timer_ms = 0;
                                                if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                    c.carrying = Some((ResourceKind::IronOre, 1));
                                                    game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                }
                                            }
                                        }
                                        BuildingKind::WheatField => {
                                            if c.carrying.is_none() && c.work_timer_ms >= 6000 {
                                                c.work_timer_ms = 0;
                                                if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                    c.carrying = Some((ResourceKind::Wheat, 1));
                                                    game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                }
                                            }
                                        }
                                        BuildingKind::Mill => {
                                            // нужна пшеница; если не несём пшеницу — идём за ней
                                            if !matches!(c.carrying, Some((ResourceKind::Wheat, _))) {
                                                let have_any = warehouses.iter().any(|w| w.wheat > 0);
                                                if have_any {
                                                    if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                        c.pending_input = Some(ResourceKind::Wheat);
                                                        c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToFetch;
                                                    }
                                                }
                                            } else {
                                                if c.work_timer_ms >= 5000 {
                                                    c.work_timer_ms = 0;
                                                    // consume carried wheat -> produce flour, then deliver
                                                    c.carrying = None;
                                                    if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                        c.carrying = Some((ResourceKind::Flour, 1));
                                                        game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                    }
                                                }
                                            }
                                        }
                                        BuildingKind::Kiln => {
                                            // нужна глина и дрова (wood)
                                            let has_clay = matches!(c.carrying, Some((ResourceKind::Clay, _)));
                                            if !has_clay {
                                                let have_any = warehouses.iter().any(|w| w.clay > 0);
                                                if have_any {
                                                    if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                        c.pending_input = Some(ResourceKind::Clay);
                                                        c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToFetch;
                                                    }
                                                }
                                            } else {
                                                if c.work_timer_ms >= 5000 {
                                                    c.work_timer_ms = 0;
                                                    // попытка списать 1 wood со складов
                                                    let mut ok = false;
                                                    'outer: for w in warehouses.iter_mut() {
                                                        if w.wood > 0 { w.wood -= 1; ok = true; break 'outer; }
                                                    }
                                                    if ok {
                                                        c.carrying = None; // глину потратили
                                                        if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                            c.carrying = Some((ResourceKind::Bricks, 1));
                                                         game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        BuildingKind::Bakery => {
                                            let has_flour = matches!(c.carrying, Some((ResourceKind::Flour, _)));
                                            if !has_flour {
                                                let have_any = warehouses.iter().any(|w| w.flour > 0);
                                                if have_any {
                                                    if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                        c.pending_input = Some(ResourceKind::Flour);
                                                        c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToFetch;
                                                    }
                                                }
                                            } else {
                                                if c.work_timer_ms >= 5000 {
                                                    c.work_timer_ms = 0;
                                                    // списать 1 wood
                                                    let mut ok = false;
                                                    'outer2: for w in warehouses.iter_mut() {
                                                        if w.wood > 0 { w.wood -= 1; ok = true; break 'outer2; }
                                                    }
                                                    if ok {
                                                        c.carrying = None; // муку потратили
                                                        if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                            c.carrying = Some((ResourceKind::Bread, 1));
                                                             game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        BuildingKind::Smelter => {
                                            // нужна IronOre и дрова (wood)
                                            let has_ore = matches!(c.carrying, Some((ResourceKind::IronOre, _)));
                                            if !has_ore {
                                                let have_any = warehouses.iter().any(|w| w.iron_ore > 0);
                                                if have_any {
                                                    if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                        c.pending_input = Some(ResourceKind::IronOre);
                                                        c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToFetch;
                                                    }
                                                }
                                            } else {
                                                if c.work_timer_ms >= 6000 {
                                                    c.work_timer_ms = 0;
                                                    // списать 1 wood
                                                    let mut ok = false;
                                                    for w in warehouses.iter_mut() { if w.wood > 0 { w.wood -= 1; ok = true; break; } }
                                                    if ok {
                                                        c.carrying = None; // руду потратили
                                                        if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                            c.carrying = Some((ResourceKind::IronIngot, 1));
                                                             game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        BuildingKind::Fishery => {
                                            if c.carrying.is_none() && c.work_timer_ms >= 5000 {
                                                const NB: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
                                                if NB.iter().any(|(dx,dy)| world.get_tile(b.pos.x+dx, b.pos.y+dy) == TileKind::Water) {
                                                    c.work_timer_ms = 0;
                                                    if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                        c.carrying = Some((ResourceKind::Fish, 1));
                                                        game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                    }
                                                }
                                            }
                                        }
                                        BuildingKind::Forester => {
                                            if c.work_timer_ms >= 4000 {
                                                c.work_timer_ms = 0;
                                                const R: i32 = 6;
                                                let mut best: Option<(i32, IVec2)> = None;
                                                for dy in -R..=R { for dx in -R..=R {
                                                    let p = IVec2::new(b.pos.x+dx, b.pos.y+dy);
                                                    let tk = world.get_tile(p.x, p.y);
                                                    if tk != TileKind::Water && !world.has_tree(p) && !world.is_occupied(p) && !world.is_road(p) && !world.is_road(IVec2::new(p.x-1, p.y-1)) {
                                                        let d = dx.abs() + dy.abs();
                                                        if best.map(|(bd,_)| d < bd).unwrap_or(true) { best = Some((d, p)); }
                                                    }
                                                }}
                                                if let Some((_, p)) = best { world.plant_tree(p); }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        accumulator_ms -= step_ms;
                        did_step = true;
                        if accumulator_ms > 10.0 * step_ms { accumulator_ms = 0.0; break; }
                    }
                }
                if did_step {
                    window.request_redraw();
                } else { window.request_redraw(); }
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

fn overlay_tint(frame: &mut [u8], fw: i32, fh: i32, [r,g,b,a]: [u8;4]) {
    if a == 0 { return; }
    let a = a as u32; let na = 255 - a;
    for y in 0..fh {
        let row = (y as usize) * (fw as usize) * 4;
        for x in 0..fw {
            let idx = row + (x as usize) * 4;
            let dr = frame[idx] as u32; let dg = frame[idx+1] as u32; let db = frame[idx+2] as u32;
            frame[idx]   = ((a * r as u32 + na * dr) / 255) as u8;
            frame[idx+1] = ((a * g as u32 + na * dg) / 255) as u8;
            frame[idx+2] = ((a * b as u32 + na * db) / 255) as u8;
            frame[idx+3] = 255;
        }
    }
}

// удалено: sat_mul_add (не используется)

// удалено: draw_iso_tile (перенесено в модуль render)

// удалено: draw_iso_tile_tinted (перенесено в модуль render)

// удалено: draw_iso_outline (перенесено в модуль render)

// удалено: draw_line (перенесено в модуль render)

// см. atlas::building_sprite_index

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
    // немного запаса вокруг экрана
    (min_tx - 2, min_ty - 2, max_tx + 2, max_ty + 2)
}

// удалено: локальный `building_cost` — используем `types::building_cost`

// удалено: локальные warehouses_total_wood/spend_wood — используем функции из types

// удалено: локальные simulate/plan_path — вынесены в модуль game

