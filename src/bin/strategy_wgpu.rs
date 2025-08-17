use anyhow::Result;
use glam::{IVec2, Vec2};
use strategy::types::{BuildingKind, Building, Resources, Citizen, Job, LogItem, WarehouseStore, FoodPolicy, TileKind, CitizenState, ResourceKind, WeatherKind as WK};
use strategy::world::World;
use strategy::atlas::{TileAtlas, BuildingAtlas, RoadAtlas, TreeAtlas};
use strategy::render::wgpu_renderer::WgpuRenderer;
use strategy::render::texture_manager::TextureManager;
use strategy::ui;
use strategy::input;
use strategy::config;
use strategy::game;
use strategy::path;
use strategy::jobs;
use strategy::controls;
use strategy::ui_interaction;
use strategy::palette;
use strategy::globals;
use std::time::Instant;
use rand::{rngs::StdRng, Rng, SeedableRng};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use std::sync::Arc;

type ResolvedInput = input::ResolvedInput;

#[derive(serde::Serialize, serde::Deserialize)]
struct SaveData {
    seed: u64,
    resources: Resources,
    buildings: Vec<SaveBuilding>,
    cam_x: f32,
    cam_y: f32,
    zoom: f32,
    trees: Vec<SaveTree>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
struct SaveBuilding { 
    kind: BuildingKind, 
    x: i32, 
    y: i32, 
    timer_ms: i32, 
    #[serde(default)] 
    workers_target: i32 
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
struct SaveTree { 
    x: i32, 
    y: i32, 
    stage: u8, 
    age_ms: i32 
}

#[derive(Clone, Debug)]
struct Firefly { 
    pos: Vec2, 
    vel: Vec2, 
    phase: f32, 
    life_s: f32 
}

fn main() -> Result<()> {
    run()
}

fn run() -> Result<()> {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Strategy Isometric Prototype - WGPU")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop).unwrap());

    // Инициализируем wgpu renderer
    let mut renderer = pollster::block_on(WgpuRenderer::new(window.clone())).unwrap();
    
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
    let mut food_policy: FoodPolicy = FoodPolicy::Balanced;
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
    let mut road_atlas = RoadAtlas::new();
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
    let mut tree_atlas: Option<TreeAtlas> = None;

    // Загружаем атласы и текстуры
    println!("Загружаем атласы и текстуры...");
    
    // Загружаем spritesheet для атласа
    if let Ok(img) = image::open("assets/spritesheet.png") {
        let img = img.to_rgba8();
        renderer.texture_manager.load_texture_from_image(&renderer.device, &renderer.queue, "spritesheet", &img).unwrap();
        println!("✅ Spritesheet загружен успешно");
        
        // Инициализируем TileAtlas с данными из spritesheet
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
    }
    
    // Загружаем остальные текстуры
    let texture_files = ["tiles.png", "buildings.png", "faces.png", "trees.png"];
    for texture_file in &texture_files {
        if let Ok(img) = image::open(format!("assets/{}", texture_file)) {
            let img = img.to_rgba8();
            let texture_name = texture_file.replace(".png", "");
            
            let result = renderer.texture_manager.load_texture_from_image(
                &renderer.device,
                &renderer.queue,
                &texture_name,
                &img,
            );
            
            match result {
                Ok(_) => println!("✅ {} загружен успешно", texture_file),
                Err(e) => println!("❌ Ошибка загрузки {}: {:?}", texture_file, e),
            }
        }
    }
    
    println!("Доступные текстуры: {:?}", renderer.texture_manager.list_textures());

    // Создаем renderers для игры
    println!("Создаем renderers...");
    
    // Размер карты (пока фиксированный, потом можно сделать динамическим)
    let map_width = 50;
    let map_height = 50;
    let tile_size = 0.05; // Размер тайла в мировых координатах (нормализованные координаты)
    
    // Создаем atlas renderer
    renderer.create_atlas_renderer(map_width, map_height, tile_size);
    println!("✅ Atlas renderer создан!");
    
    // Создаем building renderer
    renderer.create_building_renderer(tile_size);
    println!("✅ Building renderer создан!");
    
    // Создаем UI renderer
    let window_size = window.inner_size();
    renderer.create_ui_renderer(window_size.width as f32, window_size.height as f32);
    println!("✅ UI renderer создан!");

    // Основной игровой цикл
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => elwt.exit(),
            
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                renderer.resize(physical_size);
                
                // Обновляем UI renderer с новыми размерами
                if let Some(ref mut ui_renderer) = renderer.ui_renderer {
                    ui_renderer.resize(&renderer.device, physical_size.width as f32, physical_size.height as f32);
                }
            }
            
            // TODO: Добавить обработку клавиш для управления камерой и игрой
            
            Event::AboutToWait => {
                window.request_redraw();
            }
            
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Обновляем игровую логику
                let now = Instant::now();
                let delta_time = now.duration_since(last_frame).as_secs_f32();
                last_frame = now;
                
                if !paused {
                    accumulator_ms += delta_time * 1000.0 * speed_mult;
                    
                    // Обновляем игру каждый день
                    if accumulator_ms > 1000.0 {
                        game::new_day_feed_and_income(&mut citizens, &mut resources, &mut warehouses, food_policy);
                        accumulator_ms = 0.0;
                    }
                    
                    // Обновляем здания
                    if buildings_dirty {
                        if let Some(ref mut building_renderer) = renderer.building_renderer {
                            building_renderer.update_buildings(&renderer.device, buildings.clone());
                            buildings_dirty = false;
                        }
                    }
                    
                    // Обновляем AtlasRenderer с реальными данными из World
                    renderer.update_atlas_world_data(&mut world, cam_px.x, cam_px.y);
                }
                
                // Создаем UI с реальными данными игры
                let ui_elements = vec![
                    strategy::render::ui_renderer::UIRenderer::create_panel(10.0, 10.0, 250.0, 150.0),
                    strategy::render::ui_renderer::UIRenderer::create_text_element(20.0, 20.0, 230.0, 20.0, format!("Дерево: {}", resources.wood)),
                    strategy::render::ui_renderer::UIRenderer::create_text_element(20.0, 45.0, 230.0, 20.0, format!("Золото: {}", resources.gold)),
                    strategy::render::ui_renderer::UIRenderer::create_text_element(20.0, 70.0, 230.0, 20.0, format!("Хлеб: {}", resources.bread)),
                    strategy::render::ui_renderer::UIRenderer::create_text_element(20.0, 95.0, 230.0, 20.0, format!("Рыба: {}", resources.fish)),
                    strategy::render::ui_renderer::UIRenderer::create_text_element(20.0, 120.0, 230.0, 20.0, format!("Население: {}", population)),
                ];
                
                if let Some(ref mut ui_renderer) = renderer.ui_renderer {
                    ui_renderer.update_elements(&renderer.device, ui_elements);
                }
                
                // Рендерим кадр
                match renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => eprintln!("Ошибка рендеринга: {:?}", e),
                }
            }
            
            _ => {}
        }
    }).unwrap();
    
    Ok(())
}
