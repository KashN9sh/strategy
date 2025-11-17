use glam::IVec2;

use crate::atlas::TileAtlas;
use crate::input::Config;
use crate::types::{Building, BuildingKind, Citizen, Resources, WarehouseStore, CitizenState, building_cost};
use crate::ui;
use crate::types::FoodPolicy;
use crate::world::World;
use crate::research::ResearchSystem;

/// Проверка возможности размещения здания указанного типа в клетке `tp`.
pub fn building_allowed_at(world: &mut World, kind: BuildingKind, tp: IVec2) -> bool {
    // Проверяем, разблокирован ли тайл для строительства
    if !world.is_explored(tp) {
        return false;
    }
    let tile_kind = world.get_tile(tp.x, tp.y);
    let mut allowed = !world.is_occupied(tp) && tile_kind != crate::types::TileKind::Water;
    if allowed {
        match kind {
            BuildingKind::Fishery => {
                // Требуем: клетка суши и не занята, и хотя бы один из 8 соседей — вода
                const NB8: [(i32,i32);8] = [(1,0),(-1,0),(0,1),(0,-1),(1,1),(1,-1),(-1,1),(-1,-1)];
                let near_water = NB8.iter().any(|(dx,dy)| world.get_tile(tp.x + dx, tp.y + dy) == crate::types::TileKind::Water);
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
    selected_building: &mut Option<BuildingKind>,
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
    show_deposits: &mut bool,
    research_system: &mut ResearchSystem,
    show_research_tree: &mut bool,
) -> bool {
    let ui_s = ui::ui_scale(height_i32, config.ui_scale_base);
    let _bar_h = ui::top_panel_height(ui_s);
    // нижняя панель UI
    let bottom_bar_h = ui::bottom_panel_height(ui_s);
    let _by0 = height_i32 - bottom_bar_h; let _padb = 8 * ui_s; let _btn_h = 18 * ui_s;
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

    // Кнопка депозитов
    let deposits_w = ui::button_w_for(b"Deposits", s).max(80);
    let deposits_x = padb + build_w + 6 * s + econ_w + 6 * s;
    let deposits_y = by0 + padb;
    if ui::point_in_rect(cursor_xy.x, cursor_xy.y, deposits_x, deposits_y, deposits_w, btn_h) { 
        *show_deposits = !*show_deposits; 
        return true; 
    }

    // Кнопка Research (только если есть лаборатория)
    if research_system.has_research_lab {
        let research_w = ui::button_w_for(b"Research (T)", s).max(100);
        let research_x = deposits_x + deposits_w + 6 * s;
        let research_y = by0 + padb;
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, research_x, research_y, research_w, btn_h) { 
            *show_research_tree = !*show_research_tree; 
            return true; 
        }
    }

    // Если вкладка Economy — клики по её контролам (динамический расчет)
    if *ui_tab == ui::UITab::Economy {
        let control_y = by0 + padb + btn_h + 6 * ui_s; // вторая строка (контролы налогов)
        let policy_y = control_y + btn_h + 6 * ui_s; // третья строка (политика еды)
        
        // Динамический расчет координат для налогов (как в ui_gpu.rs)
        let mut current_x = padb;
        let tax_label_w = ((3 * 4 * 2 * ui_s) + 12).max(40); // "TAX"
        current_x += tax_label_w + 6 * ui_s;
        
        let taxp = (*tax_rate * 100.0).round().clamp(0.0, 100.0) as u32;
        let tax_num_w = ((taxp.to_string().len() as i32 * 4 * 2 * ui_s) + 12).max(60);
        current_x += tax_num_w + 6 * ui_s;
        
        // Кнопки изменения налогов
        let minus_btn_w = ((1 * 4 * 2 * ui_s) + 12).max(30); // "-"
        let plus_btn_w = ((1 * 4 * 2 * ui_s) + 12).max(30); // "+"
        
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, current_x, control_y, minus_btn_w, btn_h) { 
            *tax_rate = (*tax_rate - config.tax_step).max(config.tax_min); 
            return true; 
        }
        current_x += minus_btn_w + 6 * ui_s;
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, current_x, control_y, plus_btn_w, btn_h) { 
            *tax_rate = (*tax_rate + config.tax_step).min(config.tax_max); 
            return true; 
        }
        
        // Динамический расчет координат для политики еды
        current_x = padb;
        let policy_label_w = ((11 * 4 * 2 * ui_s) + 12).max(100); // "FOOD POLICY"
        current_x += policy_label_w + 6 * ui_s;
        
        // Кнопки политики еды
        let food_policies: &[(FoodPolicy, &[u8])] = &[
            (FoodPolicy::Balanced, b"Balanced"),
            (FoodPolicy::BreadFirst, b"Bread"),
            (FoodPolicy::FishFirst, b"Fish"),
        ];
        
        for (policy, label) in food_policies.iter() {
            let btn_w = ((label.len() as i32 * 4 * 2 * ui_s) + 12).max(50);
            if current_x + btn_w > width_i32 - padb {
                break;
            }
            if ui::point_in_rect(cursor_xy.x, cursor_xy.y, current_x, policy_y, btn_w, btn_h) { 
                *food_policy = *policy; 
                return true; 
            }
            current_x += btn_w + 6 * ui_s;
        }
    }

    // клик по категориям (Build): 2-я строка, с переносом
    let cats = [
        (ui::UICategory::Housing, b"Housing".as_ref()),
        (ui::UICategory::Storage, b"Storage".as_ref()),
        (ui::UICategory::Forestry, b"Forestry".as_ref()),
        (ui::UICategory::Mining, b"Mining".as_ref()),
        (ui::UICategory::Food, b"Food".as_ref()),
        (ui::UICategory::Logistics, b"Logistics".as_ref()),
        (ui::UICategory::Research, b"Research".as_ref()),
    ];
    let row_y = [by0 + padb + btn_h + 6 * s, by0 + padb + (btn_h + 6 * s) * 2];
    let mut row: usize = 0; let mut cx = padb;
    for (cat, label) in cats.iter() {
        let bw = ((label.len() as i32) * 4 * 2 * s + 12).max(60); // та же формула, что в ui_gpu.rs
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
        ui::UICategory::Research => &[BuildingKind::ResearchLab],
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
            BuildingKind::ResearchLab => b"Research Lab".as_ref(),
        };
        let bw = ((label.len() as i32) * 4 * 2 * ui_s + 12).max(70); // та же формула, что в ui_gpu.rs
        if bx + bw > width_i32 - padb { break; }
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, bx, by2, bw, btn_h) { *selected_building = Some(bk); return true; }
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
        // строительство (только если здание выбрано)
        if let Some(building_kind) = *selected_building {
            let allowed = building_allowed_at(world, building_kind, tp);
            if allowed {
                // Проверка разблокировки здания через систему исследований
                // ResearchLab всегда разблокирована
                let is_unlocked = building_kind == BuildingKind::ResearchLab 
                    || research_system.is_building_unlocked(building_kind);
                
                if !is_unlocked {
                    // Здание не разблокировано, не строим
                    return true;
                }
                
                let cost = building_cost(building_kind);
                if crate::types::can_afford_building(warehouses, resources, &cost) {
                    let _ = crate::types::spend_building_cost(warehouses, resources, &cost);
                    world.occupy(tp);
                    let default_workers = match building_kind { BuildingKind::House | BuildingKind::Warehouse => 0, _ => 1 };
                    let capacity = match building_kind { BuildingKind::House => 2, _ => 0 };
                    buildings.push(Building { kind: building_kind, pos: tp, timer_ms: 0, workers_target: default_workers, capacity, is_highlighted: false });
                    // если построен склад — зарегистрировать его в списке складов, чтобы заработали доставки
                    if building_kind == BuildingKind::Warehouse {
                        warehouses.push(WarehouseStore { pos: tp, ..Default::default() });
                    }
                    // если построена лаборатория — обновить флаг
                    if building_kind == BuildingKind::ResearchLab {
                        research_system.has_research_lab = true;
                    }
                    *buildings_dirty = true;
                    if building_kind == BuildingKind::House {
                        citizens.push(Citizen {
                            pos: tp, target: tp, moving: false, progress: 0.0, carrying_log: false, assigned_job: None,
                            idle_timer_ms: 0, home: tp, workplace: None, state: CitizenState::Idle, work_timer_ms: 0,
                            carrying: None, pending_input: None, path: Vec::new(), path_index: 0, fed_today: true, manual_workplace: false,
                            happiness: 50, last_food_mask: 0,
                        });
                        *population += 1;
                        // Разблокируем область вокруг нового дома
                        let base_radius = 8;
                        let radius = base_radius + (*population / 5).min(20);
                        world.explore_area(tp, radius);
                    }
                    // Отменяем выбор здания после постройки
                    *selected_building = None;
                    // Не открываем панель автоматически после постройки
                    return true;
                }
            }
        }
    }
    false
}

