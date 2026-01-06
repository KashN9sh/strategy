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
pub enum WeatherKind {
    Clear,
    Rain,
    Fog,
    Snow,
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
    ResearchLab,  // Лаборатория исследований
}

#[derive(Clone, Debug)]
pub struct Building {
    pub kind: BuildingKind,
    pub pos: IVec2, // координаты тайла
    pub timer_ms: i32,
    pub workers_target: i32,
    // Для домов: вместимость жильцов (у остальных 0)
    pub capacity: i32,
    // Подсветка при наведении/выборе
    pub is_highlighted: bool,
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
        BuildingKind::ResearchLab => Resources { wood: 50, gold: 100, stone: 30, ..Default::default() },
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum JobKind { ChopWood { pos: IVec2 }, HaulWood { from: IVec2, to: IVec2 } }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job { pub id: u64, pub kind: JobKind, pub taken: bool, pub done: bool }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogItem {
    pub pos: IVec2,
    pub carried: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FoodPolicy {
    #[default]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CitizenState {
    Idle,
    GoingToWork,
    Working,
    GoingToDeposit,
    GoingToFetch,
    GoingHome,
    Sleeping,
}

 


// Подсчёт суммарного ресурса из всех складов
// Теперь использует Visitor Pattern для унификации операций
pub fn warehouses_total_resource(warehouses: &[WarehouseStore], resource: ResourceKind) -> i32 {
    crate::resource_visitor::sum_warehouses_resource(warehouses, resource)
}

// Подсчёт суммарного дерева на складах (удобная функция для обратной совместимости)
pub fn warehouses_total_wood(warehouses: &[WarehouseStore]) -> i32 {
    warehouses_total_resource(warehouses, ResourceKind::Wood)
}

// Суммарное золото на складах (удобная функция для обратной совместимости)
pub fn warehouses_total_gold(warehouses: &[WarehouseStore]) -> i32 {
    warehouses_total_resource(warehouses, ResourceKind::Gold)
}

// Объединить ресурсы из складов и общих ресурсов в одну структуру
// Теперь использует Visitor Pattern для суммирования
pub fn total_resources(warehouses: &[WarehouseStore], resources: &Resources) -> Resources {
    use crate::types::ResourceKind::*;
    
    Resources {
        wood: resources.wood + crate::resource_visitor::sum_warehouses_resource(warehouses, Wood),
        gold: resources.gold + crate::resource_visitor::sum_warehouses_resource(warehouses, Gold),
        stone: resources.stone + crate::resource_visitor::sum_warehouses_resource(warehouses, Stone),
        clay: resources.clay + crate::resource_visitor::sum_warehouses_resource(warehouses, Clay),
        bricks: resources.bricks + crate::resource_visitor::sum_warehouses_resource(warehouses, Bricks),
        wheat: resources.wheat + crate::resource_visitor::sum_warehouses_resource(warehouses, Wheat),
        flour: resources.flour + crate::resource_visitor::sum_warehouses_resource(warehouses, Flour),
        bread: resources.bread + crate::resource_visitor::sum_warehouses_resource(warehouses, Bread),
        fish: resources.fish + crate::resource_visitor::sum_warehouses_resource(warehouses, Fish),
        iron_ore: resources.iron_ore + crate::resource_visitor::sum_warehouses_resource(warehouses, IronOre),
        iron_ingots: resources.iron_ingots + crate::resource_visitor::sum_warehouses_resource(warehouses, IronIngot),
    }
}

// Найти ближайший склад к указанной позиции
pub fn find_nearest_warehouse(warehouses: &[WarehouseStore], pos: IVec2) -> Option<IVec2> {
    warehouses
        .iter()
        .min_by_key(|w| (w.pos.x - pos.x).abs() + (w.pos.y - pos.y).abs())
        .map(|w| w.pos)
}

// Статистика состояний граждан
#[derive(Default, Clone, Copy, Debug)]
pub struct CitizenStateStats {
    pub idle: i32,
    pub working: i32,
    pub sleeping: i32,
    pub hauling: i32,
    pub fetching: i32,
}

// Подсчитать статистику состояний граждан
pub fn count_citizen_states(citizens: &[Citizen]) -> CitizenStateStats {
    let mut stats = CitizenStateStats::default();
    for c in citizens {
        match c.state {
            CitizenState::Idle => stats.idle += 1,
            CitizenState::Working => stats.working += 1,
            CitizenState::Sleeping => stats.sleeping += 1,
            CitizenState::GoingToDeposit => stats.hauling += 1,
            CitizenState::GoingToFetch => stats.fetching += 1,
            CitizenState::GoingToWork | CitizenState::GoingHome => stats.idle += 1,
        }
    }
    stats
}

// Проверка возможности постройки: учитываем ресурсы и склады вместе (wood + gold)
pub fn can_afford_building(warehouses: &[WarehouseStore], resources: &Resources, cost: &Resources) -> bool {
    let total_wood = warehouses_total_wood(warehouses) + resources.wood;
    let total_gold = warehouses_total_gold(warehouses) + resources.gold;
    total_wood >= cost.wood && total_gold >= cost.gold
}

// Списать ресурсы на постройку, забирая сначала со складов, затем из общих ресурсов
// Теперь использует Visitor Pattern для унификации операций списания
pub fn spend_building_cost(warehouses: &mut [WarehouseStore], resources: &mut Resources, cost: &Resources) -> bool {
    if !can_afford_building(warehouses, resources, cost) { return false; }
    
    use crate::resource_visitor::{ResourceVisitable, SpendVisitor};
    use crate::types::ResourceKind::*;
    
    // Дерево
    if cost.wood > 0 {
        let mut need_wood = cost.wood;
        for w in warehouses.iter_mut() {
            if need_wood == 0 { break; }
            let mut visitor = SpendVisitor::new(need_wood);
            w.accept_mut(&mut visitor, Wood);
            need_wood = visitor.amount;
        }
        if need_wood > 0 {
            let mut visitor = SpendVisitor::new(need_wood);
            resources.accept_mut(&mut visitor, Wood);
        }
    }
    
    // Золото
    if cost.gold > 0 {
        let mut need_gold = cost.gold;
        for w in warehouses.iter_mut() {
            if need_gold == 0 { break; }
            let mut visitor = SpendVisitor::new(need_gold);
            w.accept_mut(&mut visitor, Gold);
            need_gold = visitor.amount;
        }
        if need_gold > 0 {
            let mut visitor = SpendVisitor::new(need_gold);
            resources.accept_mut(&mut visitor, Gold);
        }
    }
    
    true
}
