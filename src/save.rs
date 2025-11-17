use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::types::{Building, BuildingKind, Resources};
use crate::research::ResearchSystem;
use crate::notifications::NotificationSystem;

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


