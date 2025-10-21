use anyhow::Result;
use glam::{IVec2, Vec2};
mod types; use types::{TileKind, BuildingKind, Building, Resources, Citizen, Job, JobKind, LogItem, WarehouseStore, CitizenState, ResourceKind};
mod world; use world::World;
mod atlas; use atlas::{TileAtlas, BuildingAtlas};
mod render; // GPU rendering module
mod ui;
mod ui_gpu; // GPU версия UI
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
use gpu_renderer::GpuRenderer;
use std::time::Instant;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::atomic::{AtomicI32, Ordering};
 
// use std::fs; // перенесено в config
// use std::path::Path; // перенесено в config
use serde::{Serialize, Deserialize};
// use image::GenericImageView; // не нужен
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;
// duplicate import removed

// Глобальный масштаб клетки миникарты (px на клетку)
static MINIMAP_CELL_PX: AtomicI32 = AtomicI32::new(0);

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

#[derive(Clone, Debug)]
struct Firefly { pos: Vec2, vel: Vec2, phase: f32, life_s: f32 }

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

    // Инициализируем логгер для wgpu
    env_logger::init();

    let size = window.inner_size();
    let mut gpu_renderer = pollster::block_on(GpuRenderer::new(window.clone()))?;

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
    world.apply_biome_config(&config);

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
    let mut citizen_sprite: Option<(Vec<Vec<u8>>, i32, i32)> = None;
    // лица: ожидаем faces.png с 3 колонками (sad,neutral,happy), опционально 2 ряда (light/dark)
    // порядок: [sad_l, neutral_l, happy_l, sad_d, neutral_d, happy_d]
    let mut face_sprites: Option<(Vec<Vec<u8>>, i32, i32)> = None; // (sprites, cell_w, cell_h)
    let mut road_mode = false;
    let mut path_debug_mode = false;
    let mut biome_overlay_debug = false;
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
        println!("Загружено {} спрайтов зданий из buildings.png", sprites.len());
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
    // faces.png: 3 колонки по эмоциям, 1-2 ряда (light/dark)
    if let Ok(img) = image::open("assets/faces.png") {
        let img = img.to_rgba8();
        let (iw, ih) = img.dimensions();
        let cols = 3u32;
        if iw >= cols && iw % cols == 0 {
            let cell_w = iw / cols;
            if cell_w > 0 {
                let rows = (ih / cell_w).max(1);
                let cell_h = if rows > 0 { cell_w } else { ih };
                let mut sprites = Vec::new();
                let slice_cell = |cx: u32, cy: u32| -> Vec<u8> {
                    let x0 = cx * cell_w; let y0 = cy * cell_h;
                    let mut out = vec![0u8; (cell_w * cell_h * 4) as usize];
                    for y in 0..cell_h as usize {
                        let src = ((y0 as usize + y) * iw as usize + x0 as usize) * 4;
                        let dst = y * cell_w as usize * 4;
                        out[dst..dst + cell_w as usize * 4].copy_from_slice(&img.as_raw()[src..src + cell_w as usize * 4]);
                    }
                    out
                };
                let rows_clamped = rows.min(2); // используем максимум 2 ряда
                for ry in 0..rows_clamped { for cx in 0..cols { sprites.push(slice_cell(cx, ry)); } }
                face_sprites = Some((sprites, cell_w as i32, cell_h as i32));
            }
        }
    }
    // citizen.png: одиночный спрайт (квадратный предпочтительно), масштабируется под радиус маркера
    if let Ok(img) = image::open("assets/citizen.png") {
        let img = img.to_rgba8();
        let (iw, ih) = img.dimensions();
        citizen_sprite = Some((vec![img.to_vec()], iw as i32, ih as i32));
    }
    let mut water_anim_time: f32 = 0.0;
    // Ночные светлячки на экране
    let mut fireflies: Vec<Firefly> = Vec::new();
    // Простая погода
    let mut weather: WeatherKind = WeatherKind::Clear;
    let mut weather_timer_ms: f32 = 0.0;
    let mut weather_next_change_ms: f32 = choose_weather_duration_ms(weather, &mut rng);
    // Экономика: вчерашние итоги
    let mut last_tax_income: i32 = 0;
    let mut last_upkeep_cost: i32 = 0;
    let mut show_grid = false;
    let mut show_forest_overlay = false;
    let mut show_tree_stage_overlay = false;
    let mut show_ui = true;
    let mut cursor_xy = IVec2::new(0, 0);
    // Drag-to-build roads state
    let mut left_mouse_down: bool = false;
    let mut drag_prev_tile: Option<IVec2> = None;
    let mut drag_road_state: Option<bool> = None; // Some(true)=build, Some(false)=erase
    let mut drag_anchor_tile: Option<IVec2> = None; // начало протяжки
    let mut preview_road_path: Vec<IVec2> = Vec::new();
    let mut fps_ema: f32 = 60.0;
    // удалён дубликат переменной show_ui
    // выбранный житель для ручного назначения на работу (отключено)
    let _selected_citizen: Option<usize> = None; // больше не используется для назначения
    // активная панель здания (по клику)
    let mut active_building_panel: Option<IVec2> = None;

    // Консоль разработчика
    let mut console_open: bool = false;
    let mut console_input: String = String::new();
    let mut console_log: Vec<String> = vec!["Console: type 'help' for commands".to_string()];

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
                        // Тоггл консоли на '/'
                        if let PhysicalKey::Code(KeyCode::Slash) = key { console_open = !console_open; return; }
                        if console_open {
                            // В консоли обычные бинды не работают
                            match key {
                                PhysicalKey::Code(KeyCode::Enter) => {
                                    if !console_input.is_empty() {
                                        let cmd = console_input.clone();
                                        console_log.push(format!("> {}", cmd));
                                        handle_console_command(&cmd, &mut console_log, &mut resources, &mut weather, &mut world_clock_ms, &mut world, &mut biome_overlay_debug);
                                        console_input.clear();
                                    }
                                    return;
                                }
                                PhysicalKey::Code(KeyCode::Backspace) => { console_input.pop(); return; }
                                PhysicalKey::Code(KeyCode::Escape) => { console_open = false; return; }
                                // Небольшой набор ASCII: буквы/цифры/символы — добавляем в строку ввода
                                PhysicalKey::Code(KeyCode::Space) => { console_input.push(' '); return; }
                                PhysicalKey::Code(KeyCode::Digit0) => { console_input.push('0'); return; }
                                PhysicalKey::Code(KeyCode::Digit1) => { console_input.push('1'); return; }
                                PhysicalKey::Code(KeyCode::Digit2) => { console_input.push('2'); return; }
                                PhysicalKey::Code(KeyCode::Digit3) => { console_input.push('3'); return; }
                                PhysicalKey::Code(KeyCode::Digit4) => { console_input.push('4'); return; }
                                PhysicalKey::Code(KeyCode::Digit5) => { console_input.push('5'); return; }
                                PhysicalKey::Code(KeyCode::Digit6) => { console_input.push('6'); return; }
                                PhysicalKey::Code(KeyCode::Digit7) => { console_input.push('7'); return; }
                                PhysicalKey::Code(KeyCode::Digit8) => { console_input.push('8'); return; }
                                PhysicalKey::Code(KeyCode::Digit9) => { console_input.push('9'); return; }
                                PhysicalKey::Code(KeyCode::Minus) => { console_input.push('-'); return; }
                                PhysicalKey::Code(KeyCode::Equal) => { console_input.push('='); return; }
                                PhysicalKey::Code(KeyCode::Comma) => { console_input.push(','); return; }
                                PhysicalKey::Code(KeyCode::Period) => { console_input.push('.'); return; }
                                PhysicalKey::Code(KeyCode::Slash) => { console_input.push('/'); return; }
                                PhysicalKey::Code(KeyCode::Backslash) => { console_input.push('\\'); return; }
                                PhysicalKey::Code(KeyCode::KeyA) => { console_input.push('a'); return; }
                                PhysicalKey::Code(KeyCode::KeyB) => { console_input.push('b'); return; }
                                PhysicalKey::Code(KeyCode::KeyC) => { console_input.push('c'); return; }
                                PhysicalKey::Code(KeyCode::KeyD) => { console_input.push('d'); return; }
                                PhysicalKey::Code(KeyCode::KeyE) => { console_input.push('e'); return; }
                                PhysicalKey::Code(KeyCode::KeyF) => { console_input.push('f'); return; }
                                PhysicalKey::Code(KeyCode::KeyG) => { console_input.push('g'); return; }
                                PhysicalKey::Code(KeyCode::KeyH) => { console_input.push('h'); return; }
                                PhysicalKey::Code(KeyCode::KeyI) => { console_input.push('i'); return; }
                                PhysicalKey::Code(KeyCode::KeyJ) => { console_input.push('j'); return; }
                                PhysicalKey::Code(KeyCode::KeyK) => { console_input.push('k'); return; }
                                PhysicalKey::Code(KeyCode::KeyL) => { console_input.push('l'); return; }
                                PhysicalKey::Code(KeyCode::KeyM) => { console_input.push('m'); return; }
                                PhysicalKey::Code(KeyCode::KeyN) => { console_input.push('n'); return; }
                                PhysicalKey::Code(KeyCode::KeyO) => { console_input.push('o'); return; }
                                PhysicalKey::Code(KeyCode::KeyP) => { console_input.push('p'); return; }
                                PhysicalKey::Code(KeyCode::KeyQ) => { console_input.push('q'); return; }
                                PhysicalKey::Code(KeyCode::KeyR) => { console_input.push('r'); return; }
                                PhysicalKey::Code(KeyCode::KeyS) => { console_input.push('s'); return; }
                                PhysicalKey::Code(KeyCode::KeyT) => { console_input.push('t'); return; }
                                PhysicalKey::Code(KeyCode::KeyU) => { console_input.push('u'); return; }
                                PhysicalKey::Code(KeyCode::KeyV) => { console_input.push('v'); return; }
                                PhysicalKey::Code(KeyCode::KeyW) => { console_input.push('w'); return; }
                                PhysicalKey::Code(KeyCode::KeyX) => { console_input.push('x'); return; }
                                PhysicalKey::Code(KeyCode::KeyY) => { console_input.push('y'); return; }
                                PhysicalKey::Code(KeyCode::KeyZ) => { console_input.push('z'); return; }
                                _ => { }
                            }
                            // Консоль перехватывает ввод
                            return;
                        }
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
                    hovered_tile = screen_to_tile_px(mx, my, width_i32, height_i32, cam_snap, atlas.half_w, atlas.half_h, zoom);
                    if left_mouse_down && road_mode {
                        // считаем только предпросмотр, без применения
                        if drag_anchor_tile.is_none() {
                            if let Some(curr) = hovered_tile {
                                if drag_road_state.is_none() { drag_road_state = Some(!world.is_road(curr)); }
                                drag_anchor_tile = Some(curr);
                            }
                        }
                        if let (Some(anchor), Some(curr)) = (drag_anchor_tile, hovered_tile) {
                            preview_road_path.clear();
                            let mut x = anchor.x; let mut y = anchor.y;
                            let sx = (curr.x - x).signum();
                            let sy = (curr.y - y).signum();
                            preview_road_path.push(IVec2::new(x, y));
                            while x != curr.x { x += sx; preview_road_path.push(IVec2::new(x, y)); }
                            while y != curr.y { y += sy; preview_road_path.push(IVec2::new(x, y)); }
                        }
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == winit::event::MouseButton::Left && state == ElementState::Pressed {
                        left_mouse_down = true;
                        if road_mode {
                            if let Some(tp) = hovered_tile {
                                let on = !world.is_road(tp);
                                drag_prev_tile = Some(tp);
                                drag_road_state = Some(on);
                                drag_anchor_tile = Some(tp);
                                preview_road_path.clear();
                                preview_road_path.push(tp);
                                return;
                            }
                        }
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
                            if path_debug_mode {
                                match (path_sel_a, path_sel_b) { (None, _) => { path_sel_a = Some(tp); last_path=None; }, (Some(_), None) => { path_sel_b = Some(tp); }, (Some(_), Some(_)) => { path_sel_a = Some(tp); path_sel_b=None; last_path=None; } }
                                if let (Some(a), Some(b)) = (path_sel_a, path_sel_b) { last_path = crate::path::astar(&world, a, b, 20_000); }
                                return;
                            }
                        }
                    } else if button == winit::event::MouseButton::Left && state == ElementState::Released {
                        left_mouse_down = false;
                        if road_mode {
                            if let Some(on) = drag_road_state {
                                for p in preview_road_path.iter() { world.set_road(*p, on); }
                                // Очищаем предпросмотр дорог после применения
                                gpu_renderer.clear_road_preview();
                            }
                        }
                        drag_prev_tile = None;
                        drag_road_state = None;
                        drag_anchor_tile = None;
                        preview_road_path.clear();
                    }
                }
                WindowEvent::Resized(new_size) => {
                    width_i32 = new_size.width as i32;
                    height_i32 = new_size.height as i32;
                    gpu_renderer.resize(new_size);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let factor = match delta {
                        MouseScrollDelta::LineDelta(_, y) => if y > 0.0 { 1.1 } else { 0.9 },
                        MouseScrollDelta::PixelDelta(p) => if p.y > 0.0 { 1.1 } else { 0.9 },
                    };
                    zoom = (zoom * factor).clamp(0.5, 8.0);
                }
                WindowEvent::RedrawRequested => {
                    if MINIMAP_CELL_PX.load(Ordering::Relaxed) == 0 {
                        let s0 = ui::ui_scale(height_i32, config.ui_scale_base);
                        MINIMAP_CELL_PX.store(3 * s0, Ordering::Relaxed);
                    }

                    // Обновим атлас для текущего зума
                    atlas.ensure_zoom(zoom);

                    // Границы видимых тайлов через инверсию проекции
                    let (min_tx, min_ty, max_tx, max_ty) = visible_tile_bounds_px(width_i32, height_i32, cam_px, atlas.half_w, atlas.half_h, zoom);
                    // Подготовим процедурный атлас дорог под текущий масштаб
                    road_atlas.ensure_zoom(atlas.half_w, atlas.half_h);
                    // Закажем генерацию колец чанков
                    world.schedule_ring(min_tx, min_ty, max_tx, max_ty);
                    // Интегрируем готовые чанки (non-blocking)
                    world.integrate_ready_chunks();

                    // Обновляем камеру GPU рендерера
                    gpu_renderer.update_camera(cam_px.x, cam_px.y, zoom);

                    // Подготавливаем тайлы для GPU рендеринга (с подсветкой при наведении)
                    gpu_renderer.prepare_tiles(&mut world, &atlas, min_tx, min_ty, max_tx, max_ty, hovered_tile);
                    
                    // Подготавливаем структуры (здания и деревья) для GPU рендеринга с правильной сортировкой
                    if buildings_dirty {
                        buildings.sort_by_key(|b| b.pos.x + b.pos.y);
                        buildings_dirty = false;
                    }
                    gpu_renderer.prepare_structures(&mut world, &buildings, &building_atlas, &tree_atlas, &atlas, min_tx, min_ty, max_tx, max_ty);
                    gpu_renderer.prepare_citizens(&citizens, &buildings, &atlas);
                    gpu_renderer.prepare_roads(&mut world, &road_atlas, &atlas, min_tx, min_ty, max_tx, max_ty);
                    
                    // Предпросмотр дорог при перетаскивании
                    if left_mouse_down && road_mode && !preview_road_path.is_empty() {
                        let is_building = drag_road_state.unwrap_or(true);
                        gpu_renderer.prepare_road_preview(&preview_road_path, is_building, &atlas);
                    }

                    // TODO: Временно комментируем CPU рендеринг, пока не реализуем полный GPU пайплайн
                    let cam_snap = Vec2::new(cam_px.x.round(), cam_px.y.round());
                    let screen_center = IVec2::new(width_i32 / 2, height_i32 / 2);
                    
                    // Центр экрана нужен для некоторых функций пока
                    // let water_frame = ((water_anim_time / 120.0) as usize) % atlas.water_frames.len().max(1);
                    // render::map::draw_terrain_and_overlays(...);
                    
                    // Базовый GPU рендеринг тайлов

                    // TODO: Временно комментируем рендеринг поленьев
                    // for li in &logs_on_ground { ... }

                    // TODO: Временно комментируем подсветку ховера / призрак здания
                    // if let Some(_tp) = screen_to_tile_px(...) { ... }

                    // TODO: Временно комментируем логику предпросмотра дорог
                    // Страховка предпросмотра: при зажатой ЛКМ пересчитываем путь
                    if left_mouse_down && road_mode {
                        if let (Some(anchor), Some(curr)) = (drag_anchor_tile, hovered_tile) {
                            preview_road_path.clear();
                            let mut x = anchor.x; let mut y = anchor.y;
                            let sx = (curr.x - x).signum();
                            let sy = (curr.y - y).signum();
                            preview_road_path.push(IVec2::new(x, y));
                            while x != curr.x { x += sx; preview_road_path.push(IVec2::new(x, y)); }
                            while y != curr.y { y += sy; preview_road_path.push(IVec2::new(x, y)); }
                        }
                    }

                    /* TODO: Временно комментируем весь CPU рендеринг
                    // Отрисовка предпросмотра дороги
                    if left_mouse_down && road_mode && !preview_road_path.is_empty() {
                        let build = drag_road_state.unwrap_or(true);
                        let hover_off = ((atlas.half_h as f32) * 0.5).round() as i32;
                        let fill_col = if build { [120, 200, 120, 90] } else { [200, 100, 100, 90] };
                        let line_col = if build { [120, 220, 120, 255] } else { [220, 120, 120, 255] };
                        for tp in preview_road_path.iter() {
                            let sp = render::map::world_to_screen(&atlas, screen_center, cam_snap, tp.x, tp.y);
                            render::tiles::draw_iso_tile_tinted(frame, width_i32, height_i32, sp.x, sp.y + hover_off, atlas.half_w, atlas.half_h, fill_col);
                            render::tiles::draw_iso_outline(frame, width_i32, height_i32, sp.x, sp.y + hover_off, atlas.half_w, atlas.half_h, line_col);
                        }
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
                    */ 

                    // TODO: Остальные эффекты будут портированы на GPU позже
                    // - Граждане (цветные кружки)
                    // - Погодные эффекты (дождь, снег, туман)
                    // - Ночное освещение (окна домов, факелы)
                    // - UI рендеринг

                    /* TODO: Портировать на GPU позже
                    // Оверлей день/ночь
                    let t = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
                    let angle = t * std::f32::consts::TAU;
                    let daylight = 0.5 - 0.5 * angle.cos();
                    let darkness = (1.0 - daylight).max(0.0);
                    let night_strength = (darkness.powf(1.4) * 180.0).min(200.0) as u8;
                    if night_strength > 0 { overlay_tint(frame, width_i32, height_i32, [18, 28, 60, night_strength]); }

                    // Погодные эффекты (простые оверлеи)
                    match weather {
                        WeatherKind::Clear => {}
                        WeatherKind::Rain => {
                            // синий прохладный фильтр
                            overlay_tint(frame, width_i32, height_i32, [40, 60, 100, 40]);
                        }
                        WeatherKind::Fog => {
                            // сероватая дымка
                            overlay_tint(frame, width_i32, height_i32, [160, 160, 160, 50]);
                            // плавная анимация тумана (2D-синусоидальное поле альфы)
                            overlay_fog(frame, width_i32, height_i32, water_anim_time);
                        }
                        WeatherKind::Snow => {
                            // холодный свет
                            overlay_tint(frame, width_i32, height_i32, [220, 230, 255, 40]);
                        }
                    }

                    // Осадки (частицы) — простой, дешёвый слой
                    match weather {
                        WeatherKind::Rain => {
                            let sx_step = 24; let sy_step = 48;
                            let speed = 120.0; // px/сек (условно)
                            let t = water_anim_time / 1000.0 * speed; // в px
                            for x in (0..width_i32).step_by(sx_step as usize) {
                                for y in (0..height_i32).step_by(sy_step as usize) {
                                    let off = ((x as f32 * 0.37 + y as f32 * 0.11 + t) % sy_step as f32) as i32;
                                    let sx = x + off / 3; let sy = y + off;
                                    let x0 = sx; let y0 = sy; let x1 = sx - 2; let y1 = sy + 8;
                                    render::tiles::draw_line(frame, width_i32, height_i32, x0, y0, x1, y1, [150, 180, 230, 255]);
                                }
                            }
                        }
                        WeatherKind::Snow => {
                            // Сетка как у дождя → сопоставимая плотность частиц
                            let sx_step = 24; let sy_step = 48;
                            let speed = 22.0; // медленнее дождя
                            let t = water_anim_time / 1000.0 * speed;
                            for x in (0..width_i32).step_by(sx_step as usize) {
                                for y in (0..height_i32).step_by(sy_step as usize) {
                                    let off = ((x as f32 * 0.17 + y as f32 * 0.07 + t) % sy_step as f32) as i32;
                                    let sway = ((t * 0.15 + x as f32 * 0.30).sin() * 3.0) as i32;
                                    let sx = (x + sway).clamp(0, width_i32 - 1);
                                    let sy = y + off;
                                    if sx >= 0 && sy >= 0 && sx < width_i32 && sy < height_i32 {
                                        let col = [245, 245, 250, 255];
                                        let size = 3;
                                        for oy in 0..size {
                                            let yy = sy + oy;
                                            if yy < 0 || yy >= height_i32 { continue; }
                                            let x0 = sx.max(0);
                                            let x1 = (sx + size - 1).min(width_i32 - 1);
                                            render::tiles::draw_line(frame, width_i32, height_i32, x0, yy, x1, yy, col);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    // Ночное освещение: окна домов и факелы на дорогах
                    {
                        let t_ms = world_clock_ms;
                        let t = t_ms / 1000.0;
                        // Порог ночи такой же как в симуляции
                        let tt = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
                        let angle = tt * std::f32::consts::TAU;
                        let daylight = 0.5 - 0.5 * angle.cos();
                        let is_night = daylight <= 0.25;
                        if is_night {
                            // Масштаб от размера тайла
                            let base_r = (atlas.half_w as f32 * 0.35).round() as i32;
                            // 1) Окна домов: по 2 «светлячка» на дом
                            for b in &buildings {
                                if b.kind != BuildingKind::House { continue; }
                                let mx = b.pos.x; let my = b.pos.y;
                                if mx < min_tx || my < min_ty || mx > max_tx || my > max_ty { continue; }
                                let sp = render::map::world_to_screen(&atlas, screen_center, cam_snap, mx, my);
                                // Псевдо-случайная фаза и число окон от координат
                                let hash = ((mx as i64).wrapping_mul(73856093) ^ (my as i64).wrapping_mul(19349663)) as i64;
                                let phase = (hash as f32).to_bits() as u32 % 1000;
                                let flicker = ( (t + (phase as f32)*0.001).sin() * 0.5 + 0.5 );
                                // позиции «окон» около основания дома
                                let y0 = sp.y + (atlas.half_h as f32 * 0.10) as i32;
                                let x0 = sp.x - (atlas.half_w as f32 * 0.25) as i32;
                                let x1 = sp.x + (atlas.half_w as f32 * 0.25) as i32;
                                let a0 = (170.0 + 70.0 * flicker).min(240.0) as u8;
                                render::tiles::draw_soft_glow(frame, width_i32, height_i32, x0, y0, (base_r as f32 * 0.60) as i32, [255, 210, 130], a0);
                                render::tiles::draw_soft_glow(frame, width_i32, height_i32, x1, y0, (base_r as f32 * 0.60) as i32, [255, 220, 150], a0);
                            }
                            // 2) Факелы на дорогах: не на каждой клетке, разреженно
                            for my in min_ty..=max_ty { for mx in min_tx..=max_tx {
                                if !world.is_road(glam::IVec2::new(mx, my)) { continue; }
                                // Разрежение по хешу координат, ~1 факел на 6 клеток
                                let h = ((mx as i64).wrapping_mul(2654435761) ^ (my as i64).wrapping_mul(1597334677)) & 0x7fffffff;
                                if (h % 6) != 0 { continue; }
                                let sp = render::map::world_to_screen(&atlas, screen_center, cam_snap, mx, my);
                                // позиция факела — верхняя грань ромба чуть выше центра
                                let tx = sp.x + (atlas.half_w as f32 * 0.30) as i32;
                                let ty = sp.y - (atlas.half_h as f32 * 0.20) as i32;
                                // Мягкое мигание
                                let phase = ((h as u32) % 1000) as f32 * 0.0031;
                                let flick = ( (t * 2.3 + phase).sin() * 0.5 + 0.5 ) * 0.7 + 0.3;
                                let a = (170.0 * flick).min(230.0) as u8;
                                render::tiles::draw_soft_glow(frame, width_i32, height_i32, tx, ty, (base_r as f32 * 0.55) as i32, [255, 200, 120], a);
                                render::tiles::draw_soft_glow(frame, width_i32, height_i32, tx, ty + 2, (base_r as f32 * 0.85) as i32, [255, 140, 60], (a as f32 * 0.7) as u8);
                            }}
                            // 3) Летающие светлячки (экраные координаты)
                            for f in &fireflies {
                                let x = f.pos.x.round() as i32;
                                let y = f.pos.y.round() as i32;
                                let flick = ((t * 5.0 + f.phase).sin() * 0.5 + 0.5) * 0.6 + 0.4;
                                let a = (210.0 * flick).min(230.0) as u8;
                                // Меньший размер, больше точек
                                render::tiles::draw_soft_glow(frame, width_i32, height_i32, x, y, (atlas.half_w as f32 * 0.22) as i32, [255, 200, 120], (a as f32 * 0.65) as u8);
                                render::tiles::draw_soft_glow(frame, width_i32, height_i32, x, y, (atlas.half_w as f32 * 0.14) as i32, [255, 240, 180], a);
                            }
                        }
                    }

                    // Мини-карта теперь рендерится в ui_gpu::draw_ui_gpu()

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
                        // Параметры погоды для UI: короткий лейбл
                        let (wlabel, wcol): (&[u8], [u8;4]) = match weather {
                            WeatherKind::Clear => (b"CLEAR", [180,200,120,255]),
                            WeatherKind::Rain => (b"RAIN", [90,120,200,255]),
                            WeatherKind::Fog => (b"FOG", [160,160,160,255]),
                            WeatherKind::Snow => (b"SNOW", [220,230,255,255]),
                        };

                        ui::draw_ui(
                            frame, width_i32, height_i32,
                            &visible, total_visible_wood, pop_show, selected_building,
                            fps_ema, speed_mult, paused, config.ui_scale_base, ui_category, day_progress,
                            idle, working, sleeping, hauling, fetching, cursor_xy.x, cursor_xy.y,
                            avg_hap, tax_rate, ui_tab, food_policy,
                            last_tax_income, last_upkeep_cost,
                            housing_used, housing_cap,
                            wlabel, wcol,
                        );
                        if console_open {
                            let s = ui::ui_scale(height_i32, config.ui_scale_base);
                            ui::draw_console(frame, width_i32, height_i32, s, &console_input, &console_log);
                        }
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
                                // Строка про биом и эффект
                                let biome_label: Option<Vec<u8>> = {
                                    use crate::types::BiomeKind::*;
                                    let bm = world.biome(b.pos);
                                    match (bm, b.kind) {
                                        (Swamp, BuildingKind::Lumberjack) => Some(format!("Biome: Swamp  x{:.2}", config.biome_swamp_lumberjack_wmul).into_bytes()),
                                        (Rocky, BuildingKind::StoneQuarry) => Some(format!("Biome: Rocky  x{:.2}", config.biome_rocky_stone_wmul).into_bytes()),
                                        (Meadow, BuildingKind::WheatField) => Some(format!("Biome: Meadow  x{:.2}", config.biome_meadow_wheat_wmul).into_bytes()),
                                        (Swamp, BuildingKind::WheatField) => Some(format!("Biome: Swamp  x{:.2}", config.biome_swamp_wheat_wmul).into_bytes()),
                                        _ => None,
                                    }
                                };
                                let biome_label_ref = biome_label.as_deref();
                                ui::draw_building_panel(frame, width_i32, height_i32, ui::ui_scale(height_i32, config.ui_scale_base), b.kind, workers_current, b.workers_target, prod, cons, biome_label_ref);
                            }
                        }

                        // Убрали тултип биома на наведении — показываем эффект в панели здания
                    }

                    ЗАКРЫТО ВРЕМЕННОЕ КОММЕНТИРОВАНИЕ CPU РЕНДЕРИНГА */
                    
                    // GPU рендеринг заменяет весь CPU рендеринг
                    
                    // UI наложение - портируем полный UI из CPU версии на GPU
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
                        // Параметры погоды для UI: короткий лейбл
                        let (wlabel, wcol): (&[u8], [u8;4]) = match weather {
                            WeatherKind::Clear => (b"CLEAR", [180,200,120,255]),
                            WeatherKind::Rain => (b"RAIN", [90,120,200,255]),
                            WeatherKind::Fog => (b"FOG", [160,160,160,255]),
                            WeatherKind::Snow => (b"SNOW", [220,230,255,255]),
                        };

                        // GPU UI рендеринг через фабрику ui_gpu
                        let wcol_f32 = [wcol[0] as f32 / 255.0, wcol[1] as f32 / 255.0, wcol[2] as f32 / 255.0, wcol[3] as f32 / 255.0];
                        ui_gpu::draw_ui_gpu(
                            &mut gpu_renderer,
                            width_i32,
                            height_i32,
                            &visible,
                            visible.wood,
                            pop_show,
                            selected_building,
                            fps_ema,
                            speed_mult,
                            paused,
                            config.ui_scale_base,
                            ui_category,
                            day_progress,
                            idle,
                            working,
                            sleeping,
                            hauling,
                            fetching,
                            avg_hap,
                            tax_rate,
                            ui_tab,
                            food_policy,
                            wlabel,
                            wcol_f32,
                            // Данные для миникарты
                            &mut world,
                            &buildings,
                            cam_px.x,
                            cam_px.y,
                            MINIMAP_CELL_PX.load(Ordering::Relaxed).max(1),
                        );
                    } else {
                        // Если UI выключен, все равно очищаем
                        gpu_renderer.clear_ui();
                    }
                    
                    // Ночное освещение (затемнение) - поверх всего
                    let t = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
                    let angle = t * std::f32::consts::TAU;
                    let daylight = 0.5 - 0.5 * angle.cos();
                    let darkness = (1.0 - daylight).max(0.0);
                    let night_strength = (darkness.powf(1.4) * 180.0).min(200.0) as u8;
                    if night_strength > 0 {
                        let alpha = night_strength as f32 / 255.0;
                        gpu_renderer.apply_screen_tint([18.0/255.0, 28.0/255.0, 60.0/255.0, alpha]);
                    }
                    
                    // Погодные эффекты (цветные оверлеи)
                    match weather {
                        WeatherKind::Clear => {}
                        WeatherKind::Rain => {
                            // синий прохладный фильтр
                            gpu_renderer.apply_screen_tint([40.0/255.0, 60.0/255.0, 100.0/255.0, 40.0/255.0]);
                        }
                        WeatherKind::Fog => {
                            // сероватая дымка
                            gpu_renderer.apply_screen_tint([160.0/255.0, 160.0/255.0, 160.0/255.0, 50.0/255.0]);
                            // TODO: добавить анимированный туман
                        }
                        WeatherKind::Snow => {
                            // холодный свет
                            gpu_renderer.apply_screen_tint([220.0/255.0, 230.0/255.0, 255.0/255.0, 40.0/255.0]);
                        }
                    }
                    // TODO: добавить частицы осадков (дождь, снег)
                    
                    // GPU рендеринг
                    if let Err(err) = gpu_renderer.render() {
                        eprintln!("gpu_renderer.render() failed: {err}");
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
                                    // запрет: без моста нельзя идти в воду
                                    {
                                        use crate::types::TileKind::*;
                                        let k = world.get_tile(c.target.x, c.target.y);
                                        if matches!(k, Water) && !world.is_road(c.target) {
                                            c.moving = false; c.progress = 0.0; c.path.clear();
                                            continue;
                                        }
                                    }
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
                                    // модификатор погоды на скорость циклов производства (по типу здания)
                                    let wmul = {
                                        // базовый множитель от погоды
                                        let w = game::production_weather_wmul(weather, b.kind);
                                        // биомный множитель от клетки здания
                                        use crate::types::BiomeKind::*;
                                        let bm = world.biome(b.pos);
                                        let cfgb = &config;
                                        let bmul = match (bm, b.kind) {
                                            (Swamp, BuildingKind::Lumberjack) => cfgb.biome_swamp_lumberjack_wmul,
                                            (Rocky, BuildingKind::StoneQuarry) => cfgb.biome_rocky_stone_wmul,
                                            _ => 1.00,
                                        };
                                        w * bmul
                                    };
                                    match b.kind {
                                        BuildingKind::StoneQuarry => {
                                            if c.carrying.is_none() && c.work_timer_ms >= (4000.0 * wmul) as i32 {
                                                c.work_timer_ms = 0;
                                                if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                    c.carrying = Some((ResourceKind::Stone, 1));
                                                    game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                }
                                            }
                                        }
                                        BuildingKind::ClayPit => {
                                            if c.carrying.is_none() && c.work_timer_ms >= (4000.0 * wmul) as i32 {
                                                c.work_timer_ms = 0;
                                                if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                    c.carrying = Some((ResourceKind::Clay, 1));
                                                    game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                }
                                            }
                                        }
                                        BuildingKind::IronMine => {
                                            if c.carrying.is_none() && c.work_timer_ms >= (5000.0 * wmul) as i32 {
                                                c.work_timer_ms = 0;
                                                if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                    c.carrying = Some((ResourceKind::IronOre, 1));
                                                    game::plan_path(&world, c, dst); c.state = CitizenState::GoingToDeposit;
                                                }
                                            }
                                        }
                                        BuildingKind::WheatField => {
                                            // учтём биом поля (Meadow быстрее, Swamp медленнее)
                                            let bmul = {
                                                use crate::types::BiomeKind::*;
                                                match world.biome(b.pos) {
                                                    Meadow => config.biome_meadow_wheat_wmul,
                                                    Swamp => config.biome_swamp_wheat_wmul,
                                                    _ => 1.0,
                                                }
                                            };
                                            if c.carrying.is_none() && c.work_timer_ms >= (6000.0 * wmul * bmul) as i32 {
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
                                                if c.work_timer_ms >= (5000.0 * wmul) as i32 {
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
                                                if c.work_timer_ms >= (5000.0 * wmul) as i32 {
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
                                                if c.work_timer_ms >= (6000.0 * wmul) as i32 {
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
                                            if c.carrying.is_none() && c.work_timer_ms >= (5000.0 * wmul) as i32 {
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
                                            if c.work_timer_ms >= (4000.0 * wmul) as i32 {
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
                // Погода: случайная длительность и вероятности смены
                weather_timer_ms += frame_ms;
                if weather_timer_ms >= weather_next_change_ms {
                    weather_timer_ms = 0.0;
                    weather = pick_next_weather(weather, &mut rng);
                    weather_next_change_ms = choose_weather_duration_ms(weather, &mut rng);
                }

                // Обновление светлячков в реальном времени (каждый кадр)
                {
                    let tt = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
                    let angle = tt * std::f32::consts::TAU;
                    let daylight = 0.5 - 0.5 * angle.cos();
                    let is_night = daylight <= 0.25;
                    // Таргет количество светлячков зависит от ночи и размера экрана
                    let target = if is_night { ((width_i32 * height_i32) as f32 / 60000.0).round().clamp(10.0, 48.0) as usize } else { 0 };
                    // Спавн/удаление
                    if fireflies.len() < target {
                        let need = target - fireflies.len();
                        for _ in 0..need {
                            let x = rng.gen_range(0.0..width_i32 as f32);
                            let y = rng.gen_range(0.0..height_i32 as f32);
                            let speed = rng.gen_range(8.0..20.0);
                            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                            let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
                            let phase = rng.gen_range(0.0..std::f32::consts::TAU);
                            let life_s = rng.gen_range(6.0..14.0);
                            fireflies.push(Firefly { pos: Vec2::new(x, y), vel, phase, life_s });
                        }
                    } else if fireflies.len() > target {
                        fireflies.truncate(target);
                    }
                    // Дрейф и границы
                    let dt = frame_ms / 1000.0;
                    for f in fireflies.iter_mut() {
                        // чуть блуждаем синусом
                        let sway = Vec2::new((water_anim_time * 0.0016 + f.phase).sin(), (water_anim_time * 0.0021 + f.phase * 0.7).cos()) * 10.0;
                        f.pos += (f.vel * 0.25 + sway) * dt;
                        // обруливаем края мягко
                        if f.pos.x < -20.0 { f.pos.x = -20.0; f.vel.x = f.vel.x.abs(); }
                        if f.pos.y < -20.0 { f.pos.y = -20.0; f.vel.y = f.vel.y.abs(); }
                        if f.pos.x > width_i32 as f32 + 20.0 { f.pos.x = width_i32 as f32 + 20.0; f.vel.x = -f.vel.x.abs(); }
                        if f.pos.y > height_i32 as f32 + 20.0 { f.pos.y = height_i32 as f32 + 20.0; f.vel.y = -f.vel.y.abs(); }
                        f.life_s -= dt;
                    }
                    fireflies.retain(|f| f.life_s > 0.0);
                }

                window.request_redraw();
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WeatherKind { Clear, Rain, Fog, Snow }

fn choose_weather_duration_ms(current: WeatherKind, rng: &mut StdRng) -> f32 {
    // Базовые интервалы (в секундах), затем добавляем разброс
    let (base_min, base_max) = match current {
        WeatherKind::Clear => (60.0, 120.0),
        WeatherKind::Rain => (40.0, 90.0),
        WeatherKind::Fog => (30.0, 70.0),
        WeatherKind::Snow => (50.0, 100.0),
    };
    let sec: f32 = rng.gen_range(base_min..base_max);
    sec * 1000.0
}

fn pick_next_weather(current: WeatherKind, rng: &mut StdRng) -> WeatherKind {
    // Вероятности переходов зависят от текущей погоды
    // Значения — веса; нормализуем автоматически
    let (opts, weights): (&[WeatherKind], &[f32]) = match current {
        WeatherKind::Clear => (&[WeatherKind::Clear, WeatherKind::Rain, WeatherKind::Fog, WeatherKind::Snow], &[0.55, 0.25, 0.15, 0.05]),
        WeatherKind::Rain  => (&[WeatherKind::Clear, WeatherKind::Rain, WeatherKind::Fog, WeatherKind::Snow], &[0.35, 0.35, 0.20, 0.10]),
        WeatherKind::Fog   => (&[WeatherKind::Clear, WeatherKind::Rain, WeatherKind::Fog, WeatherKind::Snow], &[0.40, 0.20, 0.30, 0.10]),
        WeatherKind::Snow  => (&[WeatherKind::Clear, WeatherKind::Rain, WeatherKind::Fog, WeatherKind::Snow], &[0.30, 0.20, 0.10, 0.40]),
    };
    let total: f32 = weights.iter().copied().sum();
    let mut r = rng.gen_range(0.0..total);
    for (w, &p) in opts.iter().zip(weights.iter()) {
        if r < p { return *w; }
        r -= p;
    }
    *opts.last().unwrap_or(&current)
}

fn overlay_fog(frame: &mut [u8], fw: i32, fh: i32, t_ms: f32) {
    let t = t_ms / 1000.0;
    // два больших «слоя тумана» движутся в разные стороны, альфа суммируется
    let scale1 = 1.0 / 220.0;
    let scale2 = 1.0 / 140.0;
    let speed1 = 10.0; // px/сек
    let speed2 = -6.0;
    for y in 0..fh {
        let row = (y as usize) * (fw as usize) * 4;
        let fy = y as f32;
        for x in 0..fw {
            let fx = x as f32;
            let idx = row + (x as usize) * 4;
            // две синус-«волны» дают мягкое пятнистое поле 0..1
            let v1 = (fx * scale1 + t * speed1 * scale1).sin() * (fy * scale1 * 1.3).cos();
            let v2 = (fx * scale2 + t * speed2 * scale2 * 0.8).cos() * (fy * scale2 * 0.9 + 1.7).sin();
            let fog = ((v1 + v2) * 0.5 + 0.5).clamp(0.0, 1.0);
            // базовая серость тумана
            let r = 170u32; let g = 170u32; let b = 170u32;
            // сила тумана — нежная, 0..~30 альфы
            let a = (fog * 30.0) as u32;
            if a == 0 { continue; }
            let na = 255 - a;
            let dr = frame[idx] as u32; let dg = frame[idx+1] as u32; let db = frame[idx+2] as u32;
            frame[idx]   = ((a * r + na * dr) / 255) as u8;
            frame[idx+1] = ((a * g + na * dg) / 255) as u8;
            frame[idx+2] = ((a * b + na * db) / 255) as u8;
            frame[idx+3] = 255;
        }
    }
}

fn handle_console_command(cmd: &str, log: &mut Vec<String>, resources: &mut Resources, weather: &mut WeatherKind, world_clock_ms: &mut f32, world: &mut World, biome_overlay_debug: &mut bool) {
    let trimmed = cmd.trim();
    if trimmed.is_empty() { return; }
    let mut parts = trimmed.split_whitespace();
    let Some(head) = parts.next() else { return; };
    match head.to_ascii_lowercase().as_str() {
        "help" => {
            log.push("Commands: help, weather <clear|rain|fog|snow>, gold <±N>, set gold <N>, time <day|night|dawn|dusk|<0..1>>, biome <swamp_thr rocky_thr|overlay>, biome-overlay".to_string());
        }
        "weather" => {
            if let Some(arg) = parts.next() {
                let nw = match arg.to_ascii_lowercase().as_str() {
                    "clear" => Some(WeatherKind::Clear),
                    "rain" => Some(WeatherKind::Rain),
                    "fog" => Some(WeatherKind::Fog),
                    "snow" => Some(WeatherKind::Snow),
                    _ => None,
                };
                if let Some(w) = nw { *weather = w; log.push(format!("OK: weather set to {}", arg)); }
                else { log.push("ERR: usage weather <clear|rain|fog|snow>".to_string()); }
            } else { log.push("ERR: usage weather <clear|rain|fog|snow>".to_string()); }
        }
        "gold" => {
            if let Some(arg) = parts.next() {
                if let Ok(delta) = arg.parse::<i32>() { resources.gold = resources.gold.saturating_add(delta); log.push(format!("OK: gold += {} -> {}", delta, resources.gold)); }
                else { log.push("ERR: usage gold <±N>".to_string()); }
            } else { log.push("ERR: usage gold <±N>".to_string()); }
        }
        "set" => {
            let Some(what) = parts.next() else { log.push("ERR: usage set gold <N>".to_string()); return; };
            match what.to_ascii_lowercase().as_str() {
                "gold" => {
                    if let Some(arg) = parts.next() {
                        if let Ok(val) = arg.parse::<i32>() { resources.gold = val; log.push(format!("OK: gold = {}", resources.gold)); }
                        else { log.push("ERR: usage set gold <N>".to_string()); }
                    } else { log.push("ERR: usage set gold <N>".to_string()); }
                }
                _ => log.push("ERR: unknown 'set' target".to_string()),
            }
        }
        "time" => {
            if let Some(arg) = parts.next() {
                // day=~0.5, night=~0.0, dawn≈0.25, dusk≈0.75; также принимаем число 0..1
                let t_opt: Option<f32> = match arg.to_ascii_lowercase().as_str() {
                    "day" => Some(0.5),
                    "night" => Some(0.0),
                    "dawn" => Some(0.25),
                    "dusk" => Some(0.75),
                    other => other.parse::<f32>().ok().map(|v| v.clamp(0.0, 1.0)),
                };
                if let Some(v) = t_opt { *world_clock_ms = v * 120_000.0; log.push(format!("OK: time set to {:.2}", v)); }
                else { log.push("ERR: usage time <day|night|dawn|dusk|0..1>".to_string()); }
            } else { log.push("ERR: usage time <day|night|dawn|dusk|0..1>".to_string()); }
        }
        "biome" => {
            if let Some(arg) = parts.next() {
                if arg.eq_ignore_ascii_case("overlay") {
                    *biome_overlay_debug = !*biome_overlay_debug;
                    log.push(format!("OK: biome overlay {}", if *biome_overlay_debug {"ON"} else {"OFF"}));
                } else if let Some(arg2) = parts.next() {
                    if let (Ok(sw), Ok(rk)) = (arg.parse::<f32>(), arg2.parse::<f32>()) {
                        world.biome_swamp_thr = sw; world.biome_rocky_thr = rk;
                        world.biomes.clear(); // сбросим кэш
                        log.push(format!("OK: biome thresholds set swamp_thr={:.2} rocky_thr={:.2}", sw, rk));
                    } else { log.push("ERR: usage biome <swamp_thr rocky_thr|overlay>".to_string()); }
                } else { log.push("ERR: usage biome <swamp_thr rocky_thr|overlay>".to_string()); }
            } else { log.push("ERR: usage biome <swamp_thr rocky_thr|overlay>".to_string()); }
        }
        "biome_overlay" | "biomeoverlay" | "biome-overlay" => {
            *biome_overlay_debug = !*biome_overlay_debug;
            log.push(format!("OK: biome overlay {}", if *biome_overlay_debug {"ON"} else {"OFF"}));
        }
        _ => { log.push("ERR: unknown command. Type 'help'".to_string()); }
    }
}

fn screen_to_tile_px(mx: i32, my: i32, sw: i32, sh: i32, cam_px: Vec2, half_w: i32, half_h: i32, zoom: f32) -> Option<IVec2> {
    // экран -> мир (с учетом zoom и камеры)
    // GPU: world_x = (screen_x - sw/2) / zoom + cam_x
    //      world_y = -(screen_y - sh/2) / zoom - cam_y  (камера с +cam_y, view матрица)
    let wx = (mx - sw / 2) as f32 / zoom + cam_px.x;
    let wy = (my - sh / 2) as f32 / zoom + cam_px.y;
    
    let a = half_w as f32;
    let b = half_h as f32;
    // обратное к изометрической проекции: iso_x = (mx - my)*a, iso_y = (mx + my)*b
    let tx = 0.5 * (wy / b + wx / a) + 1.0;
    let ty = 0.5 * (wy / b - wx / a) + 1.0;
    let ix = tx.floor() as i32;
    let iy = ty.floor() as i32;
    Some(IVec2::new(ix, iy))
}

fn visible_tile_bounds_px(sw: i32, sh: i32, cam_px: Vec2, half_w: i32, half_h: i32, zoom: f32) -> (i32, i32, i32, i32) {
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
            if let Some(tp) = screen_to_tile_px(x, y, sw, sh, cam_px, half_w, half_h, zoom) {
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

