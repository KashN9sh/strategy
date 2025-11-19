use serde::{Serialize, Deserialize};
use crate::types::{BuildingKind, Resources};

/// Типы исследований
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResearchKind {
    // Базовые постройки (доступны с начала)
    BasicHousing,      // Дом
    BasicStorage,      // Склад
    BasicForestry,     // Лесоруб, Лесничий
    
    // Уровень 1 - Требует Лабораторию
    AdvancedHousing,   // Улучшенные дома
    StoneWorking,      // Каменоломня, Глиняный карьер
    BasicFarming,      // Пшеничное поле
    BasicFishing,      // Рыбацкая хижина
    
    // Уровень 2
    Brickmaking,       // Печь для кирпичей
    FoodProcessing,    // Мельница, Пекарня
    Mining,            // Железная шахта
    
    // Уровень 3
    Metallurgy,        // Плавильня
    AdvancedFarming,   // Улучшенные фермы (будущее)
    AdvancedMining,    // Улучшенные шахты (будущее)
}

/// Статус исследования
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResearchStatus {
    Locked,            // Недоступно (нет пререквизитов)
    Available,         // Доступно для исследования
    InProgress,        // В процессе исследования
    Completed,         // Завершено
}

/// Данные активного исследования
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveResearch {
    pub kind: ResearchKind,
    pub days_remaining: i32,
}

/// Состояние одного исследования
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Research {
    pub kind: ResearchKind,
    pub status: ResearchStatus,
}

/// Информация об исследовании
pub struct ResearchInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub days_required: i32,
    pub cost: Resources,
    pub prerequisites: &'static [ResearchKind],
    pub unlocks_buildings: &'static [BuildingKind],
}

