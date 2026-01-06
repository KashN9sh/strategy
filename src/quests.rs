use serde::{Serialize, Deserialize};
use crate::types::{BuildingKind, Resources};
use rand::Rng;

/// Тип квеста
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestKind {
    /// Собрать определенное количество ресурса
    CollectResource {
        resource_name: String,
        target_amount: i32,
        current_amount: i32,
    },
    /// Построить определенное количество зданий
    BuildBuildings {
        building_kind: BuildingKind,
        target_count: i32,
        current_count: i32,
    },
    /// Достичь определенного населения
    ReachPopulation {
        target_population: i32,
        current_population: i32,
    },
    /// Собрать определенное количество золота
    CollectGold {
        target_amount: i32,
        current_amount: i32,
    },
}

/// Квест
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quest {
    pub id: u32,
    pub kind: QuestKind,
    pub title: String,
    pub description: String,
    pub reward_gold: i32,
    pub completed: bool,
}

/// Система управления квестами
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestSystem {
    pub active_quests: Vec<Quest>,
    pub next_quest_id: u32,
    pub next_quest_timer_ms: f32,
    pub next_quest_interval_ms: f32, // Интервал между появлением новых квестов
}

impl QuestSystem {
    pub fn new() -> Self {
        Self {
            active_quests: Vec::new(),
            next_quest_id: 1,
            next_quest_timer_ms: 0.0,
            next_quest_interval_ms: 60000.0, // 60 секунд по умолчанию
        }
    }
    
    /// Обновить систему квестов
    pub fn update(&mut self, delta_ms: f32, rng: &mut impl Rng, resources: &Resources, warehouses: &[crate::types::WarehouseStore], buildings: &[crate::types::Building], population: i32) -> Vec<Quest> {
        let mut completed_quests = Vec::new();
        
        // Обновляем таймер появления новых квестов
        self.next_quest_timer_ms -= delta_ms;
        
        // Вычисляем общее количество ресурсов (включая склады)
        let total_res = crate::types::total_resources(warehouses, resources);
        
        // Генерируем новый квест, если пришло время и есть место (максимум 3 активных квеста)
        if self.next_quest_timer_ms <= 0.0 && self.active_quests.len() < 3 {
            if let Some(new_quest) = Self::generate_random_quest(rng, self.next_quest_id, &total_res, buildings, population) {
                self.active_quests.push(new_quest);
                self.next_quest_id += 1;
                self.next_quest_timer_ms = self.next_quest_interval_ms;
            }
        }
        
        // Проверяем выполнение квестов
        for quest in &mut self.active_quests {
            if quest.completed {
                continue;
            }
            
            let is_completed = match &mut quest.kind {
                QuestKind::CollectResource { resource_name, current_amount, target_amount } => {
                    let current = match resource_name.as_str() {
                        "Wood" => total_res.wood,
                        "Stone" => total_res.stone,
                        "Clay" => total_res.clay,
                        "Bricks" => total_res.bricks,
                        "Wheat" => total_res.wheat,
                        "Flour" => total_res.flour,
                        "Bread" => total_res.bread,
                        "Fish" => total_res.fish,
                        "Iron Ore" => total_res.iron_ore,
                        "Iron Ingots" => total_res.iron_ingots,
                        _ => 0,
                    };
                    *current_amount = current;
                    current >= *target_amount
                }
                QuestKind::BuildBuildings { building_kind, current_count, target_count } => {
                    *current_count = buildings.iter().filter(|b| b.kind == *building_kind).count() as i32;
                    *current_count >= *target_count
                }
                QuestKind::ReachPopulation { current_population, target_population } => {
                    *current_population = population;
                    *current_population >= *target_population
                }
                QuestKind::CollectGold { current_amount, target_amount } => {
                    *current_amount = total_res.gold;
                    *current_amount >= *target_amount
                }
            };
            
            if is_completed && !quest.completed {
                quest.completed = true;
                completed_quests.push(quest.clone());
            }
        }
        
        // Удаляем завершенные квесты (оставляем их на некоторое время для показа награды)
        // Пока просто удаляем сразу
        self.active_quests.retain(|q| !q.completed);
        
        completed_quests
    }
    
