use glam::IVec2;
use crate::types::{Building, BuildingKind, Citizen, ResourceKind, WarehouseStore};
use crate::world::World;
use crate::input::Config;

/// Trait для стратегии производства здания
/// Каждый тип здания реализует свою стратегию производства
pub trait ProductionStrategy {
    /// Обработать производство для рабочего в этом здании
    /// Возвращает true, если производство обработано
    fn process_production(
        &self,
        citizen: &mut Citizen,
        building: &Building,
        warehouses: &mut Vec<WarehouseStore>,
        world: &mut World,
        config: &Config,
        weather_multiplier: f32,
        step_ms: f32,
    ) -> bool;
    
    /// Получить базовое время производства (в мс)
    /// Может быть полезно для отладки и UI
    #[allow(dead_code)]
    fn base_production_time_ms(&self) -> i32;
    
    /// Получить требуемый входной ресурс (если есть)
    /// Может быть полезно для отладки и UI
    #[allow(dead_code)]
    fn required_input_resource(&self) -> Option<ResourceKind>;
    
    /// Получить производимый выходной ресурс
    /// Может быть полезно для отладки и UI
    #[allow(dead_code)]
    fn output_resource(&self) -> Option<ResourceKind>;
}

/// Простая стратегия для зданий, которые добывают ресурсы напрямую
/// (StoneQuarry, ClayPit, IronMine, WheatField, Fishery)
pub struct ExtractionStrategy {
    output: ResourceKind,
    base_time_ms: i32,
}

impl ExtractionStrategy {
    pub fn new(output: ResourceKind, base_time_ms: i32) -> Self {
        Self { output, base_time_ms }
    }
}

impl ProductionStrategy for ExtractionStrategy {
    fn process_production(
        &self,
        citizen: &mut Citizen,
        building: &Building,
        warehouses: &mut Vec<WarehouseStore>,
        world: &mut World,
        _config: &Config,
        weather_multiplier: f32,
        _step_ms: f32,
    ) -> bool {
        if citizen.carrying.is_some() {
            return false;
        }
        
        let production_time = (self.base_time_ms as f32 * weather_multiplier) as i32;
        if citizen.work_timer_ms >= production_time {
            citizen.work_timer_ms = 0;
            if let Some(dst) = crate::types::find_nearest_warehouse(warehouses, building.pos) {
                citizen.carrying = Some((self.output, 1));
                crate::game::plan_path(world, citizen, dst);
                citizen.state = crate::types::CitizenState::GoingToDeposit;
                return true;
            }
        }
        false
    }
    
    fn base_production_time_ms(&self) -> i32 {
        self.base_time_ms
    }
    
    fn required_input_resource(&self) -> Option<ResourceKind> {
        None
    }
    
    fn output_resource(&self) -> Option<ResourceKind> {
        Some(self.output)
    }
}

/// Стратегия для зданий, которые перерабатывают ресурсы
/// (Mill, Bakery, Kiln, Smelter)
pub struct ProcessingStrategy {
    input: ResourceKind,
    output: ResourceKind,
    base_time_ms: i32,
    /// Дополнительные ресурсы, которые нужно списать (например, wood для Kiln)
    additional_cost: Option<(ResourceKind, i32)>,
}

impl ProcessingStrategy {
    pub fn new(input: ResourceKind, output: ResourceKind, base_time_ms: i32) -> Self {
        Self {
            input,
            output,
            base_time_ms,
            additional_cost: None,
        }
    }
    
    pub fn with_additional_cost(mut self, resource: ResourceKind, amount: i32) -> Self {
        self.additional_cost = Some((resource, amount));
        self
    }
}

