use glam::{IVec2, Vec2};
use rand::Rng;
use crate::types::{
    Building, BuildingKind, Citizen, CitizenState, Job, JobKind, LogItem,
    WarehouseStore,
};
use crate::world::World;
use crate::game;
use crate::jobs;
use crate::weather::WeatherSystem;
use crate::game_state::{GameState, Firefly};
use crate::building_production;
use crate::citizen_state;

pub const DAY_LENGTH_MS: f32 = 120_000.0;

/// Главная функция обновления игрового состояния
pub fn update_game_state(game_state: &mut GameState, frame_ms: f32, config: &crate::input::Config) {
    game_state.accumulator_ms += frame_ms;
    game_state.water_anim_time += frame_ms;
    if frame_ms > 0.0 {
        game_state.fps_ema = game_state.fps_ema * 0.9 + (1000.0 / frame_ms) * 0.1;
    }

    let base_step_ms = config.base_step_ms;
    let step_ms = (base_step_ms / game_state.speed_mult.max(0.0001)).max(1.0);

    if !game_state.paused {
        while game_state.accumulator_ms >= step_ms {
            update_game_simulation(
                step_ms,
                &mut game_state.world_clock_ms,
                &mut game_state.prev_is_day_flag,
                &mut game_state.world,
                &mut game_state.buildings,
                &mut game_state.resources,
                &mut game_state.warehouses,
                &mut game_state.citizens,
                &mut game_state.jobs,
                &mut game_state.logs_on_ground,
                &mut game_state.next_job_id,
                &mut game_state.population,
                game_state.tax_rate,
                game_state.food_policy,
                config,
                &game_state.weather_system,
            );
            game_state.accumulator_ms -= step_ms;
            if game_state.accumulator_ms > 10.0 * step_ms {
                game_state.accumulator_ms = 0.0;
                break;
            }
        }
    }
    
    // Обновление погоды и светлячков
    game_state.weather_system.update(frame_ms, &mut game_state.rng);
    update_fireflies(game_state, frame_ms);
}

