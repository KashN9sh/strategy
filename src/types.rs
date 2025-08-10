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
    Warehouse,
    Forester,
    StoneQuarry,
    ClayPit,
    Kiln,
    WheatField,
    Mill,
    Bakery,
    Fishery,
    IronMine,
    Smelter,
}

#[derive(Clone, Debug)]
pub struct Building {
    pub kind: BuildingKind,
    pub pos: IVec2, // координаты тайла
    pub timer_ms: i32,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct Resources {
    pub wood: i32,
    pub gold: i32,
    pub stone: i32,
    pub clay: i32,
    pub bricks: i32,
    pub wheat: i32,
    pub flour: i32,
    pub bread: i32,
    pub fish: i32,
    pub iron_ore: i32,
    pub iron_ingots: i32,
}

#[derive(Clone, Debug)]
pub struct Citizen {
    pub pos: IVec2,      // текущая клетка
    pub target: IVec2,   // цель (для будущего pathfinding)
    pub moving: bool,
    pub progress: f32,   // 0..1 прогресс между клетками
    pub carrying_log: bool,
    pub assigned_job: Option<u64>,
    pub deliver_to: IVec2,
    // анти-залипание: таймер ожидания у цели (мс)
    pub idle_timer_ms: i32,
    // суточный цикл и работа
    pub home: IVec2,
    pub workplace: Option<IVec2>,
    pub state: CitizenState,
    pub work_timer_ms: i32,
    // перенос любых ресурсов (в дополнение к временной системе поленьев)
    pub carrying: Option<(ResourceKind, i32)>,
    // ожидаемый к доставке входной ресурс для цикла работы
    pub pending_input: Option<ResourceKind>,
}

#[derive(Clone, Debug)]
pub enum JobKind { ChopWood { pos: IVec2 }, HaulWood { from: IVec2, to: IVec2 } }

#[derive(Clone, Debug)]
pub struct Job { pub id: u64, pub kind: JobKind, pub taken: bool, pub done: bool }

#[derive(Clone, Debug)]
pub struct LogItem {
    pub pos: IVec2,
    pub carried: bool,
}

#[derive(Clone, Debug)]
pub struct WarehouseStore {
    pub pos: IVec2,
    pub wood: i32,
    pub stone: i32,
    pub clay: i32,
    pub bricks: i32,
    pub wheat: i32,
    pub flour: i32,
    pub bread: i32,
    pub fish: i32,
    pub gold: i32,
    pub iron_ore: i32,
    pub iron_ingots: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Wood,
    Stone,
    Clay,
    Bricks,
    Wheat,
    Flour,
    Bread,
    Fish,
    Gold,
    IronOre,
    IronIngot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepositKind { Clay, Stone, Iron }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CitizenState {
    Idle,
    GoingToWork,
    Working,
    GoingToDeposit,
    GoingToFetch,
    GoingHome,
    Sleeping,
}

 


