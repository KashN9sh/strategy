use anyhow::Result;
use glam::{IVec2, Vec2};
use pixels::{Pixels, SurfaceTexture};
use std::time::Instant;
use noise::{NoiseFn, Fbm, Seedable, MultiFractal};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

const TILE_W: i32 = 32; // ширина ромба в пикселях
const TILE_H: i32 = 16; // высота ромба в пикселях
// Размер тайла в пикселях задаётся через атлас (half_w/half_h)
// Размер чанка в тайлах
const CHUNK_W: i32 = 32;
const CHUNK_H: i32 = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TileKind {
    Grass,
    Forest,
    Water,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum BuildingKind {
    Lumberjack,
    House,
}

#[derive(Clone, Debug)]
struct Building {
    kind: BuildingKind,
    pos: IVec2, // координаты тайла
    timer_ms: i32,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
struct Resources {
    wood: i32,
    gold: i32,
}

struct TileAtlas {
    zoom_px: i32,
    half_w: i32,
    half_h: i32,
    grass: Vec<u8>,
    forest: Vec<u8>,
    water: Vec<u8>,
}

impl TileAtlas {
    fn new() -> Self {
        Self { zoom_px: -1, half_w: 0, half_h: 0, grass: Vec::new(), forest: Vec::new(), water: Vec::new() }
    }

    fn ensure_zoom(&mut self, zoom: f32) {
        let half_w = ((TILE_W as f32 / 2.0) * zoom).round() as i32;
        let half_h = ((TILE_H as f32 / 2.0) * zoom).round() as i32;
        let zoom_px = half_w.max(1) * 2 + 1; // используем ширину как признак
        if zoom_px == self.zoom_px && self.half_w == half_w && self.half_h == half_h { return; }
        self.zoom_px = zoom_px;
        self.half_w = half_w.max(1);
        self.half_h = half_h.max(1);
        self.grass = Self::build_tile(self.half_w, self.half_h, [40, 120, 80, 255]);
        self.forest = Self::build_tile(self.half_w, self.half_h, [26, 100, 60, 255]);
        self.water = Self::build_tile(self.half_w, self.half_h, [28, 64, 120, 255]);
    }

    fn build_tile(half_w: i32, half_h: i32, color: [u8; 4]) -> Vec<u8> {
        let w = half_w * 2 + 1;
        let h = half_h * 2 + 1;
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for dy in -half_h..=half_h {
            let t = dy.abs() as f32 / half_h.max(1) as f32;
            let row_half = ((1.0 - t) * half_w as f32).round() as i32;
            let y = dy + half_h;
            let x0 = half_w - row_half;
            let x1 = half_w + row_half;
            let base = (y as usize) * (w as usize) * 4;
            for x in x0..=x1 {
                let idx = base + (x as usize) * 4;
                buf[idx..idx + 4].copy_from_slice(&color);
            }
        }
        buf
    }

    fn blit(&self, frame: &mut [u8], fw: i32, fh: i32, cx: i32, cy: i32, kind: TileKind) {
        let src = match kind { TileKind::Grass => &self.grass, TileKind::Forest => &self.forest, TileKind::Water => &self.water };
        let w = self.half_w * 2 + 1;
        let h = self.half_h * 2 + 1;
        let x0 = cx - self.half_w;
        let y0 = cy - self.half_h;

        // вычислим пересечение с экраном (по строкам)
        let dst_y_start = y0.max(0);
        let dst_y_end = (y0 + h).min(fh);
        if dst_y_start >= dst_y_end { return; }

        for dy in dst_y_start..dst_y_end {
            let sy = dy - y0; // строка в источнике [0..h)
            let from_center = sy - self.half_h;
            let t = (from_center.unsigned_abs() as f32) / (self.half_h.max(1) as f32);
            let row_half = ((1.0 - t) * self.half_w as f32).round() as i32;
            let src_x0 = self.half_w - row_half; // включительно
            let src_x1 = self.half_w + row_half + 1; // эксклюзивно

            // соответствующие x в целевом буфере
            let row_dst_x0 = x0 + src_x0;
            let row_dst_x1 = x0 + src_x1;

            // пересечение с экраном
            let dst_x0 = row_dst_x0.max(0);
            let dst_x1 = row_dst_x1.min(fw);
            if dst_x0 >= dst_x1 { continue; }

            // смещение в источнике с учётом обрезки
            let cut_left = dst_x0 - row_dst_x0;
            let src_copy_x0 = src_x0 + cut_left;
            let src_row = (sy as usize) * (w as usize) * 4;
            let src_slice = &src[(src_row + (src_copy_x0 as usize) * 4)..(src_row + ((src_copy_x0 + (dst_x1 - dst_x0)) as usize) * 4)];

            // копирование
            let dst_row = (dy as usize) * (fw as usize) * 4;
            let dst_slice = &mut frame[(dst_row + (dst_x0 as usize) * 4)..(dst_row + (dst_x1 as usize) * 4)];
            dst_slice.copy_from_slice(src_slice);
        }
    }
}

#[derive(Clone)]
struct Chunk {
    tiles: Vec<TileKind>, // размер CHUNK_W * CHUNK_H
}

struct World {
    seed: u64,
    fbm: Fbm<noise::OpenSimplex>,
    chunks: HashMap<(i32, i32), Chunk>,
    occupied: HashSet<(i32, i32)>,
    tx: Sender<(i32, i32)>,
    rx: Receiver<ChunkResult>,
    pending: HashSet<(i32, i32)>,
    max_chunks: usize,
}

impl World {
    fn new(seed: u64) -> Self {
        let mut fbm = Fbm::<noise::OpenSimplex>::new(0);
        fbm = fbm
            .set_seed(seed as u32)
            .set_octaves(5)
            .set_frequency(0.03)
            .set_lacunarity(2.0)
            .set_persistence(0.5);
        let (tx, rx) = spawn_chunk_worker(seed);
        Self { seed, fbm, chunks: HashMap::new(), occupied: HashSet::new(), tx, rx, pending: HashSet::new(), max_chunks: 512 }
    }

    fn reset(&mut self, seed: u64) {
        self.seed = seed;
        let mut fbm = Fbm::<noise::OpenSimplex>::new(0);
        fbm = fbm
            .set_seed(seed as u32)
            .set_octaves(5)
            .set_frequency(0.03)
            .set_lacunarity(2.0)
            .set_persistence(0.5);
        self.fbm = fbm;
        self.chunks.clear();
        self.occupied.clear();
        let (tx, rx) = spawn_chunk_worker(seed);
        self.tx = tx;
        self.rx = rx;
        self.pending.clear();
    }

    fn get_tile(&mut self, tx: i32, ty: i32) -> TileKind {
        let cx = tx.div_euclid(CHUNK_W);
        let cy = ty.div_euclid(CHUNK_H);
        let lx = tx.rem_euclid(CHUNK_W);
        let ly = ty.rem_euclid(CHUNK_H);
        if !self.chunks.contains_key(&(cx, cy)) && !self.pending.contains(&(cx, cy)) {
            let _ = self.tx.send((cx, cy));
            self.pending.insert((cx, cy));
        }
        if let Some(chunk) = self.chunks.get(&(cx, cy)) {
            return chunk.tiles[(ly * CHUNK_W + lx) as usize];
        }
        // быстрый прогноз по шуму (без ожидания чанка)
        self.tile_by_noise(tx, ty)
    }

    fn is_occupied(&self, tx: i32, ty: i32) -> bool { self.occupied.contains(&(tx, ty)) }
    fn occupy(&mut self, tx: i32, ty: i32) { self.occupied.insert((tx, ty)); }

    fn tile_by_noise(&self, tx: i32, ty: i32) -> TileKind {
        let n = self.fbm.get([tx as f64, ty as f64]) as f32;
        let h = n;
        if h < -0.2 { TileKind::Water } else if h < 0.2 { TileKind::Grass } else { TileKind::Forest }
    }

    fn integrate_ready_chunks(&mut self) {
        for res in self.rx.try_iter() {
            self.chunks.insert((res.cx, res.cy), Chunk { tiles: res.tiles });
            self.pending.remove(&(res.cx, res.cy));
        }
        // примитивная выгрузка LRU по расстоянию (ограничим общее кол-во)
        if self.chunks.len() > self.max_chunks {
            // без позиции камеры здесь оставим на будущее; пока просто ограничим, удаляя произвольно
            while self.chunks.len() > self.max_chunks {
                if let Some((&key, _)) = self.chunks.iter().next() {
                    self.chunks.remove(&key);
                } else {
                    break;
                }
            }
        }
    }

    fn schedule_ring(&mut self, min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32) {
        // диапазон тайлов -> диапазон чанков + 1 в запас
        let cmin_x = (min_tx.div_euclid(CHUNK_W)) - 1;
        let cmin_y = (min_ty.div_euclid(CHUNK_H)) - 1;
        let cmax_x = (max_tx.div_euclid(CHUNK_W)) + 1;
        let cmax_y = (max_ty.div_euclid(CHUNK_H)) + 1;
        for cy in cmin_y..=cmax_y {
            for cx in cmin_x..=cmax_x {
                if !self.chunks.contains_key(&(cx, cy)) && !self.pending.contains(&(cx, cy)) {
                    let _ = self.tx.send((cx, cy));
                    self.pending.insert((cx, cy));
                }
            }
        }
    }
}

struct ChunkResult { cx: i32, cy: i32, tiles: Vec<TileKind> }

fn spawn_chunk_worker(seed: u64) -> (Sender<(i32, i32)>, Receiver<ChunkResult>) {
    let (tx, rx_req) = channel::<(i32, i32)>();
    let (tx_res, rx_res) = channel::<ChunkResult>();
    thread::spawn(move || {
        let mut fbm = Fbm::<noise::OpenSimplex>::new(0);
        fbm = fbm
            .set_seed(seed as u32)
            .set_octaves(5)
            .set_frequency(0.03)
            .set_lacunarity(2.0)
            .set_persistence(0.5);
        while let Ok((cx, cy)) = rx_req.recv() {
            let mut tiles = vec![TileKind::Water; (CHUNK_W * CHUNK_H) as usize];
            for ly in 0..CHUNK_H {
                for lx in 0..CHUNK_W {
                    let tx = cx * CHUNK_W + lx;
                    let ty = cy * CHUNK_H + ly;
                    let n = fbm.get([tx as f64, ty as f64]) as f32;
                    let h = n;
                    tiles[(ly * CHUNK_W + lx) as usize] = if h < -0.2 { TileKind::Water } else if h < 0.2 { TileKind::Grass } else { TileKind::Forest };
                }
            }
            let _ = tx_res.send(ChunkResult { cx, cy, tiles });
        }
    });
    (tx, rx_res)
}

// --------------- Config / Save ---------------

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    base_step_ms: f32,
}

#[derive(Serialize, Deserialize, Clone)]
struct InputConfig {
    move_up: String,
    move_down: String,
    move_left: String,
    move_right: String,
    zoom_in: String,
    zoom_out: String,
    toggle_pause: String,
    speed_0_5x: String,
    speed_1x: String,
    speed_2x: String,
    speed_3x: String,
    build_lumberjack: String,
    build_house: String,
    reset_new_seed: String,
    reset_same_seed: String,
    save_game: String,
    load_game: String,
}

fn code_from_str(s: &str) -> KeyCode {
    use KeyCode::*;
    match s.to_uppercase().as_str() {
        "W" => KeyW,
        "A" => KeyA,
        "S" => KeyS,
        "D" => KeyD,
        "Q" => KeyQ,
        "E" => KeyE,
        "SPACE" => Space,
        "DIGIT1" | "1" => Digit1,
        "DIGIT2" | "2" => Digit2,
        "DIGIT3" | "3" => Digit3,
        "DIGIT4" | "4" => Digit4,
        "Z" => KeyZ,
        "X" => KeyX,
        "R" => KeyR,
        "N" => KeyN,
        "F5" => F5,
        "F9" => F9,
        _ => KeyCode::Escape,
    }
}

fn load_or_create_config(path: &str) -> Result<(Config, InputConfig)> {
    if Path::new(path).exists() {
        let data = fs::read_to_string(path)?;
        #[derive(Deserialize)]
        struct FileCfg { config: Config, input: InputConfig }
        let parsed: FileCfg = toml::from_str(&data)?;
        Ok((parsed.config, parsed.input))
    } else {
        let config = Config { base_step_ms: 33.0 };
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
            reset_new_seed: "R".into(),
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

struct ResolvedInput {
    move_up: KeyCode,
    move_down: KeyCode,
    move_left: KeyCode,
    move_right: KeyCode,
    zoom_in: KeyCode,
    zoom_out: KeyCode,
    toggle_pause: KeyCode,
    speed_0_5x: KeyCode,
    speed_1x: KeyCode,
    speed_2x: KeyCode,
    speed_3x: KeyCode,
    build_lumberjack: KeyCode,
    build_house: KeyCode,
    reset_new_seed: KeyCode,
    reset_same_seed: KeyCode,
    save_game: KeyCode,
    load_game: KeyCode,
}

impl ResolvedInput {
    fn from(cfg: &InputConfig) -> Self {
        Self {
            move_up: code_from_str(&cfg.move_up),
            move_down: code_from_str(&cfg.move_down),
            move_left: code_from_str(&cfg.move_left),
            move_right: code_from_str(&cfg.move_right),
            zoom_in: code_from_str(&cfg.zoom_in),
            zoom_out: code_from_str(&cfg.zoom_out),
            toggle_pause: code_from_str(&cfg.toggle_pause),
            speed_0_5x: code_from_str(&cfg.speed_0_5x),
            speed_1x: code_from_str(&cfg.speed_1x),
            speed_2x: code_from_str(&cfg.speed_2x),
            speed_3x: code_from_str(&cfg.speed_3x),
            build_lumberjack: code_from_str(&cfg.build_lumberjack),
            build_house: code_from_str(&cfg.build_house),
            reset_new_seed: code_from_str(&cfg.reset_new_seed),
            reset_same_seed: code_from_str(&cfg.reset_same_seed),
            save_game: code_from_str(&cfg.save_game),
            load_game: code_from_str(&cfg.load_game),
        }
    }
}

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
                        if key == PhysicalKey::Code(input.build_lumberjack) { selected_building = BuildingKind::Lumberjack; }
                        if key == PhysicalKey::Code(input.build_house) { selected_building = BuildingKind::House; }
                        if key == PhysicalKey::Code(input.reset_new_seed) { seed = rng.random(); world.reset(seed); buildings.clear(); resources = Resources { wood: 20, gold: 100 }; }
                        if key == PhysicalKey::Code(input.reset_same_seed) { world.reset(seed); buildings.clear(); resources = Resources { wood: 20, gold: 100 }; }
                        if key == PhysicalKey::Code(input.save_game) { let _ = save_game(&SaveData::from_runtime(seed, &resources, &buildings, cam_px, zoom)); }
                        if key == PhysicalKey::Code(input.load_game) {
                            if let Ok(save) = load_game() {
                                seed = save.seed;
                                world.reset(seed);
                                buildings = save.to_buildings();
                                resources = save.resources;
                                cam_px = Vec2::new(save.cam_x, save.cam_y);
                                zoom = save.zoom;
                                // восстановим отметку occupied
                                world.occupied.clear();
                                for b in &buildings { world.occupy(b.pos.x, b.pos.y); }
                            }
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let mx = position.x as i32;
                    let my = position.y as i32;
                    hovered_tile = screen_to_tile_px(mx, my, width_i32, height_i32, cam_px, atlas.half_w, atlas.half_h);
                }
                WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                    // ЛКМ — попытка построить
                    if button == winit::event::MouseButton::Left {
                        if let Some(tp) = hovered_tile {
                            let tile_kind = world.get_tile(tp.x, tp.y);
                            if !world.is_occupied(tp.x, tp.y) && tile_kind != TileKind::Water {
                                let cost = building_cost(selected_building);
                                if resources.wood >= cost.wood && resources.gold >= cost.gold {
                                    resources.wood -= cost.wood;
                                    resources.gold -= cost.gold;
                                    world.occupy(tp.x, tp.y);
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
                    for my in min_ty..=max_ty {
                        for mx in min_tx..=max_tx {
                            let kind = world.get_tile(mx, my);
                            let world_x = (mx - my) * atlas.half_w - cam_px.x.round() as i32;
                            let world_y = (mx + my) * atlas.half_h - cam_px.y.round() as i32;
                            let screen_pos = screen_center + IVec2::new(world_x, world_y);
                            atlas.blit(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, kind);
                        }
                    }

                    // Подсветка ховера
                    if let Some(tp) = hovered_tile {
                        let world_x = (tp.x - tp.y) * atlas.half_w - cam_px.x.round() as i32;
                        let world_y = (tp.x + tp.y) * atlas.half_h - cam_px.y.round() as i32;
                        let screen_pos = screen_center + IVec2::new(world_x, world_y);
                        draw_iso_outline(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [240, 230, 80, 255]);
                    }

                    // Отрисуем здания по глубине
                    buildings.sort_by_key(|b| b.pos.x + b.pos.y);
                    for b in buildings.iter() {
                        let mx = b.pos.x;
                        let my = b.pos.y;
                        if mx < min_tx || my < min_ty || mx > max_tx || my > max_ty { continue; }
                        let world_x = (mx - my) * atlas.half_w - cam_px.x.round() as i32;
                        let world_y = (mx + my) * atlas.half_h - cam_px.y.round() as i32;
                        let screen_pos = screen_center + IVec2::new(world_x, world_y);
                        let color = match b.kind { BuildingKind::Lumberjack => [140, 90, 40, 255], BuildingKind::House => [180, 180, 180, 255] };
                        draw_building(frame, width_i32, height_i32, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, color);
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

fn draw_iso_tile(frame: &mut [u8], width: i32, height: i32, cx: i32, cy: i32, zoom: f32, color: [u8; 4]) {
    // Рисуем ромб TILE_W x TILE_H, масштабированный zoom-ом, с центром в (cx, cy)
    let half_w = ((TILE_W as f32 / 2.0) * zoom).round() as i32;
    let half_h = ((TILE_H as f32 / 2.0) * zoom).round() as i32;

    // Проходим строками от вершины к вершине; ширина строки растёт до середины и сужается
    for dy in -half_h..=half_h {
        let t = dy.abs() as f32 / half_h.max(1) as f32;
        let row_half = ((1.0 - t) * half_w as f32).round() as i32;
        let y = cy + dy;
        if y < 0 || y >= height { continue; }
        let x0 = cx - row_half;
        let x1 = cx + row_half;
        let x0 = x0.clamp(0, width - 1);
        let x1 = x1.clamp(0, width - 1);
        for x in x0..=x1 {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            frame[idx..idx + 4].copy_from_slice(&color);
        }
    }
}

fn draw_iso_outline(frame: &mut [u8], width: i32, height: i32, cx: i32, cy: i32, half_w: i32, half_h: i32, color: [u8; 4]) {

    let mut plot = |x: i32, y: i32| {
        if x < 0 || y < 0 || x >= width || y >= height { return; }
        let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
        frame[idx..idx + 4].copy_from_slice(&color);
    };

    // диагональные стороны ромба
    for i in -half_w..=half_w {
        let y1 = cy + (half_h as f32 * (1.0 - (i as f32 / half_w as f32))).round() as i32;
        let y2 = cy - (half_h as f32 * (1.0 - (i as f32 / half_w as f32))).round() as i32;
        plot(cx + i, y1);
        plot(cx + i, y2);
    }
}

fn draw_building(frame: &mut [u8], width: i32, height: i32, cx: i32, cy: i32, half_w: i32, half_h: i32, color: [u8; 4]) {
    // прямоугольник поверх тайла
    let bw = (half_w as f32 * 1.2) as i32;
    let bh = (half_h as f32 * 1.8) as i32;
    let x0 = (cx - bw / 2).clamp(0, width - 1);
    let x1 = (cx + bw / 2).clamp(0, width - 1);
    let y0 = (cy - bh).clamp(0, height - 1);
    let y1 = (cy - bh / 2).clamp(0, height - 1);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            frame[idx..idx + 4].copy_from_slice(&color);
        }
    }
}

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