/// Обновить светлячков для ночного освещения
fn update_fireflies(game_state: &mut GameState, frame_ms: f32) {
    let tt = (game_state.world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
    let angle = tt * std::f32::consts::TAU;
    let daylight = 0.5 - 0.5 * angle.cos();
    let is_night = daylight <= 0.25;
    
    // Таргет количество светлячков зависит от ночи и размера экрана
    let target = if is_night {
        ((game_state.width_i32 * game_state.height_i32) as f32 / 60000.0)
            .round()
            .clamp(10.0, 48.0) as usize
    } else {
        0
    };
    
    // Спавн/удаление
    if game_state.fireflies.len() < target {
        let need = target - game_state.fireflies.len();
        for _ in 0..need {
            let x = game_state.rng.random_range(0.0..game_state.width_i32 as f32);
            let y = game_state.rng.random_range(0.0..game_state.height_i32 as f32);
            let speed = game_state.rng.random_range(8.0..20.0);
            let angle = game_state.rng.random_range(0.0..std::f32::consts::TAU);
            let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
            let phase = game_state.rng.random_range(0.0..std::f32::consts::TAU);
            let life_s = game_state.rng.random_range(6.0..14.0);
            game_state.fireflies.push(Firefly {
                pos: Vec2::new(x, y),
                vel,
                phase,
                life_s,
            });
        }
    } else if game_state.fireflies.len() > target {
        game_state.fireflies.truncate(target);
    }
    
    // Дрейф и границы
    let dt = frame_ms / 1000.0;
    for f in game_state.fireflies.iter_mut() {
        // чуть блуждаем синусом
        let sway = Vec2::new(
            (game_state.water_anim_time * 0.0016 + f.phase).sin(),
            (game_state.water_anim_time * 0.0021 + f.phase * 0.7).cos(),
        ) * 10.0;
        f.pos += (f.vel * 0.25 + sway) * dt;
        
        // обруливаем края мягко
        if f.pos.x < -20.0 {
            f.pos.x = -20.0;
            f.vel.x = f.vel.x.abs();
        }
        if f.pos.y < -20.0 {
            f.pos.y = -20.0;
            f.vel.y = f.vel.y.abs();
        }
        if f.pos.x > game_state.width_i32 as f32 + 20.0 {
            f.pos.x = game_state.width_i32 as f32 + 20.0;
            f.vel.x = -f.vel.x.abs();
        }
        if f.pos.y > game_state.height_i32 as f32 + 20.0 {
            f.pos.y = game_state.height_i32 as f32 + 20.0;
            f.vel.y = -f.vel.y.abs();
        }
        f.life_s -= dt;
    }
    game_state.fireflies.retain(|f| f.life_s > 0.0);
}

/// Обновить игровую симуляцию на один шаг
pub fn update_game_simulation(
    step_ms: f32,
    world_clock_ms: &mut f32,
    prev_is_day_flag: &mut bool,
    world: &mut World,
    buildings: &mut Vec<Building>,
    resources: &mut crate::types::Resources,
    warehouses: &mut Vec<WarehouseStore>,
    citizens: &mut Vec<Citizen>,
    jobs: &mut Vec<Job>,
    logs_on_ground: &mut Vec<LogItem>,
    next_job_id: &mut u64,
    _population: &mut i32,
    tax_rate: f32,
    food_policy: crate::types::FoodPolicy,
    config: &crate::input::Config,
    weather_system: &WeatherSystem,
) {
    // Подтянем готовые чанки перед генерацией задач
    world.integrate_ready_chunks();
    game::simulate(buildings, world, resources, warehouses, step_ms as i32);
    world.grow_trees(step_ms as i32);
    *world_clock_ms = (*world_clock_ms + step_ms) % DAY_LENGTH_MS;
    
    // Обновляем область строительства на основе населения
    world.update_exploration_by_population(buildings, *_population);

    // День/ночь
    let is_day = is_daytime(*world_clock_ms);
    
    // На рассвете (переход ночь→день) — кормление и доход
    if !*prev_is_day_flag && is_day {
        let _ = game::economy_new_day(
            citizens,
            resources,
            warehouses,
            buildings,
            tax_rate,
            config,
            food_policy,
        );
    }
    *prev_is_day_flag = is_day;

    // Ночная рутина: идём домой и спим, сбрасываем работу
    // Используем State Pattern для управления состояниями
    if !is_day {
        citizen_state::handle_night_routine_with_states(
            citizens,
            world,
            buildings,
            jobs,
            is_day,
        );
    } else {
        // Утро: разбудить спящих и отменить возвращение домой
        citizen_state::handle_dawn_routine_with_states(
            citizens,
            world,
            buildings,
            jobs,
            is_day,
        );
    }

    // Дневная рутина рабочих по зданиям
    if is_day {
        assign_workers_to_buildings(citizens, buildings, world);
        adjust_workers_count(citizens, buildings);
    }

    // Генерация и обработка задач
    if is_day {
        generate_lumberjack_jobs(buildings, jobs, next_job_id, world, citizens);
        generate_haul_jobs(jobs, logs_on_ground, warehouses, next_job_id);
        jobs::assign_jobs_nearest_worker(citizens, jobs, world, buildings);
        jobs::process_jobs(
            citizens,
            jobs,
            logs_on_ground,
            warehouses,
            resources,
            buildings,
            world,
            next_job_id,
        );
    }

    // Перемещение жителей по пути
    update_citizen_movement(step_ms, citizens, world);

    // Производство при работе у здания
    if is_day {
        update_production(
            step_ms,
            citizens,
            buildings,
            warehouses,
            world,
            weather_system,
            config,
        );
    }
}

/// Проверить, сейчас день или ночь
fn is_daytime(world_clock_ms: f32) -> bool {
    let t = (world_clock_ms / DAY_LENGTH_MS).clamp(0.0, 1.0);
    let angle = t * std::f32::consts::TAU;
    let daylight = 0.5 - 0.5 * angle.cos();
    daylight > 0.25
}


/// Назначить рабочих на здания
fn assign_workers_to_buildings(
    citizens: &mut Vec<Citizen>,
    buildings: &Vec<Building>,
    world: &World,
) {
    for b in buildings.iter() {
        match b.kind {
            BuildingKind::House | BuildingKind::Warehouse => {}
            _ => {
                // считаем сколько уже назначено на это здание
                let current = citizens
                    .iter()
                    .filter(|c| c.workplace == Some(b.pos))
                    .count() as i32;
                if current >= b.workers_target {
                    continue;
                }
                if let Some((ci, _)) = citizens
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| {
                        matches!(c.state, CitizenState::Idle | CitizenState::Sleeping)
                            && !c.moving
                            && !c.manual_workplace
                    })
                    .min_by_key(|(_, c)| (c.pos.x - b.pos.x).abs() + (c.pos.y - b.pos.y).abs())
                {
                    let c = &mut citizens[ci];
                    if matches!(c.state, CitizenState::Sleeping) && c.pos != c.home {
                        continue;
                    }
                    c.workplace = Some(b.pos);
                    c.target = b.pos;
                    game::plan_path(world, c, b.pos);
                    c.moving = true;
                    c.progress = 0.0;
                    c.state = CitizenState::GoingToWork;
                }
            }
        }
    }
}

