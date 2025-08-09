use serde::{Serialize, Deserialize};
use glam::IVec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    Grass,
    Forest,
    Water,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingKind {
    Lumberjack,
    House,
}

#[derive(Clone, Debug)]
pub struct Building {
    pub kind: BuildingKind,
    pub pos: IVec2, // координаты тайла
    pub timer_ms: i32,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Resources {
    pub wood: i32,
    pub gold: i32,
}