impl ProductionStrategy for ProcessingStrategy {
    fn process_production(
        &self,
        citizen: &mut Citizen,
        building: &Building,
        warehouses: &mut Vec<WarehouseStore>,
        world: &mut World,
        _config: &Config,
        weather_multiplier: f32,
        _step_ms: f32,
    ) -> bool {
        let has_input = matches!(citizen.carrying, Some((r, _)) if r == self.input);
        
        if !has_input {
            // Нужно получить входной ресурс
            // Используем Visitor Pattern для проверки наличия ресурса
            use crate::resource_visitor::{ResourceVisitable, CheckEnoughVisitor};
            let have_any = warehouses.iter().any(|w| {
                let mut visitor = CheckEnoughVisitor::new(1);
                w.accept(&mut visitor, self.input);
                visitor.result
            });
            
            if have_any {
                if let Some(dst) = crate::types::find_nearest_warehouse(warehouses, building.pos) {
                    citizen.pending_input = Some(self.input);
                    citizen.target = dst;
                    citizen.moving = true;
                    citizen.progress = 0.0;
                    citizen.state = crate::types::CitizenState::GoingToFetch;
                    return true;
                }
            }
            return false;
        }
        
        // Есть входной ресурс, производим
        let production_time = (self.base_time_ms as f32 * weather_multiplier) as i32;
        if citizen.work_timer_ms >= production_time {
            citizen.work_timer_ms = 0;
            
            // Списываем дополнительные ресурсы (например, wood для Kiln)
            // Используем Visitor Pattern для списания
            if let Some((res_kind, amount)) = self.additional_cost {
                use crate::resource_visitor::{ResourceVisitable, CheckEnoughVisitor, SpendVisitor};
                let mut found = false;
                for w in warehouses.iter_mut() {
                    let mut check_visitor = CheckEnoughVisitor::new(amount);
                    w.accept(&mut check_visitor, res_kind);
                    if check_visitor.result {
                        let mut spend_visitor = SpendVisitor::new(amount);
                        w.accept_mut(&mut spend_visitor, res_kind);
                        found = true;
                        break;
                    }
                }
                if !found {
                    return false;
                }
            }
            
            citizen.carrying = None; // Потратили входной ресурс
            if let Some(dst) = crate::types::find_nearest_warehouse(warehouses, building.pos) {
                citizen.carrying = Some((self.output, 1));
                crate::game::plan_path(world, citizen, dst);
                citizen.state = crate::types::CitizenState::GoingToDeposit;
                return true;
            }
        }
        false
    }
    
    fn base_production_time_ms(&self) -> i32 {
        self.base_time_ms
    }
    
    fn required_input_resource(&self) -> Option<ResourceKind> {
        Some(self.input)
    }
    
    fn output_resource(&self) -> Option<ResourceKind> {
        Some(self.output)
    }
}

/// Стратегия для Forester (сажает деревья)
pub struct ForesterStrategy;

impl ProductionStrategy for ForesterStrategy {
    fn process_production(
        &self,
        citizen: &mut Citizen,
        building: &Building,
        _warehouses: &mut Vec<WarehouseStore>,
        world: &mut World,
        _config: &Config,
        weather_multiplier: f32,
        _step_ms: f32,
    ) -> bool {
        let production_time = (4000.0 * weather_multiplier) as i32;
        if citizen.work_timer_ms >= production_time {
            citizen.work_timer_ms = 0;
            
            const R: i32 = 6;
            let mut best: Option<(i32, IVec2)> = None;
            for dy in -R..=R {
                for dx in -R..=R {
                    let p = IVec2::new(building.pos.x + dx, building.pos.y + dy);
                    let tk = world.get_tile(p.x, p.y);
                    if tk != crate::types::TileKind::Water
                        && !world.has_tree(p)
                        && !world.is_occupied(p)
                        && !world.is_road(p)
                        && !world.is_road(IVec2::new(p.x - 1, p.y - 1))
                    {
                        let d = dx.abs() + dy.abs();
                        if best.map(|(bd, _)| d < bd).unwrap_or(true) {
                            best = Some((d, p));
                        }
                    }
                }
            }
            if let Some((_, p)) = best {
                world.plant_tree(p);
                return true;
            }
        }
        false
    }
    
    fn base_production_time_ms(&self) -> i32 {
        4000
    }
    
    fn required_input_resource(&self) -> Option<ResourceKind> {
        None
    }
    
    fn output_resource(&self) -> Option<ResourceKind> {
        None // Forester не производит ресурсы напрямую
    }
}

/// Стратегия для Fishery (требует воду рядом)
pub struct FisheryStrategy;

impl ProductionStrategy for FisheryStrategy {
    fn process_production(
        &self,
        citizen: &mut Citizen,
        building: &Building,
        warehouses: &mut Vec<WarehouseStore>,
        world: &mut World,
        _config: &Config,
        weather_multiplier: f32,
        _step_ms: f32,
    ) -> bool {
        if citizen.carrying.is_some() {
            return false;
        }
        
        // Проверка наличия воды рядом
        const NB: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        if !NB.iter().any(|(dx, dy)| {
            world.get_tile(building.pos.x + dx, building.pos.y + dy) == crate::types::TileKind::Water
        }) {
            return false;
        }
        
        let production_time = (5000.0 * weather_multiplier) as i32;
        if citizen.work_timer_ms >= production_time {
            citizen.work_timer_ms = 0;
            if let Some(dst) = crate::types::find_nearest_warehouse(warehouses, building.pos) {
                citizen.carrying = Some((ResourceKind::Fish, 1));
                crate::game::plan_path(world, citizen, dst);
                citizen.state = crate::types::CitizenState::GoingToDeposit;
                return true;
            }
        }
        false
    }
    
