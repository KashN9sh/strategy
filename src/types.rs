use serde::{Serialize, Deserialize};
use glam::IVec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    Grass,
    Forest,
    Water,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiomeKind {
    Meadow, // луга
    Swamp,  // болота
    Rocky,  // скалы/каменистая местность
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
    pub workers_target: i32,
    // Для домов: вместимость жильцов (у остальных 0)
    pub capacity: i32,
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

// Единый источник стоимости зданий для логики и UI
pub fn building_cost(kind: BuildingKind) -> Resources {
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

#[derive(Clone, Debug)]
pub struct Citizen {
    pub pos: IVec2,      // текущая клетка
    pub target: IVec2,   // цель (для будущего pathfinding)
    pub moving: bool,
    pub progress: f32,   // 0..1 прогресс между клетками
    pub carrying_log: bool,
    pub assigned_job: Option<u64>,
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
    // путь (последовательность клеток) и текущий индекс шага
    pub path: Vec<IVec2>,
    pub path_index: usize,
    // был ли накормлен сегодня (дневная смена)
    pub fed_today: bool,
    // ручное закрепление за рабочим местом (не пере назначать автоматически)
    pub manual_workplace: bool,
    // Счастье 0..100
    pub happiness: u8,
    // Маска потреблённой еды в недавние дни (бит0=bread, бит1=fish), для бонуса разнообразия
    pub last_food_mask: u8,
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

impl Default for WarehouseStore {
    fn default() -> Self {
        Self { pos: IVec2::new(0,0), wood: 0, stone: 0, clay: 0, bricks: 0, wheat: 0, flour: 0, bread: 0, fish: 0, gold: 0, iron_ore: 0, iron_ingots: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoodPolicy {
    Balanced,   // выбирать более доступный ресурс
    BreadFirst, // сначала хлеб
    FishFirst,  // сначала рыба
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

// удалено: DepositKind (не используется)

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

 


// Подсчёт суммарного дерева на складах
pub fn warehouses_total_wood(warehouses: &Vec<WarehouseStore>) -> i32 {
    warehouses.iter().map(|w| w.wood).sum()
}

// Потратить дерево с учётом складов; если складов нет — тратим из общих ресурсов
pub fn spend_wood(warehouses: &mut Vec<WarehouseStore>, resources: &mut Resources, mut amount: i32) -> bool {
    if amount <= 0 { return true; }
    if warehouses.is_empty() {
        if resources.wood >= amount { resources.wood -= amount; return true; } else { return false; }
    }
    let total: i32 = warehouses_total_wood(warehouses);
    if total < amount { return false; }
    for w in warehouses.iter_mut() {
        if amount == 0 { break; }
        let take = amount.min(w.wood);
        w.wood -= take;
        amount -= take;
    }
    true
}

// Суммарное золото на складах
pub fn warehouses_total_gold(warehouses: &Vec<WarehouseStore>) -> i32 {
    warehouses.iter().map(|w| w.gold).sum()
}

// Проверка возможности постройки: учитываем ресурсы и склады вместе (wood + gold)
pub fn can_afford_building(warehouses: &Vec<WarehouseStore>, resources: &Resources, cost: &Resources) -> bool {
    let total_wood = warehouses_total_wood(warehouses) + resources.wood;
    let total_gold = warehouses_total_gold(warehouses) + resources.gold;
    total_wood >= cost.wood && total_gold >= cost.gold
}

// Списать ресурсы на постройку, забирая сначала со складов, затем из общих ресурсов
pub fn spend_building_cost(warehouses: &mut Vec<WarehouseStore>, resources: &mut Resources, cost: &Resources) -> bool {
    if !can_afford_building(warehouses, resources, cost) { return false; }
    // Дерево
    let mut need_wood = cost.wood;
    for w in warehouses.iter_mut() {
        if need_wood == 0 { break; }
        let take = need_wood.min(w.wood);
        w.wood -= take;
        need_wood -= take;
    }
    if need_wood > 0 { resources.wood -= need_wood; }
    // Золото
    let mut need_gold = cost.gold;
    for w in warehouses.iter_mut() {
        if need_gold == 0 { break; }
        let take = need_gold.min(w.gold);
        w.gold -= take;
        need_gold -= take;
    }
    if need_gold > 0 { resources.gold -= need_gold; }
    true
}