/// Скорректировать количество рабочих на зданиях
fn adjust_workers_count(citizens: &mut Vec<Citizen>, buildings: &Vec<Building>) {
    for b in buildings.iter() {
        if matches!(b.kind, BuildingKind::House | BuildingKind::Warehouse) {
            continue;
        }
        let mut assigned: Vec<usize> = citizens
            .iter()
            .enumerate()
            .filter(|(_, c)| c.workplace == Some(b.pos) && !c.manual_workplace)
            .map(|(i, _)| i)
            .collect();
        let over = (assigned.len() as i32 - b.workers_target).max(0) as usize;
        if over > 0 {
            // снимем часть: берём тех, кто дальше всего от здания
            assigned.sort_by_key(|&i| {
                let c = &citizens[i];
                (c.pos.x - b.pos.x).abs() + (c.pos.y - b.pos.y).abs()
            });
            for &i in assigned.iter().rev().take(over) {
                let c = &mut citizens[i];
                c.workplace = None;
                if matches!(c.state, CitizenState::GoingToWork | CitizenState::Working) {
                    c.state = CitizenState::Idle;
                    c.moving = false;
                }
            }
        }
    }
}

/// Генерировать задачи для лесорубок
fn generate_lumberjack_jobs(
    buildings: &Vec<Building>,
    jobs: &mut Vec<Job>,
    next_job_id: &mut u64,
    world: &World,
    citizens: &Vec<Citizen>,
) {
    for b in buildings.iter() {
        if b.kind != BuildingKind::Lumberjack {
            continue;
        }
        // сколько работников закреплено на этой лесорубке
        let workers_here = citizens
            .iter()
            .filter(|c| c.workplace == Some(b.pos) && c.fed_today)
            .count() as i32;
        if workers_here <= 0 {
            continue;
        }
        // лимит задач = работников_here; считаем только Chop-задачи рядом
        let active_tasks_here = jobs
            .iter()
            .filter(|j| {
                match j.kind {
                    JobKind::ChopWood { pos } => {
                        (pos.x - b.pos.x).abs() + (pos.y - b.pos.y).abs() <= 48
                    }
                    _ => false,
                }
            })
            .count() as i32;
        if active_tasks_here >= workers_here {
            continue;
        }
        // ищем ближайшее зрелое дерево
        let search = |rad: i32| -> Option<IVec2> {
            let mut best: Option<(i32, IVec2)> = None;
            for dy in -rad..=rad {
                for dx in -rad..=rad {
                    let np = IVec2::new(b.pos.x + dx, b.pos.y + dy);
                    if matches!(world.tree_stage(np), Some(2)) {
                        let d = dx.abs() + dy.abs();
                        if best.map(|(bd, _)| d < bd).unwrap_or(true) {
                            best = Some((d, np));
                        }
                    }
                }
            }
            best.map(|(_, p)| p)
        };
        if let Some(np) = search(24)
            .or_else(|| search(32))
            .or_else(|| search(48))
            .or_else(|| search(64))
        {
            let already = jobs.iter().any(|j| {
                match j.kind {
                    JobKind::ChopWood { pos } => pos == np,
                    JobKind::HaulWood { from, .. } => from == np,
                }
            });
            if !already {
                let id = *next_job_id;
                *next_job_id += 1;
                jobs.push(Job {
                    id,
                    kind: JobKind::ChopWood { pos: np },
                    taken: false,
                    done: false,
                });
            }
        }
    }
}

