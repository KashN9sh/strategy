use glam::IVec2;

use crate::types::{Building, BuildingKind, Citizen, Resources, WarehouseStore, ResourceKind};
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

pub fn new_day_feed_and_income(citizens: &mut [Citizen], resources: &mut Resources, warehouses: &mut [WarehouseStore]) {
    for c in citizens.iter_mut() { c.fed_today = false; }
    for c in citizens.iter_mut() {
        let mut consumed = false;
        for w in warehouses.iter_mut() { if w.bread > 0 { w.bread -= 1; consumed = true; break; } }
        if !consumed { for w in warehouses.iter_mut() { if w.fish > 0 { w.fish -= 1; consumed = true; break; } } }
        if !consumed {
            if resources.bread > 0 { resources.bread -= 1; consumed = true; }
            else if resources.fish > 0 { resources.fish -= 1; consumed = true; }
        }
        if consumed { c.fed_today = true; resources.gold += 1; }
    }
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


