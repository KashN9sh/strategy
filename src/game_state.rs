use glam::{IVec2, Vec2};
use crate::types::{
    Building, BuildingKind, Citizen, Job, LogItem, Resources, WarehouseStore,
    FoodPolicy,
};
use crate::ui::{UICategory, UITab};
use crate::world::World;
use crate::atlas::{TileAtlas, BuildingAtlas, TreeAtlas};
use crate::weather::WeatherSystem;
use crate::console::DeveloperConsole;
use crate::input::Config;
use crate::music::MusicManager;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::Instant;

/// Структура светлячка для ночного освещения
#[derive(Clone, Debug)]
pub struct Firefly {
    pub pos: Vec2,
    pub vel: Vec2,
    pub phase: f32,
    pub life_s: f32,
}

/// Полное состояние игры
pub struct GameState {
    // === Игровое состояние ===
    pub resources: Resources,
    pub buildings: Vec<Building>,
    pub buildings_dirty: bool,
    pub citizens: Vec<Citizen>,
    pub jobs: Vec<Job>,
    pub next_job_id: u64,
    pub logs_on_ground: Vec<LogItem>,
    pub warehouses: Vec<WarehouseStore>,
    pub population: i32,
    pub world: World,
    pub seed: u64,
    
    // === Экономика ===
    pub tax_rate: f32,
    pub speed_mult: f32, // 0.5, 1, 2, 3
    pub food_policy: FoodPolicy,
    
    // === Время и симуляция ===
    pub world_clock_ms: f32,
    pub prev_is_day_flag: bool,
    pub paused: bool,
    pub accumulator_ms: f32,
    pub last_frame: Instant,
    
    // === UI состояние ===
    pub hovered_tile: Option<IVec2>,
    pub selected_building: Option<BuildingKind>,
    pub ui_category: UICategory,
    pub ui_tab: UITab,
    pub show_ui: bool,
    pub cursor_xy: IVec2,
    pub active_building_panel: Option<IVec2>,
    
    // === Дороги ===
    pub road_mode: bool,
    pub left_mouse_down: bool,
    pub drag_road_state: Option<bool>, // Some(true)=build, Some(false)=erase
    pub drag_anchor_tile: Option<IVec2>,
    pub preview_road_path: Vec<IVec2>,
    
    // === Отладка и пути ===
    pub path_debug_mode: bool,
    pub path_sel_a: Option<IVec2>,
    pub path_sel_b: Option<IVec2>,
    pub last_path: Option<Vec<IVec2>>,
    
    // === Биомы и депозиты ===
    pub biome_overlay_debug: bool,
    pub biome_debug_mode: bool,
    pub show_deposits: bool,
    
    // === Рендеринг и визуализация ===
    pub show_grid: bool,
    pub show_forest_overlay: bool,
    pub show_tree_stage_overlay: bool,
    pub atlas: TileAtlas,
    pub building_atlas: Option<BuildingAtlas>,
    pub tree_atlas: Option<TreeAtlas>,
    pub water_anim_time: f32,
    
    // === Визуальные эффекты ===
    pub fireflies: Vec<Firefly>,
    
    // === Системы ===
    pub weather_system: WeatherSystem,
    pub console: DeveloperConsole,
    pub rng: StdRng,
    pub music_manager: Option<MusicManager>,
    
    // === Размеры окна ===
    pub width_i32: i32,
    pub height_i32: i32,
    
    // === Производительность ===
    pub fps_ema: f32,
}

impl GameState {
    /// Создать новое состояние игры с начальными значениями
    pub fn new(rng: &mut StdRng, config: &Config) -> Self {
        let seed = rng.random();
        let mut world = World::new(seed);
        world.apply_biome_config(config);
        
        // Инициализируем начальную область строительства (небольшой радиус вокруг центра)
        // Это позволяет игроку начать строить с самого начала
        let initial_center = IVec2::new(0, 0);
        let initial_radius = 10;
        world.explore_area(initial_center, initial_radius);
        
        const DAY_LENGTH_MS: f32 = 120_000.0;
        const START_HOUR: f32 = 8.0;
        let world_clock_ms = DAY_LENGTH_MS * (START_HOUR / 24.0);
        let prev_is_day_flag = {
            let t0 = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
            let angle0 = t0 * std::f32::consts::TAU;
            let daylight0 = 0.5 - 0.5 * angle0.cos();
            daylight0 > 0.25
        };
        
        Self {
            // Игровое состояние
            resources: Resources {
                wood: 60,
                gold: 200,
                bread: 10,
                fish: 10,
                ..Default::default()
            },
            buildings: Vec::new(),
            buildings_dirty: true,
            citizens: Vec::new(),
            jobs: Vec::new(),
            next_job_id: 1,
            logs_on_ground: Vec::new(),
            warehouses: Vec::new(),
            population: 0,
            world,
            seed,
            
            // Экономика
            tax_rate: 2.0,
            speed_mult: 1.0,
            food_policy: FoodPolicy::Balanced,
            
            // Время и симуляция
            world_clock_ms,
            prev_is_day_flag,
            paused: false,
            accumulator_ms: 0.0,
            last_frame: Instant::now(),
            
            // UI состояние
            hovered_tile: None,
            selected_building: None,
            ui_category: UICategory::Forestry,
            ui_tab: UITab::Build,
            show_ui: true,
            cursor_xy: IVec2::new(0, 0),
            active_building_panel: None,
            
            // Дороги
            road_mode: false,
            left_mouse_down: false,
            drag_road_state: None,
            drag_anchor_tile: None,
            preview_road_path: Vec::new(),
            
            // Отладка
            path_debug_mode: false,
            path_sel_a: None,
            path_sel_b: None,
            last_path: None,
            
            // Биомы
            biome_overlay_debug: false,
            biome_debug_mode: false,
            show_deposits: false,
            
            // Рендеринг
            show_grid: false,
            show_forest_overlay: false,
            show_tree_stage_overlay: false,
            atlas: TileAtlas::new(),
            building_atlas: None,
            tree_atlas: None,
            water_anim_time: 0.0,
            
            // Визуальные эффекты
            fireflies: Vec::new(),
            
            // Системы
            weather_system: WeatherSystem::new(crate::types::WeatherKind::Clear, rng),
            console: DeveloperConsole::new(),
            rng: StdRng::seed_from_u64(rng.random()),
            music_manager: None, // Инициализируется в main.rs после создания GameState
            
            // Размеры
            width_i32: 1280,
            height_i32: 720,
            
            // Производительность
            fps_ema: 60.0,
        }
    }
}