    /// Генерировать случайный квест
    fn generate_random_quest(
        rng: &mut impl Rng,
        quest_id: u32,
        total_resources: &Resources,
        buildings: &[crate::types::Building],
        population: i32,
    ) -> Option<Quest> {
        let quest_type = rng.random_range(0..4);
        
        match quest_type {
            0 => {
                // Квест на сбор ресурса
                let resources_list = vec![
                    ("Wood", total_resources.wood),
                    ("Stone", total_resources.stone),
                    ("Clay", total_resources.clay),
                    ("Bricks", total_resources.bricks),
                    ("Wheat", total_resources.wheat),
                    ("Bread", total_resources.bread),
                    ("Fish", total_resources.fish),
                ];
                
                let idx = rng.random_range(0..resources_list.len());
                if let Some((name, current)) = resources_list.get(idx) {
                    let target = current + rng.random_range(10..50);
                    let reward = target / 2;
                    
                    Some(Quest {
                        id: quest_id,
                        kind: QuestKind::CollectResource {
                            resource_name: name.to_string(),
                            target_amount: target,
                            current_amount: *current,
                        },
                        title: format!("Collect {} {}", target, name),
                        description: format!("Gather {} units of {}", target, name),
                        reward_gold: reward.max(10),
                        completed: false,
                    })
                } else {
                    None
                }
            }
            1 => {
                // Квест на постройку зданий
                let building_kinds = vec![
                    BuildingKind::House,
                    BuildingKind::Lumberjack,
                    BuildingKind::Warehouse,
                    BuildingKind::WheatField,
                    BuildingKind::Fishery,
                ];
                
                let idx = rng.random_range(0..building_kinds.len());
                if let Some(&building_kind) = building_kinds.get(idx) {
                    let current = buildings.iter().filter(|b| b.kind == building_kind).count() as i32;
                    let target = current + rng.random_range(1..4);
                    let reward = target * 20;
                    
                    let building_name = match building_kind {
                        BuildingKind::House => "Houses",
                        BuildingKind::Lumberjack => "Lumberjacks",
                        BuildingKind::Warehouse => "Warehouses",
                        BuildingKind::WheatField => "Wheat Fields",
                        BuildingKind::Fishery => "Fisheries",
                        _ => "Buildings",
                    };
                    
                    Some(Quest {
                        id: quest_id,
                        kind: QuestKind::BuildBuildings {
                            building_kind,
                            target_count: target,
                            current_count: current,
                        },
                        title: format!("Build {} {}", target, building_name),
                        description: format!("Construct {} {}", target, building_name),
                        reward_gold: reward.max(20),
                        completed: false,
                    })
                } else {
                    None
                }
            }
            2 => {
                // Квест на достижение населения
                let target = population + rng.random_range(5..15);
                let reward = (target - population) * 5;
                
                Some(Quest {
                    id: quest_id,
                    kind: QuestKind::ReachPopulation {
                        target_population: target,
                        current_population: population,
                    },
                    title: format!("Reach {} Population", target),
                    description: format!("Grow your population to {} citizens", target),
                    reward_gold: reward.max(25),
                    completed: false,
                })
            }
            3 => {
                // Квест на сбор золота
                let target = total_resources.gold + rng.random_range(50..200);
                let reward = (target - total_resources.gold) / 5;
                
                    Some(Quest {
                        id: quest_id,
                        kind: QuestKind::CollectGold {
                            target_amount: target,
                            current_amount: total_resources.gold,
                        },
                    title: format!("Collect {} Gold", target),
                    description: format!("Accumulate {} gold coins", target),
                    reward_gold: reward.max(30),
                    completed: false,
                })
            }
            _ => None,
        }
    }
}

