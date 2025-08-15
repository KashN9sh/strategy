use glam::IVec2;

use crate::types::{Building, BuildingKind, Citizen, Resources, WarehouseStore, ResourceKind, FoodPolicy};
use crate::world::World;

pub fn simulate(
    buildings: &mut Vec<Building>,
    _world: &mut World,
    resources: &mut Resources,
    warehouses: &mut Vec<WarehouseStore>,
    dt_ms: i32,
) {
    for b in buildings.iter_mut() {
        b.timer_ms += dt_ms;
        match b.kind {
            BuildingKind::Lumberjack => {}
            BuildingKind::House => {}
            BuildingKind::Warehouse => {}
            BuildingKind::Forester => {}
            BuildingKind::StoneQuarry => {}
            BuildingKind::ClayPit => {}
            BuildingKind::Kiln => {}
            BuildingKind::WheatField => {}
            BuildingKind::Mill => {}
            BuildingKind::Bakery => {}
            BuildingKind::Fishery => {}
            BuildingKind::IronMine => {}
            BuildingKind::Smelter => {}
        }
    }
}

pub fn new_day_feed_and_income(citizens: &mut [Citizen], resources: &mut Resources, warehouses: &mut [WarehouseStore], policy: FoodPolicy) {
    for c in citizens.iter_mut() { c.fed_today = false; }
    for c in citizens.iter_mut() {
        let mut consumed = 0u8; // бит0=bread, бит1=fish
        let mut take_bread = |wss: &mut [WarehouseStore], res: &mut Resources, consumed: &mut u8| {
            for w in wss.iter_mut() { if w.bread > 0 { w.bread -= 1; *consumed = 1; return; } }
            if *consumed == 0 && res.bread > 0 { res.bread -= 1; *consumed = 1; }
        };
        let mut take_fish = |wss: &mut [WarehouseStore], res: &mut Resources, consumed: &mut u8| {
            for w in wss.iter_mut() { if w.fish > 0 { w.fish -= 1; *consumed = 2; return; } }
            if *consumed == 0 && res.fish > 0 { res.fish -= 1; *consumed = 2; }
        };
        match policy {
            FoodPolicy::Balanced => {
                // выбирать более доступный ресурс: сравним на складах+ресурсах
                let total_bread = warehouses.iter().map(|w| w.bread).sum::<i32>() + resources.bread;
                let total_fish = warehouses.iter().map(|w| w.fish).sum::<i32>() + resources.fish;
                if total_bread >= total_fish { take_bread(warehouses, resources, &mut consumed); }
                if consumed == 0 { take_fish(warehouses, resources, &mut consumed); }
                if consumed == 0 { take_bread(warehouses, resources, &mut consumed); }
            }
            FoodPolicy::BreadFirst => { take_bread(warehouses, resources, &mut consumed); if consumed == 0 { take_fish(warehouses, resources, &mut consumed); } }
            FoodPolicy::FishFirst => { take_fish(warehouses, resources, &mut consumed); if consumed == 0 { take_bread(warehouses, resources, &mut consumed); } }
        }
        if consumed != 0 { c.fed_today = true; c.last_food_mask |= consumed; }
    }
}