/// Генерировать задачи на перенос поленьев
fn generate_haul_jobs(
    jobs: &mut Vec<Job>,
    logs_on_ground: &Vec<LogItem>,
    warehouses: &Vec<WarehouseStore>,
    next_job_id: &mut u64,
) {
    if warehouses.is_empty() {
        return;
    }
    for li in logs_on_ground.iter() {
        if li.carried {
            continue;
        }
        let already = jobs.iter().any(|j| {
            match j.kind {
                JobKind::HaulWood { from, .. } => from == li.pos,
                _ => false,
            }
        });
        if !already {
            if let Some(dst) = crate::types::find_nearest_warehouse(warehouses, li.pos) {
                let id = *next_job_id;
                *next_job_id += 1;
                jobs.push(Job {
                    id,
                    kind: JobKind::HaulWood { from: li.pos, to: dst },
                    taken: false,
                    done: false,
                });
            }
        }
    }
}

/// Обновить движение граждан
fn update_citizen_movement(step_ms: f32, citizens: &mut Vec<Citizen>, world: &mut World) {
    for c in citizens.iter_mut() {
        if !c.moving {
            c.idle_timer_ms += step_ms as i32;
            c.progress = 0.0;
            // если стоим > 5 секунд с назначенной задачей — сбросим её
            if c.idle_timer_ms > 5000 {
                c.assigned_job = None;
                c.carrying_log = false;
                c.idle_timer_ms = 0;
            }
            // смена состояний при прибытии
            handle_arrival_state(c, world);
        } else {
            c.idle_timer_ms = 0;
            update_movement(step_ms, c, world);
        }
    }
}

/// Обработать состояние граждан при прибытии
fn handle_arrival_state(c: &mut Citizen, world: &mut World) {
    match c.state {
        CitizenState::Idle => {
            // Если у гражданина есть рабочее место
            if let Some(workplace) = c.workplace {
                if c.pos == workplace {
                    // Если уже на рабочем месте и накормлен, начинаем работать
                    if c.fed_today {
                        c.state = CitizenState::Working;
                    }
                } else if !c.moving {
                    // Если не на рабочем месте, идем туда
                    crate::game::plan_path(world, c, workplace);
                    c.state = CitizenState::GoingToWork;
                }
            }
        }
        CitizenState::GoingToWork => {
            // Проверяем, достигли ли рабочего места
            if let Some(workplace) = c.workplace {
                if c.pos == workplace {
                    // Не пускаем работать, если не накормлен
                    if c.fed_today {
                        c.state = CitizenState::Working;
                    } else {
                        c.state = CitizenState::Idle;
                    }
                }
            }
        }
        CitizenState::GoingHome => {
            if c.pos == c.home {
                c.state = CitizenState::Sleeping;
            }
        }
        CitizenState::GoingToDeposit => {
            // Обрабатывается через jobs::process_jobs при достижении цели
            // Здесь просто проверяем, что позиция достигнута
        }
        CitizenState::GoingToFetch => {
            // Обрабатывается через jobs::process_jobs при достижении цели
            // Здесь просто проверяем, что позиция достигнута
        }
        CitizenState::Sleeping => {}
        CitizenState::Working => {
            // Если гражданин в Working, но не на рабочем месте, возвращаемся в Idle
            if let Some(workplace) = c.workplace {
                if c.pos != workplace && !c.moving {
                    c.state = CitizenState::Idle;
                }
            } else {
                // Если рабочее место потеряно, возвращаемся в Idle
                c.state = CitizenState::Idle;
            }
        }
    }
}