impl ResearchKind {
    /// Получить информацию об исследовании
    pub fn info(self) -> ResearchInfo {
        match self {
            ResearchKind::BasicHousing => ResearchInfo {
                name: "Basic Housing",
                description: "Unlocks construction of houses for citizens",
                days_required: 0,
                cost: Resources::default(),
                prerequisites: &[],
                unlocks_buildings: &[BuildingKind::House],
            },
            ResearchKind::BasicStorage => ResearchInfo {
                name: "Basic Storage",
                description: "Unlocks construction of warehouses",
                days_required: 0,
                cost: Resources::default(),
                prerequisites: &[],
                unlocks_buildings: &[BuildingKind::Warehouse],
            },
            ResearchKind::BasicForestry => ResearchInfo {
                name: "Basic Forestry",
                description: "Unlocks lumberjacks and foresters",
                days_required: 0,
                cost: Resources::default(),
                prerequisites: &[],
                unlocks_buildings: &[BuildingKind::Lumberjack, BuildingKind::Forester],
            },
            
            ResearchKind::AdvancedHousing => ResearchInfo {
                name: "Advanced Housing",
                description: "Improved houses with greater capacity",
                days_required: 3,
                cost: Resources { wood: 50, gold: 100, ..Default::default() },
                prerequisites: &[ResearchKind::BasicHousing],
                unlocks_buildings: &[],
            },
            ResearchKind::StoneWorking => ResearchInfo {
                name: "Stone Working",
                description: "Unlocks quarry and clay pit",
                days_required: 5,
                cost: Resources { wood: 100, gold: 150, ..Default::default() },
                prerequisites: &[ResearchKind::BasicForestry],
                unlocks_buildings: &[BuildingKind::StoneQuarry, BuildingKind::ClayPit],
            },
            ResearchKind::BasicFarming => ResearchInfo {
                name: "Basic Farming",
                description: "Unlocks wheat fields",
                days_required: 4,
                cost: Resources { wood: 80, gold: 120, ..Default::default() },
                prerequisites: &[ResearchKind::BasicForestry],
                unlocks_buildings: &[BuildingKind::WheatField],
            },
            ResearchKind::BasicFishing => ResearchInfo {
                name: "Basic Fishing",
                description: "Unlocks fishing hut",
                days_required: 3,
                cost: Resources { wood: 60, gold: 80, ..Default::default() },
                prerequisites: &[ResearchKind::BasicForestry],
                unlocks_buildings: &[BuildingKind::Fishery],
            },
            
            ResearchKind::Brickmaking => ResearchInfo {
                name: "Brickmaking",
                description: "Unlocks kiln for brick production",
                days_required: 6,
                cost: Resources { wood: 150, gold: 200, stone: 50, clay: 50, ..Default::default() },
                prerequisites: &[ResearchKind::StoneWorking],
                unlocks_buildings: &[BuildingKind::Kiln],
            },
            ResearchKind::FoodProcessing => ResearchInfo {
                name: "Food Processing",
                description: "Unlocks mill and bakery",
                days_required: 7,
                cost: Resources { wood: 180, gold: 250, stone: 30, ..Default::default() },
                prerequisites: &[ResearchKind::BasicFarming],
                unlocks_buildings: &[BuildingKind::Mill, BuildingKind::Bakery],
            },
            ResearchKind::Mining => ResearchInfo {
                name: "Mining",
                description: "Unlocks iron mine",
                days_required: 8,
                cost: Resources { wood: 200, gold: 300, stone: 100, ..Default::default() },
                prerequisites: &[ResearchKind::StoneWorking],
                unlocks_buildings: &[BuildingKind::IronMine],
            },
            
            ResearchKind::Metallurgy => ResearchInfo {
                name: "Metallurgy",
                description: "Unlocks smelter for ingot production",
                days_required: 10,
                cost: Resources { wood: 250, gold: 400, stone: 150, bricks: 50, ..Default::default() },
                prerequisites: &[ResearchKind::Mining, ResearchKind::Brickmaking],
                unlocks_buildings: &[BuildingKind::Smelter],
            },
            ResearchKind::AdvancedFarming => ResearchInfo {
                name: "Advanced Farming",
                description: "Improved farming methods",
                days_required: 12,
                cost: Resources { wood: 300, gold: 500, ..Default::default() },
                prerequisites: &[ResearchKind::FoodProcessing],
                unlocks_buildings: &[],
            },
            ResearchKind::AdvancedMining => ResearchInfo {
                name: "Advanced Mining",
                description: "Improved mining methods",
                days_required: 12,
                cost: Resources { wood: 300, gold: 500, iron_ingots: 20, ..Default::default() },
                prerequisites: &[ResearchKind::Metallurgy],
                unlocks_buildings: &[],
            },
        }
    }
    
    /// Все виды исследований в порядке отображения
    pub fn all() -> &'static [ResearchKind] {
        &[
            ResearchKind::BasicHousing,
            ResearchKind::BasicStorage,
            ResearchKind::BasicForestry,
            ResearchKind::AdvancedHousing,
            ResearchKind::StoneWorking,
            ResearchKind::BasicFarming,
            ResearchKind::BasicFishing,
            ResearchKind::Brickmaking,
            ResearchKind::FoodProcessing,
            ResearchKind::Mining,
            ResearchKind::Metallurgy,
            ResearchKind::AdvancedFarming,
            ResearchKind::AdvancedMining,
        ]
    }
    
    /// Позиция в дереве исследований (для UI)
    pub fn tree_position(self) -> (i32, i32) {
        match self {
            // Уровень 0 (верхний ряд - стартовые, не показываются т.к. завершены)
            ResearchKind::BasicHousing => (0, 0),
            ResearchKind::BasicStorage => (1, 0),
            ResearchKind::BasicForestry => (2, 0),
            
            // Уровень 1 - Первый ряд видимых исследований
            ResearchKind::AdvancedHousing => (0, 1),
            ResearchKind::StoneWorking => (1, 1),
            ResearchKind::BasicFarming => (2, 1),
            ResearchKind::BasicFishing => (3, 1),
            
            // Уровень 2 - Второй ряд
            ResearchKind::Brickmaking => (0, 2),
            ResearchKind::Mining => (1, 2),
            ResearchKind::FoodProcessing => (2, 2),
            
            // Уровень 3 - Третий ряд
            ResearchKind::Metallurgy => (1, 3),
            ResearchKind::AdvancedFarming => (2, 3),
            
            // Уровень 4 - Четвертый ряд
            ResearchKind::AdvancedMining => (1, 4),
        }
    }
}