pub fn economy_new_day(citizens: &mut Vec<Citizen>, resources: &mut Resources, warehouses: &mut [WarehouseStore], buildings: &[Building], tax_rate: f32, cfg: &crate::input::Config, policy: FoodPolicy) -> (i32, i32) {
    // 1) Кормление и фиксация типов еды
    new_day_feed_and_income(citizens, resources, warehouses, policy);
    // 2) Пересчёт счастья
    let has_house_at = |pos: IVec2| -> bool { buildings.iter().any(|b| b.kind == BuildingKind::House && b.pos == pos) };
    let mut happiness_sum: i32 = 0;
    for c in citizens.iter_mut() {
        let mut h: i32 = 50;
        if c.fed_today { h += cfg.happy_feed_bonus; } else { h += cfg.happy_starving_penalty; }
        if c.last_food_mask & 0b11 == 0b11 { h += cfg.happy_variety_bonus; }
        if has_house_at(c.home) { h += cfg.happy_house_bonus; }
        // простой штраф за высокие налоги перенесём в доход
        c.happiness = h.clamp(0, 100) as u8;
        happiness_sum += c.happiness as i32;
        // обнулим маску раз в день (считаем только последний день)
        c.last_food_mask = 0;
    }
    let pop = citizens.len() as i32;
    let happiness_avg = if pop > 0 { happiness_sum as f32 / pop as f32 } else { 50.0 };
    // 3) Налоги (простая формула)
    let base = cfg.tax_income_base;
    let scale = cfg.tax_income_happy_scale;
    // Налог теперь в монетах на жителя в день: tax_rate — уже монеты/чел
    let per_cap = tax_rate.max(0.0);
    let income = (per_cap * (citizens.len() as f32) * (base + scale * (happiness_avg / 100.0))).round() as i32;
    resources.gold += income.max(0);

    // 4) Апкип зданий (простая модель — золотом)
    let mut upkeep: i32 = 0;
    for b in buildings.iter() {
        use BuildingKind::*;
        let u = match b.kind {
            House => cfg.upkeep_house,
            Warehouse => cfg.upkeep_warehouse,
            Lumberjack => cfg.upkeep_lumberjack,
            Forester => cfg.upkeep_forester,
            StoneQuarry => cfg.upkeep_stone_quarry,
            ClayPit => cfg.upkeep_clay_pit,
            IronMine => cfg.upkeep_iron_mine,
            WheatField => cfg.upkeep_wheat_field,
            Mill => cfg.upkeep_mill,
            Bakery => cfg.upkeep_bakery,
            Kiln => cfg.upkeep_kiln,
            Fishery => cfg.upkeep_fishery,
            Smelter => cfg.upkeep_smelter,
        };
        upkeep += u;
    }
    resources.gold -= upkeep.max(0);

    // 5) Простая миграция: если достаточно счастья и есть свободные места в домах — прибывает 1 житель.
    // Если очень низкое счастье — уходит 1 житель.
    use std::collections::HashMap;
    let mut occ: HashMap<IVec2, i32> = HashMap::new();
    for c in citizens.iter() { *occ.entry(c.home).or_insert(0) += 1; }
    let mut free_home: Option<IVec2> = None;
    for b in buildings.iter().filter(|b| b.kind == BuildingKind::House) {
        let used = *occ.get(&b.pos).unwrap_or(&0);
        if used < b.capacity { free_home = Some(b.pos); break; }
    }
    if happiness_avg > cfg.migration_join_threshold {
        if let Some(home) = free_home {
            citizens.push(Citizen {
                pos: home,
                target: home,
                moving: false,
                progress: 0.0,
                carrying_log: false,
                assigned_job: None,
                idle_timer_ms: 0,
                home,
                workplace: None,
                state: crate::types::CitizenState::Idle,
                work_timer_ms: 0,
                carrying: None,
                pending_input: None,
                path: Vec::new(),
                path_index: 0,
                fed_today: true,
                manual_workplace: false,
                happiness: 55,
                last_food_mask: 0,
            });
        }
    } else if happiness_avg < cfg.migration_leave_threshold {
        // Уходит 1 случайный незакреплённый житель
        if let Some(idx) = citizens.iter().position(|c| !c.manual_workplace) {
            citizens.remove(idx);
        }
    }
    (income.max(0), upkeep.max(0))
}

pub fn plan_path(world: &World, c: &mut Citizen, goal: IVec2) {
    c.target = goal;
    if let Some(path) = crate::path::astar(world, c.pos, goal, 50_000) {
        c.path = path;
        c.path_index = 1;
        if c.path_index < c.path.len() {
            c.target = c.path[c.path_index];
            c.moving = true;
            c.progress = 0.0;
        } else {
            c.moving = false;
        }
    } else {
        c.path.clear();
        c.path_index = 0;
        let dx = (goal.x - c.pos.x).signum();
        let dy = (goal.y - c.pos.y).signum();
        let next = if dx != 0 { IVec2::new(c.pos.x + dx, c.pos.y) } else { IVec2::new(c.pos.x, c.pos.y + dy) };
        c.target = next;
        c.moving = true;
        c.progress = 0.0;
    }
}