/// Обновить движение гражданина по пути
fn update_movement(step_ms: f32, c: &mut Citizen, world: &mut World) {
    // если дорога пустая — идём к следующей точке пути
    if c.pos == c.target {
        // достигнута вершина пути
        if c.path_index + 1 < c.path.len() {
            c.path_index += 1;
            c.target = c.path[c.path_index];
            c.progress = 0.0;
        } else {
            c.moving = false;
            c.progress = 0.0;
        }
    } else {
        // запрет: без моста нельзя идти в воду
        {
            use crate::types::TileKind::*;
            let k = world.get_tile(c.target.x, c.target.y);
            if matches!(k, Water) && !world.is_road(c.target) {
                c.moving = false;
                c.progress = 0.0;
                c.path.clear();
                return;
            }
        }
        // скорость шага зависит от целевой клетки
        let step_time_ms: f32 = if world.is_road(c.target) {
            300.0
        } else {
            use crate::types::TileKind::*;
            match world.get_tile(c.target.x, c.target.y) {
                Grass => 450.0,
                Forest => 600.0,
                Water => 300.0,
            }
        };
        c.progress += (step_ms / step_time_ms) as f32;
        if c.progress >= 1.0 {
            c.pos = c.target;
            c.progress = 0.0;
        }
    }
}

/// Обновить производство в зданиях
fn update_production(
    step_ms: f32,
    citizens: &mut Vec<Citizen>,
    buildings: &Vec<Building>,
    warehouses: &mut Vec<WarehouseStore>,
    world: &mut World,
    weather_system: &WeatherSystem,
    config: &crate::input::Config,
) {
    for c in citizens.iter_mut() {
        if !matches!(c.state, CitizenState::Working) {
            continue;
        }
        if !c.fed_today {
            c.state = CitizenState::Idle;
            continue;
        }
        let Some(wp) = c.workplace else {
            continue;
        };
        if c.pos != wp {
            continue;
        }
        let Some(b) = buildings.iter().find(|b| b.pos == wp) else {
            continue;
        };

        c.work_timer_ms += step_ms as i32;
        
        // модификатор погоды на скорость циклов производства
        let wmul = {
            let w = game::production_weather_wmul(weather_system.current(), b.kind);
            use crate::types::BiomeKind::*;
            let bm = world.biome(b.pos);
            let bmul = match (bm, b.kind) {
                (Swamp, BuildingKind::Lumberjack) => config.biome_swamp_lumberjack_wmul,
                (Rocky, BuildingKind::StoneQuarry) => config.biome_rocky_stone_wmul,
                _ => 1.00,
            };
            w * bmul
        };

        handle_building_production(c, b, warehouses, world, config, wmul, step_ms);
    }
}

/// Обработать производство конкретного здания
/// Теперь использует Strategy Pattern для разделения логики разных типов зданий
fn handle_building_production(
    c: &mut Citizen,
    b: &Building,
    warehouses: &mut Vec<WarehouseStore>,
    world: &mut World,
    config: &crate::input::Config,
    wmul: f32,
    step_ms: f32,
) {
    let strategy = building_production::create_production_strategy(b.kind);
    strategy.process_production(c, b, warehouses, world, config, wmul, step_ms);
}