/// Определяет, на какую кнопку наведен курсор (для тултипов)
pub fn get_hovered_button(
    cursor_xy: IVec2,
    width_i32: i32,
    height_i32: i32,
    config: &Config,
    ui_category: ui::UICategory,
    ui_tab: ui::UITab,
    paused: bool,
    _speed_mult: f32,
    tax_rate: f32,
    _food_policy: FoodPolicy,
) -> Option<&'static str> {
    let ui_s = ui::ui_scale(height_i32, config.ui_scale_base);
    let bottom_bar_h = ui::bottom_panel_height(ui_s);
    let by0 = height_i32 - bottom_bar_h;
    let padb = 8 * ui_s;
    let btn_h = 18 * ui_s;
    
    // Кнопки управления (пауза, скорость)
    let control_btn_w = ui::button_w_for(b"Pause", ui_s);
    let control_x = width_i32 - padb - control_btn_w * 4 - 6 * ui_s * 3;
    let control_y = by0 + padb;
    
    if ui::point_in_rect(cursor_xy.x, cursor_xy.y, control_x, control_y, control_btn_w, btn_h) {
        return Some(if paused { "Resume" } else { "Pause" });
    }
    
    let speed_btn_w = ui::button_w_for(b"1x", ui_s);
    let speed_x = control_x + control_btn_w + 6 * ui_s;
    if ui::point_in_rect(cursor_xy.x, cursor_xy.y, speed_x, control_y, speed_btn_w, btn_h) {
        return Some("Speed 1x");
    }
    
    let speed2_x = speed_x + speed_btn_w + 6 * ui_s;
    if ui::point_in_rect(cursor_xy.x, cursor_xy.y, speed2_x, control_y, speed_btn_w, btn_h) {
        return Some("Speed 2x");
    }
    
    let speed4_x = speed2_x + speed_btn_w + 6 * ui_s;
    if ui::point_in_rect(cursor_xy.x, cursor_xy.y, speed4_x, control_y, speed_btn_w, btn_h) {
        return Some("Speed 4x");
    }
    
    // Вкладки (с теми же минимальными размерами, что и в ui_gpu.rs)
    let build_w = ui::button_w_for(b"Build", ui_s).max(60);
    let econ_w = ui::button_w_for(b"Economy", ui_s).max(80);
    let build_x = padb;
    let build_y = by0 + padb;
    let econ_x = padb + build_w + 6 * ui_s; // используем масштабированный отступ, как в handle_left_click
    let econ_y = by0 + padb;
    
    if ui::point_in_rect(cursor_xy.x, cursor_xy.y, build_x, build_y, build_w, btn_h) {
        return Some("Build Tab");
    }
    if ui::point_in_rect(cursor_xy.x, cursor_xy.y, econ_x, econ_y, econ_w, btn_h) {
        return Some("Economy Tab");
    }
    
    // Кнопка депозитов
    let deposits_w = ui::button_w_for(b"Deposits", ui_s).max(80);
    let deposits_x = econ_x + econ_w + 6 * ui_s;
    let deposits_y = by0 + padb;
    if ui::point_in_rect(cursor_xy.x, cursor_xy.y, deposits_x, deposits_y, deposits_w, btn_h) {
        return Some("Deposits");
    }
    
    // Если вкладка Economy — контролы экономики (динамический расчет)
    if ui_tab == ui::UITab::Economy {
        let control_y = by0 + padb + btn_h + 6 * ui_s; // вторая строка (контролы налогов)
        let policy_y = control_y + btn_h + 6 * ui_s; // третья строка (политика еды)
        
        // Динамический расчет координат для налогов (как в ui_gpu.rs)
        let mut current_x = padb;
        let tax_label_w = ((3 * 4 * 2 * ui_s) + 12).max(40); // "TAX"
        current_x += tax_label_w + 6 * ui_s;
        
        let taxp = (tax_rate * 100.0).round().clamp(0.0, 100.0) as u32;
        let tax_num_w = ((taxp.to_string().len() as i32 * 4 * 2 * ui_s) + 12).max(60);
        current_x += tax_num_w + 6 * ui_s;
        
        // Кнопки изменения налогов
        let minus_btn_w = ((1 * 4 * 2 * ui_s) + 12).max(30); // "-"
        let plus_btn_w = ((1 * 4 * 2 * ui_s) + 12).max(30); // "+"
        
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, current_x, control_y, minus_btn_w, btn_h) {
            return Some("Decrease Tax");
        }
        current_x += minus_btn_w + 6 * ui_s;
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, current_x, control_y, plus_btn_w, btn_h) {
            return Some("Increase Tax");
        }
        
        // Динамический расчет координат для политики еды
        current_x = padb;
        let policy_label_w = ((11 * 4 * 2 * ui_s) + 12).max(100); // "FOOD POLICY"
        current_x += policy_label_w + 6 * ui_s;
        
        // Кнопки политики еды
        let food_policies: &[(FoodPolicy, &[u8], &str)] = &[
            (FoodPolicy::Balanced, b"Balanced", "Balanced Food Policy"),
            (FoodPolicy::BreadFirst, b"Bread", "Bread First Policy"),
            (FoodPolicy::FishFirst, b"Fish", "Fish First Policy"),
        ];
        
        for (_policy, label, tooltip) in food_policies.iter() {
            let btn_w = ((label.len() as i32 * 4 * 2 * ui_s) + 12).max(50);
            if current_x + btn_w > width_i32 - padb {
                break;
            }
            if ui::point_in_rect(cursor_xy.x, cursor_xy.y, current_x, policy_y, btn_w, btn_h) {
                return Some(tooltip);
            }
            current_x += btn_w + 6 * ui_s;
        }
    }
    
    // Категории зданий
    let cats = [
        (ui::UICategory::Housing, "Housing"),
        (ui::UICategory::Storage, "Storage"),
        (ui::UICategory::Forestry, "Forestry"),
        (ui::UICategory::Mining, "Mining"),
        (ui::UICategory::Food, "Food"),
        (ui::UICategory::Logistics, "Logistics"),
    ];
    let row_y = [by0 + padb + btn_h + 6 * ui_s, by0 + padb + (btn_h + 6 * ui_s) * 2];
    let mut row: usize = 0; let mut cx = padb;
    for (_cat, label) in cats.iter() {
        let bw = ((label.len() as i32) * 4 * 2 * ui_s + 12).max(60); // та же формула, что в ui_gpu.rs
        if cx + bw > width_i32 - padb { row = (row + 1).min(row_y.len()-1); cx = padb; }
        let y = row_y[row];
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, cx, y, bw, btn_h) {
            return Some(label);
        }
        cx += bw + 6 * ui_s;
    }
    
    // Здания выбранной категории
    let mut bx = padb;
    let by2 = by0 + padb + btn_h + 6 * ui_s + btn_h + 6 * ui_s; // две строки с масштабированными отступами
    let buildings_for_cat: &[BuildingKind] = match ui_category {
        ui::UICategory::Housing => &[BuildingKind::House],
        ui::UICategory::Storage => &[BuildingKind::Warehouse],
        ui::UICategory::Forestry => &[BuildingKind::Lumberjack, BuildingKind::Forester],
        ui::UICategory::Mining => &[BuildingKind::StoneQuarry, BuildingKind::ClayPit, BuildingKind::IronMine, BuildingKind::Kiln],
        ui::UICategory::Food => &[BuildingKind::WheatField, BuildingKind::Mill, BuildingKind::Bakery, BuildingKind::Fishery],
        ui::UICategory::Logistics => &[],
        ui::UICategory::Research => &[BuildingKind::ResearchLab],
    };
    for &bk in buildings_for_cat.iter() {
        let label = match bk {
            BuildingKind::Lumberjack => "Lumberjack",
            BuildingKind::House => "House",
            BuildingKind::Warehouse => "Warehouse",
            BuildingKind::Forester => "Forester",
            BuildingKind::StoneQuarry => "Quarry",
            BuildingKind::ClayPit => "Clay Pit",
            BuildingKind::Kiln => "Kiln",
            BuildingKind::WheatField => "Wheat Field",
            BuildingKind::Mill => "Mill",
            BuildingKind::Bakery => "Bakery",
            BuildingKind::Fishery => "Fishery",
            BuildingKind::IronMine => "Iron Mine",
            BuildingKind::Smelter => "Smelter",
            BuildingKind::ResearchLab => "Research Lab",
        };
        let bw = ((label.len() as i32) * 4 * 2 * ui_s + 12).max(70); // та же формула, что в ui_gpu.rs
        if bx + bw > width_i32 - padb { break; }
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, bx, by2, bw, btn_h) {
            return Some(label);
        }
        bx += bw + 6 * ui_s; // используем масштабированный отступ
    }
    
    None
}

