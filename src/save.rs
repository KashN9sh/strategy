use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::types::{Building, BuildingKind, Resources, Citizen, Job, WarehouseStore, LogItem, FoodPolicy};
use crate::research::ResearchSystem;
use crate::notifications::NotificationSystem;
use crate::quests::QuestSystem;

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub seed: u64,
    pub resources: Resources,
    pub buildings: Vec<SaveBuilding>,
    pub cam_x: f32,
    pub cam_y: f32,
    pub zoom: f32,
    pub trees: Vec<SaveTree>,
    #[serde(default)]
    pub research_system: Option<ResearchSystem>,
    #[serde(default)]
    pub notification_system: Option<NotificationSystem>,
    #[serde(default)]
    pub quest_system: Option<QuestSystem>,
    // Расширенные данные
    #[serde(default)]
    pub citizens: Vec<Citizen>,
    #[serde(default)]
    pub jobs: Vec<Job>,
    #[serde(default)]
    pub next_job_id: u64,
    #[serde(default)]
    pub logs_on_ground: Vec<LogItem>,
    #[serde(default)]
    pub warehouses: Vec<WarehouseStore>,
    #[serde(default)]
    pub population: i32,
    #[serde(default)]
    pub world_clock_ms: f32,
    #[serde(default)]
    pub tax_rate: f32,
    #[serde(default)]
    pub speed_mult: f32,
    #[serde(default)]
    pub food_policy: FoodPolicy,
    // Туман войны (разведанные тайлы)
    #[serde(default)]
    pub explored_tiles: Vec<(i32, i32)>,
    // Дороги
    #[serde(default)]
    pub roads: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SaveBuilding {
    pub kind: BuildingKind,
    pub x: i32,
    pub y: i32,
    pub timer_ms: i32,
    #[serde(default)]
    pub workers_target: i32,
    #[serde(default)]
    pub capacity: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SaveTree {
    pub x: i32,
    pub y: i32,
    pub stage: u8,
    pub age_ms: i32,
}

impl SaveData {
    pub fn from_runtime(
        seed: u64,
        res: &Resources,
        buildings: &Vec<Building>,
        cam_px: Vec2,
        zoom: f32,
        world: &crate::world::World,
        research_system: &ResearchSystem,
        notification_system: &NotificationSystem,
        quest_system: &QuestSystem,
        citizens: &Vec<Citizen>,
        jobs: &Vec<Job>,
        next_job_id: u64,
        logs_on_ground: &Vec<LogItem>,
        warehouses: &Vec<WarehouseStore>,
        population: i32,
        world_clock_ms: f32,
        tax_rate: f32,
        speed_mult: f32,
        food_policy: FoodPolicy,
    ) -> Self {
        let buildings = buildings
            .iter()
            .map(|b| SaveBuilding {
                kind: b.kind,
                x: b.pos.x,
                y: b.pos.y,
                timer_ms: b.timer_ms,
                workers_target: b.workers_target,
                capacity: b.capacity,
            })
            .collect();
        let mut trees = Vec::new();
        for (&(x, y), tr) in world.trees.iter() {
            trees.push(SaveTree { x, y, stage: tr.stage, age_ms: tr.age_ms });
        }
        // Сохраняем разведанные тайлы (туман войны)
        let explored_tiles: Vec<(i32, i32)> = world.explored_tiles.iter().copied().collect();
        // Сохраняем дороги
        let roads: Vec<(i32, i32)> = world.roads.iter().copied().collect();
        SaveData { 
            seed, 
            resources: *res, 
            buildings, 
            cam_x: cam_px.x, 
            cam_y: cam_px.y, 
            zoom, 
            trees,
            research_system: Some(research_system.clone()),
            notification_system: Some(notification_system.clone()),
            quest_system: Some(quest_system.clone()),
            citizens: citizens.clone(),
            jobs: jobs.clone(),
            next_job_id,
            logs_on_ground: logs_on_ground.clone(),
            warehouses: warehouses.clone(),
            population,
            world_clock_ms,
            tax_rate,
            speed_mult,
            food_policy,
            explored_tiles,
            roads,
        }
    }

    pub fn to_buildings(&self) -> Vec<Building> {
        self.buildings
            .iter()
            .map(|sb| Building {
                kind: sb.kind,
                pos: glam::IVec2::new(sb.x, sb.y),
                timer_ms: sb.timer_ms,
                workers_target: sb.workers_target,
                capacity: sb.capacity,
                is_highlighted: false,
            })
            .collect()
    }
}

pub fn save_game(data: &SaveData) -> anyhow::Result<()> {
    let txt = serde_json::to_string_pretty(data)?;
    std::fs::write("save.json", txt)?;
    Ok(())
}

pub fn load_game() -> anyhow::Result<SaveData> {
    let txt = std::fs::read_to_string("save.json")?;
    let data: SaveData = serde_json::from_str(&txt)?;
    Ok(data)
}


