use glam::IVec2;

use crate::atlas::TileAtlas;
use crate::input::Config;
use crate::types::{Building, BuildingKind, Citizen, Resources, WarehouseStore, CitizenState, building_cost, spend_wood};
use crate::ui;
use crate::types::FoodPolicy;
use crate::world::World;

/// Проверка возможности размещения здания указанного типа в клетке `tp`.
pub fn building_allowed_at(world: &mut World, kind: BuildingKind, tp: IVec2) -> bool {
    let tile_kind = world.get_tile(tp.x, tp.y);
    let mut allowed = !world.is_occupied(tp) && tile_kind != crate::types::TileKind::Water;
    if allowed {
        match kind {
            BuildingKind::Fishery => {
                // Требуем: клетка суши и не занята, и хотя бы один из 8 соседей — вода
                const NB8: [(i32,i32);8] = [(1,0),(-1,0),(0,1),(0,-1),(1,1),(1,-1),(-1,1),(-1,-1)];
                let near_water = NB8.iter().any(|(dx,dy)| world.get_tile(tp.x + 1 + dx, tp.y + 1 + dy) == crate::types::TileKind::Water);
                allowed = !world.is_occupied(tp) && near_water;
            }
            BuildingKind::WheatField => { allowed = tile_kind == crate::types::TileKind::Grass; }
            BuildingKind::StoneQuarry => { allowed = world.has_stone_deposit(tp + IVec2::new(1, 1)); }
            BuildingKind::ClayPit => { allowed = world.has_clay_deposit(tp + IVec2::new(1, 1)); }
            BuildingKind::IronMine => { allowed = world.has_iron_deposit(tp + IVec2::new(1, 1)); }
            _ => {}
        }
    }
    allowed
}