/// Определяет, на какой ресурс наведен курсор (для тултипов)
pub fn get_hovered_resource(
    cursor_xy: IVec2,
    _width_i32: i32,
    height_i32: i32,
    config: &Config,
    resources: &Resources,
    total_wood: i32,
    population: i32,
    avg_happiness: f32,
    tax_rate: f32,
    citizens_idle: i32,
    citizens_working: i32,
    citizens_sleeping: i32,
    citizens_hauling: i32,
    citizens_fetching: i32,
) -> Option<&'static str> {
    let ui_s = ui::ui_scale(height_i32, config.ui_scale_base);
    let panel_height = ui::top_panel_height(ui_s);
    let pad = (8 * ui_s) as f32;
    let icon_size = (12 * ui_s) as f32;
    let gap = (6 * ui_s) as f32;
    
    // Проверяем, что курсор в верхней панели
    if cursor_xy.y < panel_height {
        let row1_y = pad;
        let row2_y = row1_y + icon_size + gap;
        let mut x = pad;
        
        // Первая строка: Population, Gold, Happiness, Tax, статусы граждан
        // Population
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, x as i32, row1_y as i32, icon_size as i32, icon_size as i32) {
            return Some("Population");
        }
        x += icon_size + 4.0;
        x += ((population.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * (ui_s as f32)) + gap;
        
        // Gold
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, x as i32, row1_y as i32, icon_size as i32, icon_size as i32) {
            return Some("Gold");
        }
        x += icon_size + 4.0;
        x += ((resources.gold.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * (ui_s as f32)) + gap;
        
        // Happiness
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, x as i32, row1_y as i32, icon_size as i32, icon_size as i32) {
            return Some("Happiness");
        }
        x += icon_size + 4.0;
        let hap = avg_happiness.round().clamp(0.0, 100.0) as u32;
        x += (hap.to_string().len() as f32 * 4.0 * 2.0 * (ui_s as f32)) + gap;
        
        // Tax
        if ui::point_in_rect(cursor_xy.x, cursor_xy.y, x as i32, row1_y as i32, icon_size as i32, icon_size as i32) {
            return Some("Tax");
        }
        x += icon_size + 4.0;
        let taxp = (tax_rate * 100.0).round().clamp(0.0, 100.0) as u32;
        x += (taxp.to_string().len() as f32 * 4.0 * 2.0 * (ui_s as f32)) + gap;
        
        // Citizen status icons
        let stat_icons = [
            ("Idle", citizens_idle),
            ("Working", citizens_working),
            ("Sleeping", citizens_sleeping),
            ("Hauling", citizens_hauling),
            ("Fetching", citizens_fetching),
        ];
        
        for (name, count) in stat_icons {
            if ui::point_in_rect(cursor_xy.x, cursor_xy.y, x as i32, row1_y as i32, icon_size as i32, icon_size as i32) {
                return Some(name);
            }
            x += icon_size + 4.0;
            x += (count.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * (ui_s as f32) + gap;
        }
        
        // Вторая строка: ресурсы
        x = pad;
        let resources_list = [
            ("Wood", total_wood),
            ("Stone", resources.stone),
            ("Clay", resources.clay),
            ("Bricks", resources.bricks),
            ("Wheat", resources.wheat),
            ("Flour", resources.flour),
            ("Bread", resources.bread),
            ("Fish", resources.fish),
            ("Iron Ore", resources.iron_ore),
            ("Iron Ingots", resources.iron_ingots),
        ];
        
        for (name, amount) in resources_list {
            if ui::point_in_rect(cursor_xy.x, cursor_xy.y, x as i32, row2_y as i32, icon_size as i32, icon_size as i32) {
                return Some(name);
            }
            x += icon_size + 4.0;
            x += (amount.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * (ui_s as f32) + gap;
        }
    }
    
    None
}

