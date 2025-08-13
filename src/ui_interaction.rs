use glam::IVec2;

use crate::atlas::TileAtlas;
use crate::input::Config;
use crate::types::{Building, BuildingKind, Citizen, Resources, WarehouseStore, CitizenState, building_cost, spend_wood};
use crate::ui;
use crate::world::World;

pub fn handle_left_click(
    cursor_xy: IVec2,
    width_i32: i32,
    height_i32: i32,
    config: &Config,
    _atlas: &TileAtlas,
    hovered_tile: Option<IVec2>,
    ui_category: &mut ui::UICategory,
    selected_building: &mut BuildingKind,
    active_building_panel: &mut Option<IVec2>,
    world: &mut World,
    buildings: &mut Vec<Building>,
    buildings_dirty: &mut bool,
    citizens: &mut Vec<Citizen>,
    population: &mut i32,
    warehouses: &mut Vec<WarehouseStore>,
    resources: &mut Resources,
    road_mode: &mut bool,
    path_debug_mode: &mut bool,
    path_sel_a: &mut Option<IVec2>,
    path_sel_b: &mut Option<IVec2>,
    last_path: &mut Option<Vec<IVec2>>,
) -> bool {
    let ui_s = ui::ui_scale(height_i32, config.ui_scale_base);
    let _bar_h = ui::top_panel_height(ui_s);
    // нижняя панель UI
    let bottom_bar_h = ui::bottom_panel_height(ui_s);
    let by0 = height_i32 - bottom_bar_h; let padb = 8 * ui_s; let btn_h = 18 * ui_s;
    // клик по категориям
    let mut cx = padb; let cy = by0 + padb;
    let cats = [
        (ui::UICategory::Housing, b"Housing".as_ref()),
        (ui::UICategory::Storage, b"Storage".as_ref()),
        (ui::UICategory::Forestry, b"Forestry".as_ref()),
        (ui::UICategory::Mining, b"Mining".as_ref()),
        (ui::UICategory::Food, b"Food".as_ref()),
        (ui::UICategory::Logistics, b"Logistics".as_ref()),
    ];
    for (cat, label) in cats.iter() {
        let bw = ui::button_w_for(label, ui_s);
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, cx, cy, bw, btn_h) { *ui_category = *cat; return true; }
        cx += bw + 6 * ui_s;
    }
    // клик по зданиям выбранной категории
    let mut bx = padb; let by2 = cy + btn_h + 6 * ui_s;
    let buildings_for_cat: &[BuildingKind] = match *ui_category {
        ui::UICategory::Housing => &[BuildingKind::House],
        ui::UICategory::Storage => &[BuildingKind::Warehouse],
        ui::UICategory::Forestry => &[BuildingKind::Lumberjack, BuildingKind::Forester],
        ui::UICategory::Mining => &[BuildingKind::StoneQuarry, BuildingKind::ClayPit, BuildingKind::IronMine, BuildingKind::Kiln],
        ui::UICategory::Food => &[BuildingKind::WheatField, BuildingKind::Mill, BuildingKind::Bakery, BuildingKind::Fishery],
        ui::UICategory::Logistics => &[],
    };
    for &bk in buildings_for_cat.iter() {
        let label = match bk {
            BuildingKind::Lumberjack => b"Lumberjack".as_ref(),
            BuildingKind::House => b"House".as_ref(),
            BuildingKind::Warehouse => b"Warehouse".as_ref(),
            BuildingKind::Forester => b"Forester".as_ref(),
            BuildingKind::StoneQuarry => b"Quarry".as_ref(),
            BuildingKind::ClayPit => b"Clay Pit".as_ref(),
            BuildingKind::Kiln => b"Kiln".as_ref(),
            BuildingKind::WheatField => b"Wheat Field".as_ref(),
            BuildingKind::Mill => b"Mill".as_ref(),
            BuildingKind::Bakery => b"Bakery".as_ref(),
            BuildingKind::Fishery => b"Fishery".as_ref(),
            BuildingKind::IronMine => b"Iron Mine".as_ref(),
            BuildingKind::Smelter => b"Smelter".as_ref(),
        };
        let bw = ui::button_w_for(label, ui_s);
        if bx + bw > width_i32 - padb { break; }
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, bx, by2, bw, btn_h) { *selected_building = bk; return true; }
        bx += bw + 6 * ui_s;
    }

    // обработка клика по панели здания (+/-) — только если панель активна
    if let Some(p) = *active_building_panel {
        let panel = ui::layout_building_panel(width_i32, height_i32, ui_s);
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, panel.minus_x, panel.minus_y, panel.minus_w, panel.minus_h) {
            if let Some(b) = buildings.iter_mut().find(|bb| bb.pos == p) { b.workers_target = (b.workers_target - 1).max(0); }
            return true;
        }
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, panel.plus_x, panel.plus_y, panel.plus_w, panel.plus_h) {
            if let Some(b) = buildings.iter_mut().find(|bb| bb.pos == p) { b.workers_target = (b.workers_target + 1).min(9); }
            return true;
        }
    }

    if let Some(tp) = hovered_tile {
        // клик по зданию — открыть/закрыть панель здания
        if let Some(bh) = buildings.iter().find(|bb| bb.pos == tp) {
            *active_building_panel = match *active_building_panel { Some(cur) if cur == bh.pos => None, _ => Some(bh.pos) };
            return true;
        }
        // режим дороги
        if *road_mode { let on = !world.is_road(tp); world.set_road(tp, on); return true; }
        // режим отладки пути
        if *path_debug_mode {
            match (*path_sel_a, *path_sel_b) {
                (None, _) => { *path_sel_a = Some(tp); *last_path=None; }
                (Some(_), None) => { *path_sel_b = Some(tp); }
                (Some(_), Some(_)) => { *path_sel_a = Some(tp); *path_sel_b=None; *last_path=None; }
            }
            if let (Some(a), Some(b)) = (*path_sel_a, *path_sel_b) {
                *last_path = crate::path::astar(&world, a, b, 20_000);
            }
            return true;
        }
        // строительство
        let tile_kind = world.get_tile(tp.x, tp.y);
        let mut allowed = !world.is_occupied(tp) && tile_kind != crate::types::TileKind::Water;
        if allowed {
            match *selected_building {
                BuildingKind::Fishery => {
                    const NB: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
                    allowed = NB.iter().any(|(dx,dy)| world.get_tile(tp.x+dx, tp.y+dy) == crate::types::TileKind::Water);
                }
                BuildingKind::WheatField => { allowed = tile_kind == crate::types::TileKind::Grass; }
                BuildingKind::StoneQuarry => { allowed = world.has_stone_deposit(tp); }
                BuildingKind::ClayPit => { allowed = world.has_clay_deposit(tp); }
                BuildingKind::IronMine => { allowed = world.has_iron_deposit(tp); }
                _ => {}
            }
        }
        if allowed {
            let cost = building_cost(*selected_building);
            if crate::types::can_afford_building(warehouses, resources, &cost) {
                let _ = crate::types::spend_building_cost(warehouses, resources, &cost);
                world.occupy(tp);
                let default_workers = match *selected_building { BuildingKind::House | BuildingKind::Warehouse => 0, _ => 1 };
                buildings.push(Building { kind: *selected_building, pos: tp, timer_ms: 0, workers_target: default_workers });
                // если построен склад — зарегистрировать его в списке складов, чтобы заработали доставки
                if *selected_building == BuildingKind::Warehouse {
                    warehouses.push(WarehouseStore { pos: tp, ..Default::default() });
                }
                *buildings_dirty = true;
                if *selected_building == BuildingKind::House {
                    citizens.push(Citizen {
                        pos: tp, target: tp, moving: false, progress: 0.0, carrying_log: false, assigned_job: None,
                        idle_timer_ms: 0, home: tp, workplace: None, state: CitizenState::Idle, work_timer_ms: 0,
                        carrying: None, pending_input: None, path: Vec::new(), path_index: 0, fed_today: true, manual_workplace: false,
                    });
                    *population += 1;
                }
                *active_building_panel = Some(tp);
                return true;
            }
        }
    }
    false
}