pub fn handle_left_click(
    cursor_xy: IVec2,
    width_i32: i32,
    height_i32: i32,
    config: &Config,
    _atlas: &TileAtlas,
    hovered_tile: Option<IVec2>,
    ui_category: &mut ui::UICategory,
    ui_tab: &mut ui::UITab,
    tax_rate: &mut f32,
    food_policy: &mut FoodPolicy,
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
    // Миникарта +/- в правом нижнем углу (совпадает с координатами в ui::draw_minimap_widget)
    {
        let s = ui_s; let pad = ui::ui_pad(s); let base_cell = 2 * s; let base_w_tiles = 96; let base_h_tiles = 64;
        let widget_w = base_w_tiles * base_cell; let widget_h = base_h_tiles * base_cell;
        let x = width_i32 - pad - widget_w; let y = height_i32 - bottom_bar_h - pad - widget_h;
        let btn_h = ui::ui_item_h(s); let btn_w = ui::button_w_for(b"+", s); let gap = ui::ui_gap(s);
        let plus_x = x - (btn_w + gap); let plus_y = y;
        let minus_x = plus_x; let minus_y = plus_y + btn_h + gap;
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, minus_x, minus_y, btn_w, btn_h) {
            // уменьшить масштаб миникарты (не ниже 1px)
            use std::sync::atomic::Ordering;
            let cur = super::MINIMAP_CELL_PX.load(Ordering::Relaxed);
            super::MINIMAP_CELL_PX.store((cur - s.max(1)).max(1), Ordering::Relaxed);
            return true;
        }
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, plus_x, plus_y, btn_w, btn_h) {
            use std::sync::atomic::Ordering;
            let cur = super::MINIMAP_CELL_PX.load(Ordering::Relaxed);
            super::MINIMAP_CELL_PX.store(cur + s.max(1), Ordering::Relaxed);
            return true;
        }
    }

    // Вкладки
    let s = ui_s; let padb = 8 * s; let btn_h = 18 * s; let by0 = height_i32 - bottom_bar_h;
    let build_w = ui::button_w_for(b"Build", s); let econ_w = ui::button_w_for(b"Economy", s);
    if ui::point_in_rect(cursor_xy.x, cursor_xy.y, padb, by0 + padb, build_w, btn_h) { *ui_tab = ui::UITab::Build; return true; }
    if ui::point_in_rect(cursor_xy.x, cursor_xy.y, padb + build_w + 6 * s, by0 + padb, econ_w, btn_h) { *ui_tab = ui::UITab::Economy; return true; }

    // Если вкладка Economy — клики по её контролам
    if *ui_tab == ui::UITab::Economy {
        let lay = ui::layout_economy_panel(width_i32, height_i32, s);
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, lay.tax_minus_x, lay.tax_minus_y, lay.tax_minus_w, lay.tax_minus_h) { *tax_rate = (*tax_rate - config.tax_step).max(config.tax_min); return true; }
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, lay.tax_plus_x, lay.tax_plus_y, lay.tax_plus_w, lay.tax_plus_h) { *tax_rate = (*tax_rate + config.tax_step).min(config.tax_max); return true; }
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, lay.policy_bal_x, lay.policy_bal_y, lay.policy_bal_w, lay.policy_bal_h) { *food_policy = FoodPolicy::Balanced; return true; }
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, lay.policy_bread_x, lay.policy_bread_y, lay.policy_bread_w, lay.policy_bread_h) { *food_policy = FoodPolicy::BreadFirst; return true; }
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, lay.policy_fish_x, lay.policy_fish_y, lay.policy_fish_w, lay.policy_fish_h) { *food_policy = FoodPolicy::FishFirst; return true; }
    }

    // клик по категориям (Build): 2-я строка, с переносом
    let cats = [
        (ui::UICategory::Housing, b"Housing".as_ref()),
        (ui::UICategory::Storage, b"Storage".as_ref()),
        (ui::UICategory::Forestry, b"Forestry".as_ref()),
        (ui::UICategory::Mining, b"Mining".as_ref()),
        (ui::UICategory::Food, b"Food".as_ref()),
        (ui::UICategory::Logistics, b"Logistics".as_ref()),
    ];
    let row_y = [by0 + padb + btn_h + 6 * s, by0 + padb + (btn_h + 6 * s) * 2];
    let mut row: usize = 0; let mut cx = padb;
    for (cat, label) in cats.iter() {
        let bw = ui::button_w_for(label, s);
        if cx + bw > width_i32 - padb { row = (row + 1).min(row_y.len()-1); cx = padb; }
        let y = row_y[row];
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, cx, y, bw, btn_h) { *ui_category = *cat; return true; }
        cx += bw + 6 * s;
    }
    // клик по зданиям выбранной категории — 3-я строка
    let mut bx = padb; let by2 = by0 + padb + (btn_h + 6 * s) * 2;
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

    // обработка клика по панели здания (+/-/Demolish) — только если панель активна
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
        // Снос
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, panel.dem_x, panel.dem_y, panel.dem_w, panel.dem_h) {
            if let Some(idx) = buildings.iter().position(|bb| bb.pos == p) {
                let b = buildings.remove(idx);
                // освободить клетку
                world.occupied.remove(&(p.x, p.y));
                // вернуть 50% стоимости и 50% накопленных ресурсов (если применимо)
                let cost = crate::types::building_cost(b.kind);
                // Возврат половины стоимости
                resources.wood += (cost.wood as f32 * 0.5).round() as i32;
                resources.gold += (cost.gold as f32 * 0.5).round() as i32;
                // Возврат половины внутреннего «содержимого» здания
                match b.kind {
                    BuildingKind::House => { /* нет ресурсов */ }
                    BuildingKind::Warehouse => {
                        // Найти склад по позиции и вытащить его содержимое
                        if let Some(iw) = warehouses.iter().position(|w| w.pos == p) {
                            let w = warehouses.remove(iw);
                            resources.wood += (w.wood as f32 * 0.5).round() as i32;
                            resources.stone += (w.stone as f32 * 0.5).round() as i32;
                            resources.clay += (w.clay as f32 * 0.5).round() as i32;
                            resources.bricks += (w.bricks as f32 * 0.5).round() as i32;
                            resources.wheat += (w.wheat as f32 * 0.5).round() as i32;
                            resources.flour += (w.flour as f32 * 0.5).round() as i32;
                            resources.bread += (w.bread as f32 * 0.5).round() as i32;
                            resources.fish += (w.fish as f32 * 0.5).round() as i32;
                            resources.gold += (w.gold as f32 * 0.5).round() as i32;
                            resources.iron_ore += (w.iron_ore as f32 * 0.5).round() as i32;
                            resources.iron_ingots += (w.iron_ingots as f32 * 0.5).round() as i32;
                        }
                    }
                    _ => { /* производственные не хранят, возвращаем только стоимость */ }
                }
                *buildings_dirty = true;
                *active_building_panel = None;
                return true;
            }
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
        let allowed = building_allowed_at(world, *selected_building, tp);
        if allowed {
            let cost = building_cost(*selected_building);
            if crate::types::can_afford_building(warehouses, resources, &cost) {
                let _ = crate::types::spend_building_cost(warehouses, resources, &cost);
                world.occupy(tp);
                let default_workers = match *selected_building { BuildingKind::House | BuildingKind::Warehouse => 0, _ => 1 };
                let capacity = match *selected_building { BuildingKind::House => 2, _ => 0 };
                buildings.push(Building { kind: *selected_building, pos: tp, timer_ms: 0, workers_target: default_workers, capacity });
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
                        happiness: 50, last_food_mask: 0,
                    });
                    *population += 1;
                }
                // Не открываем панель автоматически после постройки
                return true;
            }
        }
    }
    false
}