/// Обработка кликов в окне дерева исследований
/// Возвращает true если нужно закрыть окно
pub fn handle_research_tree_click(
    cursor_xy: IVec2,
    fw: i32,
    fh: i32,
    base_scale_k: f32,
    research_system: &mut crate::research::ResearchSystem,
    warehouses: &mut Vec<crate::types::WarehouseStore>,
    resources: &mut crate::types::Resources,
    scroll_offset: f32,
) -> bool {
    use crate::research::{ResearchKind, ResearchStatus};
    use crate::types;
    
    let s = ui::ui_scale(fh, base_scale_k);
    let scale = s as f32;
    
    // Размеры окна (те же, что в draw_research_tree_gpu)
    let window_w = (fw as f32 * 0.9).max(900.0);
    let window_h = (fh as f32 * 0.85).max(650.0);
    let window_x = (fw as f32 - window_w) / 2.0;
    let window_y = (fh as f32 - window_h) / 2.0;
    
    let pad = (16 * s) as f32;
    
    // Проверка клика на кнопку закрытия (крестик)
    let close_btn_size = (20 * s) as f32;
    let close_btn_x = window_x + window_w - pad - close_btn_size;
    let close_btn_y = window_y + pad;
    
    if ui::point_in_rect(
        cursor_xy.x, cursor_xy.y,
        close_btn_x as i32, close_btn_y as i32,
        close_btn_size as i32, close_btn_size as i32
    ) {
        return true; // Закрыть окно
    }
    let title_height = (24 * s) as f32;
    let info_y = window_y + pad + title_height + 8.0;
    let tree_start_y = info_y + (20 * s) as f32;
    
    let node_w = (110 * s) as f32; // Уменьшенный размер
    let node_h = (60 * s) as f32;  // Уменьшенный размер
    let gap_x = (20 * s) as f32;
    let gap_y = (35 * s) as f32;
    
    // Проверяем клики по узлам
    for &research_kind in ResearchKind::all() {
        let (col, row) = research_kind.tree_position();
        let status = research_system.get_status(research_kind);
        let info = research_kind.info();
        
        // Пропускаем завершенные базовые исследования
        if status == ResearchStatus::Completed && info.days_required == 0 {
            continue;
        }
        
        let node_x = window_x + pad + (col as f32) * (node_w + gap_x);
        let node_y = tree_start_y + (row as f32) * (node_h + gap_y) - scroll_offset;
        
        // Проверка клика
        if ui::point_in_rect(
            cursor_xy.x, cursor_xy.y,
            node_x as i32, node_y as i32,
            node_w as i32, node_h as i32
        ) && status == ResearchStatus::Available {
            // Проверяем ресурсы
            let total_res = types::total_resources(warehouses, resources);
            let can_afford = total_res.wood >= info.cost.wood 
                && total_res.gold >= info.cost.gold
                && total_res.stone >= info.cost.stone;
            
            if can_afford {
                // Списываем ресурсы
                let _ = types::spend_building_cost(warehouses, resources, &info.cost);
                // Начинаем исследование
                research_system.start_research(research_kind);
            }
            
            return true;
        }
    }
    
    false
}