    fn base_production_time_ms(&self) -> i32 {
        5000
    }
    
    fn required_input_resource(&self) -> Option<ResourceKind> {
        None
    }
    
    fn output_resource(&self) -> Option<ResourceKind> {
        Some(ResourceKind::Fish)
    }
}

/// Стратегия для WheatField с учетом биома
pub struct WheatFieldStrategy;

impl ProductionStrategy for WheatFieldStrategy {
    fn process_production(
        &self,
        citizen: &mut Citizen,
        building: &Building,
        warehouses: &mut Vec<WarehouseStore>,
        world: &mut World,
        config: &Config,
        weather_multiplier: f32,
        _step_ms: f32,
    ) -> bool {
        if citizen.carrying.is_some() {
            return false;
        }
        
        // Биомный модификатор
        let biome_multiplier = {
            use crate::types::BiomeKind::*;
            match world.biome(building.pos) {
                Meadow => config.biome_meadow_wheat_wmul,
                Swamp => config.biome_swamp_wheat_wmul,
                _ => 1.0,
            }
        };
        
        let production_time = (6000.0 * weather_multiplier * biome_multiplier) as i32;
        if citizen.work_timer_ms >= production_time {
            citizen.work_timer_ms = 0;
            if let Some(dst) = crate::types::find_nearest_warehouse(warehouses, building.pos) {
                citizen.carrying = Some((ResourceKind::Wheat, 1));
                crate::game::plan_path(world, citizen, dst);
                citizen.state = crate::types::CitizenState::GoingToDeposit;
                return true;
            }
        }
        false
    }
    
    fn base_production_time_ms(&self) -> i32 {
        6000
    }
    
    fn required_input_resource(&self) -> Option<ResourceKind> {
        None
    }
    
    fn output_resource(&self) -> Option<ResourceKind> {
        Some(ResourceKind::Wheat)
    }
}

/// Фабрика для создания стратегий производства
pub fn create_production_strategy(kind: BuildingKind) -> Box<dyn ProductionStrategy> {
    use crate::types::BuildingKind::*;
    use crate::types::ResourceKind::*;
    
    match kind {
        StoneQuarry => Box::new(ExtractionStrategy::new(Stone, 4000)),
        ClayPit => Box::new(ExtractionStrategy::new(Clay, 4000)),
        IronMine => Box::new(ExtractionStrategy::new(IronOre, 5000)),
        WheatField => Box::new(WheatFieldStrategy),
        Fishery => Box::new(FisheryStrategy),
        Mill => Box::new(ProcessingStrategy::new(Wheat, Flour, 5000)),
        Bakery => Box::new(ProcessingStrategy::new(Flour, Bread, 5000)),
        Kiln => Box::new(
            ProcessingStrategy::new(Clay, Bricks, 5000)
                .with_additional_cost(Wood, 1)
        ),
        Smelter => Box::new(ProcessingStrategy::new(IronOre, IronIngot, 6000)),
        Forester => Box::new(ForesterStrategy),
        // Здания без производства
        Lumberjack | House | Warehouse => {
            // Возвращаем пустую стратегию (или можно сделать NoOpStrategy)
            Box::new(NoOpStrategy)
        }
    }
}

/// Пустая стратегия для зданий без производства
struct NoOpStrategy;

impl ProductionStrategy for NoOpStrategy {
    fn process_production(
        &self,
        _citizen: &mut Citizen,
        _building: &Building,
        _warehouses: &mut Vec<WarehouseStore>,
        _world: &mut World,
        _config: &Config,
        _weather_multiplier: f32,
        _step_ms: f32,
    ) -> bool {
        false
    }
    
    fn base_production_time_ms(&self) -> i32 {
        0
    }
    
    fn required_input_resource(&self) -> Option<ResourceKind> {
        None
    }
    
    fn output_resource(&self) -> Option<ResourceKind> {
        None
    }
}

