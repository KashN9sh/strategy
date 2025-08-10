use anyhow::Result;
use glam::{IVec2, Vec2};
mod types; use types::{TileKind, BuildingKind, Building, Resources, Citizen, Job, JobKind, LogItem, WarehouseStore, CitizenState, ResourceKind};
mod world; use world::World;
mod atlas; use atlas::{TileAtlas, BuildingAtlas};
mod render { pub mod tiles; }
mod ui;
mod input;
mod path;
mod jobs;
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
    trees: Vec<SaveTree>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct SaveBuilding { kind: BuildingKind, x: i32, y: i32, timer_ms: i32 }

#[derive(Serialize, Deserialize, Clone, Copy)]
struct SaveTree { x: i32, y: i32, stage: u8, age_ms: i32 }

impl SaveData {
    fn from_runtime(seed: u64, res: &Resources, buildings: &Vec<Building>, cam_px: Vec2, zoom: f32, world: &world::World) -> Self {
        let buildings = buildings
            .iter()
            .map(|b| SaveBuilding { kind: b.kind, x: b.pos.x, y: b.pos.y, timer_ms: b.timer_ms })
            .collect();
        let mut trees = Vec::new();
        for (&(x, y), tr) in world.trees.iter() { trees.push(SaveTree { x, y, stage: tr.stage, age_ms: tr.age_ms }); }
        SaveData { seed, resources: *res, buildings, cam_x: cam_px.x, cam_y: cam_px.y, zoom, trees }
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
    let mut ui_category: ui::UICategory = ui::UICategory::Forestry;
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
    let mut road_mode = false;
    let mut building_atlas: Option<BuildingAtlas> = None;
    let mut tree_atlas: Option<atlas::TreeAtlas> = None;
    // Попытаемся загрузить атлас из assets/tiles.png (ожидаем 6 тайлов в строку: grass, forest, water, clay, stone, iron)
    if let Ok(img) = image::open("assets/tiles.png") {
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
    let mut show_grid = false;
    let mut show_forest_overlay = false;
    let mut show_ui = true;
    let mut cursor_xy = IVec2::new(0, 0);
    let mut fps_ema: f32 = 60.0;
    let mut show_ui = true;

    let mut width_i32 = size.width as i32;
    let mut height_i32 = size.height as i32;
    // День/ночь
    const DAY_LENGTH_MS: f32 = 120_000.0;
    const START_HOUR: f32 = 8.0; // старт в 08:00
    let mut world_clock_ms: f32 = DAY_LENGTH_MS * (START_HOUR / 24.0);

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
                        if key == PhysicalKey::Code(input.reset_new_seed) { seed = rng.random(); world.reset_noise(seed); buildings.clear(); buildings_dirty = true; citizens.clear(); population = 0; resources = Resources { wood: 20, gold: 100, ..Default::default() }; }
                        if key == PhysicalKey::Code(input.reset_same_seed) { world.reset_noise(seed); buildings.clear(); buildings_dirty = true; citizens.clear(); population = 0; resources = Resources { wood: 20, gold: 100, ..Default::default() }; }
                        if key == PhysicalKey::Code(input.save_game) { let _ = save_game(&SaveData::from_runtime(seed, &resources, &buildings, cam_px, zoom, &world)); }
                        if key == PhysicalKey::Code(input.load_game) {
                            if let Ok(save) = load_game() {
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
                    // ЛКМ — попытка построить
                    if button == winit::event::MouseButton::Left {
                        if show_ui {
                            let ui_s = ui::ui_scale(height_i32, config.ui_scale_base);
                            let bar_h = ui::top_panel_height(ui_s);
                            // нижняя панель UI
                            let bottom_bar_h = ui::bottom_panel_height(ui_s);
                            let by0 = height_i32 - bottom_bar_h; let padb = 8 * ui_s; let btn_h = 18 * ui_s;
                            // клик по категориям
                            let mut cx = padb; let cy = by0 + padb;
                            let cats = [
                                (ui::UICategory::Housing, b"Housing".as_ref()),
                                (ui::UICategory::Storage, b"Storage".as_ref()),
                                (ui::UICategory::Forestry, b"Forestry".as_ref()),
                                (ui::UICategory::Mining, b"Mining".as_ref()),
                                (ui::UICategory::Food, b"Food".as_ref()),
                                (ui::UICategory::Logistics, b"Logistics".as_ref()),
                            ];
                            for (cat, label) in cats.iter() {
                                let bw = ui::button_w_for(label, ui_s);
                                if ui::point_in_rect(cursor_xy.x, cursor_xy.y, cx, cy, bw, btn_h) { ui_category = *cat; return; }
                                cx += bw + 6 * ui_s;
                            }
                            // клик по зданиям выбранной категории
                            let mut bx = padb; let by2 = cy + btn_h + 6 * ui_s;
                             let buildings_for_cat: &[BuildingKind] = match ui_category {
                                ui::UICategory::Housing => &[BuildingKind::House],
                                 ui::UICategory::Storage => &[BuildingKind::Warehouse],
                                 ui::UICategory::Forestry => &[BuildingKind::Lumberjack, BuildingKind::Forester],
                                 ui::UICategory::Mining => &[BuildingKind::StoneQuarry, BuildingKind::ClayPit, BuildingKind::IronMine, BuildingKind::Kiln],
                                ui::UICategory::Food => &[BuildingKind::WheatField, BuildingKind::Mill, BuildingKind::Bakery, BuildingKind::Fishery],
                                ui::UICategory::Logistics => &[],
                            };
                            for &bk in buildings_for_cat.iter() {
                            let label = match bk {
                                    BuildingKind::Lumberjack => b"Lumberjack".as_ref(),
                                    BuildingKind::House => b"House".as_ref(),
                                    BuildingKind::Warehouse => b"Warehouse".as_ref(),
                                    BuildingKind::Forester => b"Forester".as_ref(),
                                    BuildingKind::StoneQuarry => b"Quarry".as_ref(),
                                    BuildingKind::ClayPit => b"Clay Pit".as_ref(),
                                    BuildingKind::Kiln => b"Kiln".as_ref(),
                                    BuildingKind::WheatField => b"Wheat Field".as_ref(),
                                    BuildingKind::Mill => b"Mill".as_ref(),
                                    BuildingKind::Bakery => b"Bakery".as_ref(),
                                     BuildingKind::Fishery => b"Fishery".as_ref(),
                                     BuildingKind::IronMine => b"Iron Mine".as_ref(),
                                     BuildingKind::Smelter => b"Smelter".as_ref(),
                                };
                                let bw = ui::button_w_for(label, ui_s);
                                if bx + bw > width_i32 - padb { break; }
                                if ui::point_in_rect(cursor_xy.x, cursor_xy.y, bx, by2, bw, btn_h) { selected_building = bk; return; }
                                bx += bw + 6 * ui_s;
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
                            // дополнительные валидаторы по типу здания
                            let mut allowed = !world.is_occupied(tp) && tile_kind != TileKind::Water;
                            if allowed {
                                match selected_building {
                                    BuildingKind::Fishery => {
                                        // должен быть рядом с водой
                                        const NB: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
                                        allowed = NB.iter().any(|(dx,dy)| world.get_tile(tp.x+dx, tp.y+dy) == TileKind::Water);
                                    }
                                    BuildingKind::WheatField => {
                                        allowed = tile_kind == TileKind::Grass;
                                    }
                                    BuildingKind::StoneQuarry => {
                                        // Требуем spot камня
                                        allowed = world.has_stone_deposit(tp);
                                    }
                                    BuildingKind::ClayPit => {
                                        // Требуем spot глины
                                        allowed = world.has_clay_deposit(tp);
                                    }
                                    BuildingKind::IronMine => {
                                        // Требуем spot железа
                                        allowed = world.has_iron_deposit(tp);
                                    }
                                    _ => {}
                                }
                            }
                            if allowed {
                                let cost = building_cost(selected_building);
                                if resources.gold >= cost.gold && spend_wood(&mut warehouses, &mut resources, cost.wood) {
                                    resources.gold -= cost.gold;
                                    world.occupy(tp);
                                    buildings.push(Building { kind: selected_building, pos: tp, timer_ms: 0 });
                                    buildings_dirty = true;
                                     if selected_building == BuildingKind::House {
                                        // Простой спавн одного жителя на дом
                                        citizens.push(Citizen {
                                            pos: tp,
                                            target: tp,
                                            moving: false,
                                            progress: 0.0,
                                            carrying_log: false,
                                            assigned_job: None,
                                            deliver_to: tp,
                                            idle_timer_ms: 0,
                                            home: tp,
                                            workplace: None,
                                            state: CitizenState::Idle,
                                            work_timer_ms: 0,
                                            carrying: None,
                                            pending_input: None,
                                        });
                                        population += 1;
                                      } else if selected_building == BuildingKind::Warehouse {
                                          warehouses.push(WarehouseStore { pos: tp, wood: 0, stone: 0, clay: 0, bricks: 0, wheat: 0, flour: 0, bread: 0, fish: 0, gold: 0, iron_ore: 0, iron_ingots: 0 });
                                          // Переложим остатки ресурсов (кроме золота) в только что построенный склад
                                          if let Some(w) = warehouses.last_mut() {
                                              if resources.wood > 0 { w.wood += resources.wood; resources.wood = 0; }
                                              if resources.stone > 0 { w.stone += resources.stone; resources.stone = 0; }
                                              if resources.clay > 0 { w.clay += resources.clay; resources.clay = 0; }
                                              if resources.bricks > 0 { w.bricks += resources.bricks; resources.bricks = 0; }
                                              if resources.wheat > 0 { w.wheat += resources.wheat; resources.wheat = 0; }
                                              if resources.flour > 0 { w.flour += resources.flour; resources.flour = 0; }
                                              if resources.bread > 0 { w.bread += resources.bread; resources.bread = 0; }
                                               if resources.fish > 0 { w.fish += resources.fish; resources.fish = 0; }
                                               if resources.iron_ore > 0 { w.iron_ore += resources.iron_ore; resources.iron_ore = 0; }
                                              // золото хранится в казне (resources.gold)
                                          }
                                     } else if selected_building == BuildingKind::Forester {
                                         // просто добавим здание, логика в simulate()
                                    }
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

                              // деревья на лесных клетках как отдельные сущности
                              if world.has_tree(IVec2::new(mx, my)) {
                                  let stage = world.tree_stage(IVec2::new(mx, my)).unwrap_or(2) as usize;
                                  if let Some(ta) = &tree_atlas { if !ta.sprites.is_empty() {
                                      let idx = stage.min(ta.sprites.len()-1);
                                      // Масштаб спрайта дерева под текущий тайл
                                      let tile_w_px = atlas.half_w * 2 + 1;
                                      let scale = tile_w_px as f32 / ta.w as f32;
                                      let draw_w = (ta.w as f32 * scale).round() as i32;
                                      let draw_h = (ta.h as f32 * scale).round() as i32;
                                      let top_left_x = screen_pos.x - draw_w / 2;
                                      let top_left_y = screen_pos.y - atlas.half_h - draw_h + (atlas.half_h/2);
                                      render::tiles::blit_sprite_alpha_scaled(frame, width_i32, height_i32, top_left_x, top_left_y, &ta.sprites[idx], ta.w, ta.h, draw_w, draw_h);
                                  } else { render::tiles::draw_tree(frame, width_i32, height_i32, screen_pos.x, screen_pos.y - atlas.half_h, atlas.half_w, atlas.half_h, stage as u8); }}
                                  else { render::tiles::draw_tree(frame, width_i32, height_i32, screen_pos.x, screen_pos.y - atlas.half_h, atlas.half_w, atlas.half_h, stage as u8); }
                              }
                              // оверлеи месторождений (простая заливка-рамка)
                              let tp = IVec2::new(mx, my);
                              if world.get_tile(mx, my) != TileKind::Water {
                                  // если загружен базовый атлас — используем спрайты месторождений поверх травы
                                  if atlas.base_loaded {
                                      let tile_w_px = atlas.half_w * 2 + 1;
                                      let tile_h_px = atlas.half_h * 2 + 1;
                                      let top_left_x = screen_pos.x - atlas.half_w;
                                      let top_left_y = screen_pos.y - atlas.half_h;
                                       if world.has_clay_deposit(tp) {
                                           render::tiles::blit_sprite_alpha_noscale_tinted(frame, width_i32, height_i32, top_left_x, top_left_y, &atlas.clay, tile_w_px, tile_h_px, 170);
                                       }
                                       if world.has_stone_deposit(tp) {
                                           render::tiles::blit_sprite_alpha_noscale_tinted(frame, width_i32, height_i32, top_left_x, top_left_y, &atlas.stone, tile_w_px, tile_h_px, 170);
                                       }
                                       if world.has_iron_deposit(tp) {
                                           render::tiles::blit_sprite_alpha_noscale_tinted(frame, width_i32, height_i32, top_left_x, top_left_y, &atlas.iron, tile_w_px, tile_h_px, 190);
                                       }
                                  } else {
                                      if world.has_clay_deposit(tp) { render::tiles::draw_iso_tile_tinted(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [200, 120, 80, 120]); }
                                      if world.has_stone_deposit(tp) { render::tiles::draw_iso_tile_tinted(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [200, 200, 200, 120]); }
                                      if world.has_iron_deposit(tp) { render::tiles::draw_iso_tile_tinted(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [150, 140, 220, 140]); }
                                  }
                              }
                        }
                    }

                    // Поленья на земле
                    for li in &logs_on_ground {
                        if li.carried { continue; }
                        let mx = li.pos.x; let my = li.pos.y;
                        if mx < min_tx || my < min_ty || mx > max_tx || my > max_ty { continue; }
                        let world_x = (mx - my) * atlas.half_w - cam_snap.x as i32;
                        let world_y = (mx + my) * atlas.half_h - cam_snap.y as i32;
                        let screen_pos = screen_center + IVec2::new(world_x, world_y);
                        render::tiles::draw_log(frame, width_i32, height_i32, screen_pos.x, screen_pos.y - atlas.half_h/4, atlas.half_w, atlas.half_h);
                    }

                    // Подсветка ховера
                    if let Some(tp) = hovered_tile {
                        let world_x = (tp.x - tp.y) * atlas.half_w - cam_snap.x as i32;
                        let world_y = (tp.x + tp.y) * atlas.half_h - cam_snap.y as i32;
                        let screen_pos = screen_center + IVec2::new(world_x, world_y);
                        render::tiles::draw_iso_outline(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [240, 230, 80, 255]);
                    }

                    // Отрисуем здания по глубине
                    if buildings_dirty {
                        buildings.sort_by_key(|b| b.pos.x + b.pos.y);
                        buildings_dirty = false;
                    }
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
                                    let top_left_y = screen_pos.y + atlas.half_h - draw_h;
                                    render::tiles::blit_sprite_alpha_scaled(frame, width_i32, height_i32, top_left_x, top_left_y, &ba.sprites[idx], ba.w, ba.h, draw_w, draw_h);
                                    continue;
                                }
                            }
                        }
                        let color = match b.kind {
                            BuildingKind::Lumberjack => [140, 90, 40, 255],
                            BuildingKind::House => [180, 180, 180, 255],
                            BuildingKind::Warehouse => [150, 120, 80, 255],
                            BuildingKind::Forester => [90, 140, 90, 255],
                            BuildingKind::StoneQuarry => [120, 120, 120, 255],
                            BuildingKind::ClayPit => [150, 90, 70, 255],
                            BuildingKind::Kiln => [160, 60, 40, 255],
                            BuildingKind::WheatField => [200, 180, 80, 255],
                            BuildingKind::Mill => [210, 210, 180, 255],
                            BuildingKind::Bakery => [200, 160, 120, 255],
                            BuildingKind::Fishery => [100, 140, 200, 255],
                            BuildingKind::IronMine => [90, 90, 110, 255],
                            BuildingKind::Smelter => [190, 190, 210, 255],
                        };
                        render::tiles::draw_building(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, color);
                    }

                    // Рендер жителей (кружок) с интерполяцией между клетками
                    for c in &citizens {
                        let (fx, fy) = if c.moving {
                            let dx = (c.target.x - c.pos.x) as f32;
                            let dy = (c.target.y - c.pos.y) as f32;
                            (c.pos.x as f32 + dx * c.progress, c.pos.y as f32 + dy * c.progress)
                        } else { (c.pos.x as f32, c.pos.y as f32) };
                        let world_x = ((fx - fy) * atlas.half_w as f32).round() as i32 - cam_snap.x as i32;
                        let world_y = ((fx + fy) * atlas.half_h as f32).round() as i32 - cam_snap.y as i32;
                        let screen_pos = screen_center + IVec2::new(world_x, world_y);
                        let r = (atlas.half_w as f32 * 0.15).round() as i32; // масштаб от тайла
                        render::tiles::draw_citizen_marker(frame, width_i32, height_i32, screen_pos.x, screen_pos.y - atlas.half_h/3, r.max(2), [255, 230, 120, 255]);
                    }

                    // Оверлей день/ночь
                    let t = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
                    let angle = t * std::f32::consts::TAU;
                    let daylight = 0.5 - 0.5 * angle.cos();
                    let darkness = (1.0 - daylight).max(0.0);
                    let night_strength = (darkness.powf(1.4) * 180.0).min(200.0) as u8;
                    if night_strength > 0 { overlay_tint(frame, width_i32, height_i32, [18, 28, 60, night_strength]); }

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
                        ui::draw_ui(frame, width_i32, height_i32, &visible, total_visible_wood, population, selected_building, fps_ema, speed_mult, paused, config.ui_scale_base, ui_category, day_progress, idle, working, sleeping, hauling, fetching, cursor_xy.x, cursor_xy.y);
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
                        simulate(&mut buildings, &mut world, &mut resources, step_ms as i32);
                        world.grow_trees(step_ms as i32);
                        world_clock_ms = (world_clock_ms + step_ms) % DAY_LENGTH_MS;

                        // День/ночь
                        let t = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
                        let angle = t * std::f32::consts::TAU;
                        let daylight = 0.5 - 0.5 * angle.cos();
                        let is_day = daylight > 0.25; // простой порог

                        // Ночная рутина: идём домой и спим, сбрасываем работу
                        if !is_day {
                            for c in citizens.iter_mut() {
                                // отменяем активную задачу/перенос
                                c.assigned_job = None;
                                c.carrying_log = false;
                                if c.state != CitizenState::Sleeping {
                                    if c.pos != c.home && !c.moving {
                                        c.target = c.home;
                                        c.moving = true;
                                        c.progress = 0.0;
                                        c.state = CitizenState::GoingHome;
                                    }
                                    if !c.moving && c.pos == c.home {
                                        c.state = CitizenState::Sleeping;
                                        c.workplace = None;
                                        c.work_timer_ms = 0;
                                        c.carrying = None;
                                    }
                                }
                            }
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
                        }

                        // Дневная рутина рабочих по зданиям (кроме лесоруба/склада/дома)
                        if is_day {
                            use std::collections::HashSet;
                            let mut occupied: HashSet<(i32,i32)> = citizens.iter()
                                .filter_map(|c| c.workplace.map(|p| (p.x, p.y)))
                                .collect();
                            // Назначение рабочих
                            for b in buildings.iter() {
                                match b.kind {
                                    BuildingKind::House | BuildingKind::Warehouse | BuildingKind::Lumberjack => {}
                                    _ => {
                                        if !occupied.contains(&(b.pos.x, b.pos.y)) {
                                            if let Some((ci, _)) = citizens.iter()
                                                .enumerate()
                                                .filter(|(_, c)| matches!(c.state, CitizenState::Idle | CitizenState::Sleeping) && !c.moving)
                                                .min_by_key(|(_, c)| (c.pos.x - b.pos.x).abs() + (c.pos.y - b.pos.y).abs())
                                            {
                                                let c = &mut citizens[ci];
                                                if matches!(c.state, CitizenState::Sleeping) && c.pos != c.home { continue; }
                                                c.workplace = Some(b.pos);
                                                c.target = b.pos;
                                                c.moving = true;
                                                c.progress = 0.0;
                                                c.state = CitizenState::GoingToWork;
                                                occupied.insert((b.pos.x, b.pos.y));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 1) генерация задач лесорубками — только если нет активных задач/носильщиков для этого места
                        if is_day {
                            for b in buildings.iter() {
                                if b.kind == BuildingKind::Lumberjack {
                                    // ищем ближайшее дерево в радиусе R и публикуем задачу, если её ещё нет
                                    const R: i32 = 16;
                                    let mut best: Option<(i32, IVec2)> = None;
                                    for dy in -R..=R { for dx in -R..=R {
                                        let np = IVec2::new(b.pos.x + dx, b.pos.y + dy);
                                        if matches!(world.tree_stage(np), Some(2)) {
                                            let d = dx.abs() + dy.abs();
                                            if best.map(|(bd, _)| d < bd).unwrap_or(true) { best = Some((d, np)); }
                                        }
                                    }}
                                    if let Some((_, np)) = best {
                                        let already = jobs.iter().any(|j| match j.kind { JobKind::ChopWood { pos } => pos==np, JobKind::HaulWood { from, .. } => from==np });
                                        if !already {
                                            jobs.push(Job { id: { let id=next_job_id; next_job_id+=1; id }, kind: JobKind::ChopWood { pos: np }, taken: false, done: false });
                                        }
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
                            jobs::assign_jobs_nearest_worker(&mut citizens, &mut jobs);
                            // 3) выполнение задач
                            jobs::process_jobs(&mut citizens, &mut jobs, &mut logs_on_ground, &mut warehouses, &mut resources, &buildings, &mut world, &mut next_job_id);
                        }
                        // простая симуляция жителей
                        for c in citizens.iter_mut() {
                            if !c.moving {
                                c.idle_timer_ms += step_ms as i32;
                                c.progress = 0.0;
                                // если стоим > 5 секунд с назначенной задачей — сбросим её
                                if c.idle_timer_ms > 5000 { c.assigned_job = None; c.carrying_log = false; c.idle_timer_ms = 0; }
                                // смена состояний при прибытии
                                match c.state {
                                    CitizenState::GoingToWork => { c.state = CitizenState::Working; }
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
                                            if let Some(wp) = c.workplace { c.target = wp; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToWork; }
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
                                                    if let Some(wp) = c.workplace { c.target = wp; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToWork; }
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
                                c.progress += (step_ms / 300.0) as f32;
                                if c.progress >= 1.0 { c.pos = c.target; c.moving = false; c.progress = 0.0; }
                            }
                        }

                        // Производство при работе у здания
                        if is_day {
                            for c in citizens.iter_mut() {
                                if !matches!(c.state, CitizenState::Working) { continue; }
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
                                                    c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToDeposit;
                                                }
                                            }
                                        }
                                        BuildingKind::ClayPit => {
                                            if c.carrying.is_none() && c.work_timer_ms >= 4000 {
                                                c.work_timer_ms = 0;
                                                if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                    c.carrying = Some((ResourceKind::Clay, 1));
                                                    c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToDeposit;
                                                }
                                            }
                                        }
                                        BuildingKind::IronMine => {
                                            if c.carrying.is_none() && c.work_timer_ms >= 5000 {
                                                c.work_timer_ms = 0;
                                                if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                    c.carrying = Some((ResourceKind::IronOre, 1));
                                                    c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToDeposit;
                                                }
                                            }
                                        }
                                        BuildingKind::WheatField => {
                                            if c.carrying.is_none() && c.work_timer_ms >= 6000 {
                                                c.work_timer_ms = 0;
                                                if let Some(dst) = warehouses.iter().min_by_key(|w| (w.pos.x - b.pos.x).abs() + (w.pos.y - b.pos.y).abs()).map(|w| w.pos) {
                                                    c.carrying = Some((ResourceKind::Wheat, 1));
                                                    c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToDeposit;
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
                                                        c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToDeposit;
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
                                                            c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToDeposit;
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
                                                            c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToDeposit;
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
                                                            c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToDeposit;
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
                                                        c.target = dst; c.moving = true; c.progress = 0.0; c.state = CitizenState::GoingToDeposit;
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
                                                    if tk != TileKind::Water && !world.has_tree(p) && !world.is_occupied(p) && !world.is_road(p) {
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

fn sat_mul_add(a: i32, b: i32, c: i32) -> i32 {
    let v = (a as i64) * (b as i64) + (c as i64);
    if v > i32::MAX as i64 { i32::MAX } else if v < i32::MIN as i64 { i32::MIN } else { v as i32 }
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
        BuildingKind::Warehouse => Some(2),
        BuildingKind::Forester => Some(3),
        BuildingKind::StoneQuarry => Some(4),
        BuildingKind::ClayPit => Some(5),
        BuildingKind::Kiln => Some(6),
        BuildingKind::WheatField => Some(7),
        BuildingKind::Mill => Some(8),
        BuildingKind::Bakery => Some(9),
        BuildingKind::Fishery => Some(10),
        BuildingKind::IronMine => Some(11),
        BuildingKind::Smelter => Some(12),
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
    // немного запаса вокруг экрана
    (min_tx - 2, min_ty - 2, max_tx + 2, max_ty + 2)
}

fn building_cost(kind: BuildingKind) -> Resources {
    match kind {
        BuildingKind::Lumberjack => Resources { wood: 5, gold: 10, ..Default::default() },
        BuildingKind::House => Resources { wood: 10, gold: 15, ..Default::default() },
        BuildingKind::Warehouse => Resources { wood: 20, gold: 30, ..Default::default() },
        BuildingKind::Forester => Resources { wood: 15, gold: 20, ..Default::default() },
        BuildingKind::StoneQuarry => Resources { wood: 10, gold: 10, ..Default::default() },
        BuildingKind::ClayPit => Resources { wood: 10, gold: 10, ..Default::default() },
        BuildingKind::Kiln => Resources { wood: 15, gold: 15, ..Default::default() },
        BuildingKind::WheatField => Resources { wood: 5, gold: 5, ..Default::default() },
        BuildingKind::Mill => Resources { wood: 20, gold: 20, ..Default::default() },
        BuildingKind::Bakery => Resources { wood: 20, gold: 25, ..Default::default() },
        BuildingKind::Fishery => Resources { wood: 15, gold: 10, ..Default::default() },
        BuildingKind::IronMine => Resources { wood: 15, gold: 20, ..Default::default() },
        BuildingKind::Smelter => Resources { wood: 20, gold: 25, ..Default::default() },
    }
}

fn warehouses_total_wood(warehouses: &Vec<WarehouseStore>) -> i32 { warehouses.iter().map(|w| w.wood).sum() }

fn spend_wood(warehouses: &mut Vec<WarehouseStore>, resources: &mut Resources, mut amount: i32) -> bool {
    if amount <= 0 { return true; }
    if warehouses.is_empty() {
        if resources.wood >= amount { resources.wood -= amount; return true; } else { return false; }
    }
    let total = warehouses_total_wood(warehouses);
    if total < amount { return false; }
    for w in warehouses.iter_mut() {
        if amount == 0 { break; }
        let take = amount.min(w.wood);
        w.wood -= take;
        amount -= take;
    }
    true
}

fn simulate(buildings: &mut Vec<Building>, world: &mut World, resources: &mut Resources, dt_ms: i32) {
    for b in buildings.iter_mut() {
        b.timer_ms += dt_ms;
        match b.kind {
            BuildingKind::Lumberjack => { /* Производство выполняют жители через систему задач */ }
            BuildingKind::House => {
                // каждые 5с: потребляет еду (хлеб или рыбу) и даёт золото
                if b.timer_ms >= 5000 {
                    b.timer_ms = 0;
                    if resources.bread > 0 { resources.bread -= 1; resources.gold += 1; }
                    else if resources.fish > 0 { resources.fish -= 1; resources.gold += 1; }
                }
            }
            BuildingKind::Warehouse => { /* пока пассивный */ }
            BuildingKind::Forester => { /* Производство выполняют жители */ }
            BuildingKind::StoneQuarry => { /* Производство выполняют жители */ }
            BuildingKind::ClayPit => { /* Производство выполняют жители */ }
            BuildingKind::Kiln => { /* Производство выполняют жители */ }
            BuildingKind::WheatField => { /* Производство выполняют жители */ }
            BuildingKind::Mill => { /* Производство выполняют жители */ }
            BuildingKind::Bakery => { /* Производство выполняют жители */ }
            BuildingKind::Fishery => { /* Производство выполняют жители */ }
            BuildingKind::IronMine => { /* Производство выполняют жители */ }
            BuildingKind::Smelter => { /* Производство выполняют жители */ }
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