/// Система управления исследованиями
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchSystem {
    pub researches: Vec<Research>,
    pub active_research: Option<ActiveResearch>,
    #[serde(default)]
    pub has_research_lab: bool,
}

impl ResearchSystem {
    /// Создать новую систему исследований
    pub fn new() -> Self {
        let mut researches = Vec::new();
        
        // Инициализируем все исследования
        for &kind in ResearchKind::all() {
            let info = kind.info();
            let status = if info.prerequisites.is_empty() {
                ResearchStatus::Completed // Базовые исследования завершены с начала
            } else {
                ResearchStatus::Locked
            };
            
            researches.push(Research { kind, status });
        }
        
        let mut system = Self {
            researches,
            active_research: None,
            has_research_lab: false,
        };
        
        // Обновляем статусы, чтобы разблокировать доступные исследования
        system.update_statuses();
        
        system
    }
    
    /// Обновить статусы исследований на основе завершённых
    pub fn update_statuses(&mut self) {
        for i in 0..self.researches.len() {
            if self.researches[i].status == ResearchStatus::InProgress 
                || self.researches[i].status == ResearchStatus::Completed {
                continue;
            }
            
            let kind = self.researches[i].kind;
            let info = kind.info();
            
            // Проверяем, выполнены ли все пререквизиты
            let all_prerequisites_met = info.prerequisites.iter().all(|&prereq| {
                self.researches.iter().any(|r| r.kind == prereq && r.status == ResearchStatus::Completed)
            });
            
            if all_prerequisites_met {
                self.researches[i].status = ResearchStatus::Available;
            } else {
                self.researches[i].status = ResearchStatus::Locked;
            }
        }
    }
    
    /// Начать исследование
    pub fn start_research(&mut self, kind: ResearchKind) -> bool {
        // Проверяем, что нет активного исследования
        if self.active_research.is_some() {
            return false;
        }
        
        // Находим исследование
        if let Some(research) = self.researches.iter_mut().find(|r| r.kind == kind) {
            if research.status == ResearchStatus::Available {
                let info = kind.info();
                research.status = ResearchStatus::InProgress;
                self.active_research = Some(ActiveResearch {
                    kind,
                    days_remaining: info.days_required,
                });
                return true;
            }
        }
        
        false
    }
    
    /// Обновить прогресс исследования (вызывается каждый игровой день)
    pub fn update_daily(&mut self) -> Option<ResearchKind> {
        if let Some(ref mut active) = self.active_research {
            active.days_remaining -= 1;
            
            if active.days_remaining <= 0 {
                let completed_kind = active.kind;
                
                // Завершаем исследование
                if let Some(research) = self.researches.iter_mut().find(|r| r.kind == completed_kind) {
                    research.status = ResearchStatus::Completed;
                }
                
                self.active_research = None;
                self.update_statuses();
                
                return Some(completed_kind);
            }
        }
        
        None
    }
    
    /// Проверить, разблокировано ли здание
    pub fn is_building_unlocked(&self, building: BuildingKind) -> bool {
        for research in &self.researches {
            if research.status == ResearchStatus::Completed {
                let info = research.kind.info();
                if info.unlocks_buildings.contains(&building) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Получить статус исследования
    pub fn get_status(&self, kind: ResearchKind) -> ResearchStatus {
        self.researches.iter()
            .find(|r| r.kind == kind)
            .map(|r| r.status)
            .unwrap_or(ResearchStatus::Locked)
    }
}

