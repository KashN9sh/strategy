// GPU версия UI модуля
// Использует всю логику из ui.rs, но рендерит через GpuRenderer

use crate::gpu_renderer::GpuRenderer;
use crate::types::{Resources, BuildingKind, FoodPolicy};
use crate::ui::{self, UICategory, UITab};

/// GPU версия draw_ui - использует GpuRenderer вместо CPU frame buffer
pub fn draw_ui_gpu(
    gpu: &mut GpuRenderer,
    fw: i32,
    fh: i32,
    resources: &Resources,
    total_wood: i32,
    population: i32,
    selected: BuildingKind,
    fps: f32,
    speed: f32,
    paused: bool,
    base_scale_k: f32,
    category: UICategory,
    day_progress_01: f32,
    citizens_idle: i32,
    citizens_working: i32,
    citizens_sleeping: i32,
    citizens_hauling: i32,
    citizens_fetching: i32,
    avg_happiness: f32,
    tax_rate: f32,
    ui_tab: UITab,
    food_policy: FoodPolicy,
    weather_label: &[u8],
    weather_icon_col: [f32; 4],
    // Данные для миникарты
    world: &mut crate::world::World,
    buildings: &[crate::types::Building],
    cam_x: f32,
    cam_y: f32,
    cell_size: i32,
    // Данные для тултипов
    cursor_x: f32,
    cursor_y: f32,
    hovered_building: Option<crate::types::Building>,
    hovered_button: Option<&'static str>,
    hovered_resource: Option<&'static str>,
) {
    gpu.clear_ui();
    
    let s = ui::ui_scale(fh, base_scale_k);
    let scale = s as f32;
    
    // === ВЕРХНЯЯ ПАНЕЛЬ ===
    let panel_height = ui::top_panel_height(s) as f32;
    gpu.draw_ui_panel(0.0, 0.0, fw as f32, panel_height);
    
    let pad = (8 * s) as f32;
    let icon_size = (10 * s) as f32;
    
    // Первая строка: Population, Gold, Happiness, Tax, статусы граждан
    let row1_y = pad;
    let gap = (6 * s) as f32;
    let mut x = pad;
    
    // Population
    gpu.draw_ui_resource_icon(x, row1_y, icon_size, [180.0/255.0, 60.0/255.0, 60.0/255.0, 1.0]);
    x += icon_size + 4.0;
    gpu.draw_number(x, row1_y, population.max(0) as u32, [1.0, 1.0, 1.0, 1.0], scale);
    x += ((population.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * scale) + gap;
    
    // Gold
    gpu.draw_ui_resource_icon(x, row1_y, icon_size, [220.0/255.0, 180.0/255.0, 60.0/255.0, 1.0]);
    x += icon_size + 4.0;
    gpu.draw_number(x, row1_y, resources.gold.max(0) as u32, [1.0, 1.0, 1.0, 1.0], scale);
    x += ((resources.gold.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * scale) + gap;
    
    // Happiness
    gpu.draw_ui_resource_icon(x, row1_y, icon_size, [220.0/255.0, 120.0/255.0, 160.0/255.0, 1.0]);
    x += icon_size + 4.0;
    let hap = avg_happiness.round().clamp(0.0, 100.0) as u32;
    gpu.draw_number(x, row1_y, hap, [1.0, 1.0, 1.0, 1.0], scale);
    x += (hap.to_string().len() as f32 * 4.0 * 2.0 * scale) + gap;
    
    // Tax
    gpu.draw_ui_resource_icon(x, row1_y, icon_size, [200.0/255.0, 180.0/255.0, 90.0/255.0, 1.0]);
    x += icon_size + 4.0;
    let taxp = (tax_rate * 100.0).round().clamp(0.0, 100.0) as u32;
    gpu.draw_number(x, row1_y, taxp, [1.0, 1.0, 1.0, 1.0], scale);
    x += (taxp.to_string().len() as f32 * 4.0 * 2.0 * scale) + gap;
    
    // Citizen status icons
    let stat_icons = [
        ([70.0/255.0, 160.0/255.0, 70.0/255.0, 1.0], citizens_idle),
        ([70.0/255.0, 110.0/255.0, 200.0/255.0, 1.0], citizens_working),
        ([120.0/255.0, 120.0/255.0, 120.0/255.0, 1.0], citizens_sleeping),
        ([210.0/255.0, 150.0/255.0, 70.0/255.0, 1.0], citizens_hauling),
        ([80.0/255.0, 200.0/255.0, 200.0/255.0, 1.0], citizens_fetching),
    ];
    
    for (color, count) in stat_icons {
        gpu.draw_ui_resource_icon(x, row1_y, icon_size, color);
        x += icon_size + 4.0;
        gpu.draw_number(x, row1_y, count.max(0) as u32, [1.0, 1.0, 1.0, 1.0], scale);
        x += (count.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * scale + gap;
    }
    
    // Правая часть в одну строку: Weather | Day | Speed | FPS
    let mut right_x = fw as f32 - pad;
    
    // FPS (самый правый)
    let fps_rounded = fps.round() as u32;
    let fps_num_w = (fps_rounded.to_string().len() as f32 * 4.0 * 2.0 * scale);
    right_x -= fps_num_w;
    gpu.draw_number(right_x, row1_y, fps_rounded, [220.0/255.0, 220.0/255.0, 220.0/255.0, 1.0], scale);
    right_x -= 80.0;
    gpu.draw_text(right_x, row1_y, b"FPS", [180.0/255.0, 180.0/255.0, 180.0/255.0, 1.0], scale);
    right_x -= gap;
    
    // Speed (слева от FPS)
    let speed_val = (speed * 10.0).round() as u32;
    let speed_num_w = (speed_val.to_string().len() as f32 * 4.0 * 2.0 * scale);
    right_x -= speed_num_w;
    gpu.draw_number(right_x, row1_y, speed_val, [220.0/255.0, 220.0/255.0, 220.0/255.0, 1.0], scale);
    right_x -= 30.0;
    gpu.draw_text(right_x, row1_y, b"x", [180.0/255.0, 180.0/255.0, 180.0/255.0, 1.0], scale);
    right_x -= gap;
    
    
    // Weather (слева от Speed)
    let weather_text_w = weather_label.len() as f32 * 4.0 * 2.0 * scale;
    right_x -= weather_text_w;
    gpu.draw_text(right_x, row1_y + 2.0, weather_label, [230.0/255.0, 230.0/255.0, 230.0/255.0, 1.0], scale);
    right_x -= icon_size + 4.0;
    gpu.draw_ui_resource_icon(right_x, row1_y, icon_size, weather_icon_col);
    
    // PAUSED (если активна, вторая строка)
    if paused {
        let paused_x = fw as f32 - pad - 200.0;
        let paused_y = row1_y + icon_size + 20.0;
        gpu.draw_text(paused_x, paused_y, b"PAUSED", [255.0/255.0, 120.0/255.0, 120.0/255.0, 1.0], scale);
    }
    
    // Вторая строка: ресурсы
    let row2_y = row1_y + icon_size + gap;
    x = pad;
    
    let resources_list = [
        (total_wood, [110.0/255.0, 70.0/255.0, 30.0/255.0, 1.0]),
        (resources.stone, [120.0/255.0, 120.0/255.0, 120.0/255.0, 1.0]),
        (resources.clay, [150.0/255.0, 90.0/255.0, 70.0/255.0, 1.0]),
        (resources.bricks, [180.0/255.0, 120.0/255.0, 90.0/255.0, 1.0]),
        (resources.wheat, [200.0/255.0, 180.0/255.0, 80.0/255.0, 1.0]),
        (resources.flour, [210.0/255.0, 210.0/255.0, 180.0/255.0, 1.0]),
        (resources.bread, [200.0/255.0, 160.0/255.0, 120.0/255.0, 1.0]),
        (resources.fish, [100.0/255.0, 140.0/255.0, 200.0/255.0, 1.0]),
        (resources.iron_ore, [90.0/255.0, 90.0/255.0, 110.0/255.0, 1.0]),
        (resources.iron_ingots, [190.0/255.0, 190.0/255.0, 210.0/255.0, 1.0]),
    ];
    
    for (amount, color) in resources_list {
        gpu.draw_ui_resource_icon(x, row2_y, icon_size, color);
        x += icon_size + 4.0;
        gpu.draw_number(x, row2_y, amount.max(0) as u32, [1.0, 1.0, 1.0, 1.0], scale);
        x += (amount.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * scale + gap;
    }
    
    // Day progress bar
    let progress_y = panel_height - 4.0;
    let progress_w = fw as f32 * day_progress_01.clamp(0.0, 1.0);
    gpu.add_ui_rect(0.0, progress_y, fw as f32, 4.0, [0.0, 0.0, 0.0, 0.5]);
    gpu.add_ui_rect(0.0, progress_y, progress_w, 4.0, [220.0/255.0, 200.0/255.0, 120.0/255.0, 0.8]);
    
    // === НИЖНЯЯ ПАНЕЛЬ ===
    let bottom_panel_h = ui::bottom_panel_height(s) as f32;
    let bottom_y = fh as f32 - bottom_panel_h;
    gpu.draw_ui_panel(0.0, bottom_y, fw as f32, bottom_panel_h);
    
    let pad = 8.0 * scale;
    let btn_h = 18.0 * scale;
    let btn_scale = scale;
    let mut current_x = pad;
    let tab_y = bottom_y + pad;
    
    // Вкладки Build / Economy
    let build_w = (ui::button_w_for(b"Build", s) as f32).max(60.0);
    let economy_w = (ui::button_w_for(b"Economy", s) as f32).max(80.0);
    
    gpu.draw_button(current_x, tab_y, build_w, btn_h, b"Build", ui_tab == UITab::Build, btn_scale);
    current_x += build_w + 6.0;
    
    gpu.draw_button(current_x, tab_y, economy_w, btn_h, b"Economy", ui_tab == UITab::Economy, btn_scale);
    
    if ui_tab == UITab::Build {
        // Категории зданий (вторая строка)
        current_x = pad;
        let cat_y = tab_y + btn_h + 6.0;
        
        let categories: &[(UICategory, &[u8])] = &[
            (UICategory::Housing, b"Housing"),
            (UICategory::Storage, b"Storage"),
            (UICategory::Forestry, b"Forestry"),
            (UICategory::Mining, b"Mining"),
            (UICategory::Food, b"Food"),
            (UICategory::Logistics, b"Logistics"),
        ];
        
        for (cat, label) in categories.iter() {
            let btn_w = (label.len() as f32 * 4.0 * 2.0 * scale + 12.0).max(60.0);
            if current_x + btn_w > fw as f32 - pad {
                break;
            }
            gpu.draw_button(current_x, cat_y, btn_w, btn_h, *label, *cat == category, btn_scale);
            current_x += btn_w + 6.0;
        }
        
        // Здания выбранной категории (третья строка)
        current_x = pad;
        let build_y = cat_y + btn_h + 6.0;
        
        let buildings_for_cat: &[(BuildingKind, &[u8])] = match category {
            UICategory::Housing => &[(BuildingKind::House, b"House")],
            UICategory::Storage => &[(BuildingKind::Warehouse, b"Warehouse")],
            UICategory::Forestry => &[
                (BuildingKind::Lumberjack, b"Lumberjack"),
                (BuildingKind::Forester, b"Forester")
            ],
            UICategory::Mining => &[
                (BuildingKind::StoneQuarry, b"Quarry"),
                (BuildingKind::ClayPit, b"Clay Pit"),
                (BuildingKind::IronMine, b"Iron Mine"),
                (BuildingKind::Kiln, b"Kiln"),
                (BuildingKind::Smelter, b"Smelter")
            ],
            UICategory::Food => &[
                (BuildingKind::WheatField, b"Wheat Field"),
                (BuildingKind::Mill, b"Mill"),
                (BuildingKind::Bakery, b"Bakery"),
                (BuildingKind::Fishery, b"Fishery")
            ],
            UICategory::Logistics => &[],
        };
        
        for (bk, label) in buildings_for_cat.iter() {
            let btn_w = (label.len() as f32 * 4.0 * 2.0 * scale + 12.0).max(70.0);
            if current_x + btn_w > fw as f32 - pad {
                break;
            }
            gpu.draw_button(current_x, build_y, btn_w, btn_h, label, *bk == selected, btn_scale);
            current_x += btn_w + 6.0;
        }
    } else {
        // Economy panel
        current_x = pad;
        let econ_y = tab_y + btn_h + 6.0;
        
        // TAX
        gpu.draw_text(current_x, econ_y + 5.0, b"TAX", [200.0/255.0, 200.0/255.0, 200.0/255.0, 1.0], btn_scale);
        current_x += 40.0;
        
        let taxp = (tax_rate * 100.0).round().clamp(0.0, 100.0) as u32;
        gpu.draw_number(current_x, econ_y + 5.0, taxp, [1.0, 1.0, 1.0, 1.0], btn_scale);
        current_x += 60.0;
        
        // FOOD
        gpu.draw_text(current_x, econ_y + 5.0, b"FOOD", [200.0/255.0, 200.0/255.0, 200.0/255.0, 1.0], btn_scale);
        current_x += 60.0;
        
        // Food policy buttons
        gpu.draw_button(current_x, econ_y, 80.0, btn_h, b"Balanced", food_policy == FoodPolicy::Balanced, btn_scale);
        current_x += 86.0;
        
        gpu.draw_button(current_x, econ_y, 60.0, btn_h, b"Bread", food_policy == FoodPolicy::BreadFirst, btn_scale);
        current_x += 66.0;
        
        gpu.draw_button(current_x, econ_y, 50.0, btn_h, b"Fish", food_policy == FoodPolicy::FishFirst, btn_scale);
    }
    
    // === МИНИКАРТА ===
    // Рендерим миникарту в правом нижнем углу
    let pad = (8 * s) as f32;
    let base_cell = (2 * s) as f32;
    let base_w_tiles = 80.0;  // уменьшаем ширину
    let base_h_tiles = 60.0;  // уменьшаем высоту
    let widget_w = base_w_tiles * base_cell;
    let widget_h = base_h_tiles * base_cell;
    let minimap_x = fw as f32 - pad - widget_w;
    let minimap_y = fh as f32 - ui::bottom_panel_height(s) as f32 - pad - widget_h;
    
    // Рамка миникарты
    gpu.add_ui_rect(minimap_x - 2.0, minimap_y - 2.0, widget_w + 4.0, widget_h + 4.0, [0.2, 0.2, 0.2, 1.0]);
    gpu.add_ui_rect(minimap_x, minimap_y, widget_w, widget_h, [0.1, 0.1, 0.1, 0.8]);
    
    // Подготавливаем миникарту
    gpu.prepare_minimap(
        world, buildings,
        cam_x, cam_y,
        minimap_x as i32, minimap_y as i32, 
        widget_w as i32, widget_h as i32,
        cell_size,
    );
    
    // === ТУЛТИПЫ ===
    if let Some(building) = hovered_building {
        // Подсчитываем работников для этого здания
        let workers_current = 0; // TODO: подсчитать реальных работников
        let workers_target = building.workers_target;
        
        draw_building_tooltip(
            gpu,
            cursor_x,
            cursor_y,
            building.kind,
            workers_current,
            workers_target,
            scale,
            fw as f32,
            fh as f32,
        );
    } else if let Some(button_text) = hovered_button {
        draw_button_tooltip(
            gpu,
            cursor_x,
            cursor_y,
            button_text,
            scale,
            fw as f32,
            fh as f32,
        );
    } else if let Some(resource_name) = hovered_resource {
        draw_resource_tooltip(
            gpu,
            cursor_x,
            cursor_y,
            resource_name,
            resources,
            total_wood,
            population,
            avg_happiness,
            tax_rate,
            citizens_idle,
            citizens_working,
            citizens_sleeping,
            citizens_hauling,
            citizens_fetching,
            scale,
            fw as f32,
            fh as f32,
        );
    }
}

/// Рендеринг тултипа для здания
pub fn draw_building_tooltip(
    gpu: &mut GpuRenderer,
    x: f32,
    y: f32,
    building_kind: BuildingKind,
    workers_current: i32,
    workers_target: i32,
    scale: f32,
    screen_width: f32,
    screen_height: f32,
) {
    let s = scale as i32;
    let pad = (4 * s) as f32;
    let line_height = (12 * s) as f32;
    
    // Получаем информацию о здании
    let (name, prod, cons) = match building_kind {
        BuildingKind::House => ("House", "Housing", None),
        BuildingKind::Warehouse => ("Warehouse", "Storage", None),
        BuildingKind::Lumberjack => ("Lumberjack", "+ Wood", None),
        BuildingKind::Forester => ("Forester", "Forestry", None),
        BuildingKind::StoneQuarry => ("Stone Quarry", "+ Stone", None),
        BuildingKind::ClayPit => ("Clay Pit", "+ Clay", None),
        BuildingKind::Kiln => ("Kiln", "+ Bricks", Some("- Clay, - Wood")),
        BuildingKind::IronMine => ("Iron Mine", "+ Iron Ore", None),
        BuildingKind::WheatField => ("Wheat Field", "+ Wheat", None),
        BuildingKind::Mill => ("Mill", "+ Flour", Some("- Wheat")),
        BuildingKind::Bakery => ("Bakery", "+ Bread", Some("- Flour, - Wood")),
        BuildingKind::Smelter => ("Smelter", "+ Iron Ingot", Some("- Iron Ore, - Wood")),
        BuildingKind::Fishery => ("Fishery", "+ Fish", None),
    };
    
    // Вычисляем размер тултипа
    let name_w = (name.len() as f32 * 4.0 * 2.0 * scale);
    let prod_w = (prod.len() as f32 * 4.0 * 2.0 * scale);
    let cons_w = cons.map(|c| (c.len() as f32 * 4.0 * 2.0 * scale)).unwrap_or(0.0);
    let workers_w = (format!("Workers: {}/{}", workers_current, workers_target).len() as f32 * 4.0 * 2.0 * scale);
    
    let tooltip_w = [name_w, prod_w, cons_w, workers_w].iter().fold(0.0_f32, |a, &b| a.max(b)) + pad * 2.0;
    let tooltip_h = line_height * 4.0 + pad * 2.0;
    
    // Позиционируем тултип рядом с курсором (с проверкой границ экрана)
    let tooltip_x = (x + 20.0).min(screen_width - tooltip_w - 10.0);
    let tooltip_y = (y - tooltip_h - 10.0).max(10.0);
    
    // Фон тултипа
    gpu.add_ui_rect(tooltip_x, tooltip_y, tooltip_w, tooltip_h, [0.1, 0.1, 0.1, 0.9]);
    gpu.add_ui_rect(tooltip_x + 1.0, tooltip_y + 1.0, tooltip_w - 2.0, tooltip_h - 2.0, [0.2, 0.2, 0.2, 0.8]);
    
    // Текст тултипа
    let mut text_y = tooltip_y + pad;
    
    // Название здания
    gpu.draw_text(tooltip_x + pad, text_y, name.as_bytes(), [1.0, 1.0, 1.0, 1.0], scale);
    text_y += line_height;
    
    // Производство
    gpu.draw_text(tooltip_x + pad, text_y, prod.as_bytes(), [0.7, 1.0, 0.7, 1.0], scale);
    text_y += line_height;
    
    // Потребление
    if let Some(cons_text) = cons {
        gpu.draw_text(tooltip_x + pad, text_y, cons_text.as_bytes(), [1.0, 0.7, 0.7, 1.0], scale);
        text_y += line_height;
    }
    
    // Работники
    let workers_text = format!("Workers: {}/{}", workers_current, workers_target);
    gpu.draw_text(tooltip_x + pad, text_y, workers_text.as_bytes(), [1.0, 1.0, 0.7, 1.0], scale);
}

/// Рендеринг тултипа для кнопки интерфейса
pub fn draw_button_tooltip(
    gpu: &mut GpuRenderer,
    x: f32,
    y: f32,
    button_text: &str,
    scale: f32,
    screen_width: f32,
    screen_height: f32,
) {
    let s = scale as i32;
    let pad = (4 * s) as f32;
    let line_height = (12 * s) as f32;
    
    // Получаем информацию о кнопке
    let (name, description) = match button_text {
        // Здания
        "Lumberjack" => ("Lumberjack", "Produces wood from trees. Requires workers."),
        "Forester" => ("Forester", "Plants new trees. Requires workers."),
        "Stone Quarry" => ("Stone Quarry", "Mines stone from deposits. Requires workers."),
        "Clay Pit" => ("Clay Pit", "Mines clay from deposits. Requires workers."),
        "Iron Mine" => ("Iron Mine", "Mines iron ore from deposits. Requires workers."),
        "Wheat Field" => ("Wheat Field", "Grows wheat for food. Requires workers."),
        "Mill" => ("Mill", "Processes wheat into flour. Requires workers."),
        "Bakery" => ("Bakery", "Bakes bread from flour. Requires workers."),
        "Kiln" => ("Kiln", "Bakes clay into bricks. Requires workers."),
        "Smelter" => ("Smelter", "Smelts iron ore into iron ingots. Requires workers."),
        "Fishery" => ("Fishery", "Catches fish from water. Requires workers."),
        "House" => ("House", "Provides housing for citizens."),
        "Warehouse" => ("Warehouse", "Stores resources and goods."),
        
        // Управление
        "Pause" => ("Pause", "Pause/unpause the game."),
        "Resume" => ("Resume", "Resume the game."),
        "Speed 1x" => ("Speed 1x", "Set game speed to normal."),
        "Speed 2x" => ("Speed 2x", "Set game speed to 2x."),
        "Speed 4x" => ("Speed 4x", "Set game speed to 4x."),
        
        // Вкладки
        "Build Tab" => ("Build Tab", "Switch to building construction mode."),
        "Economy Tab" => ("Economy Tab", "Switch to economy management mode."),
        
        // Категории
        "Housing" => ("Housing", "Buildings for citizen housing."),
        "Storage" => ("Storage", "Buildings for resource storage."),
        "Forestry" => ("Forestry", "Buildings for wood production."),
        "Mining" => ("Mining", "Buildings for resource extraction."),
        "Food" => ("Food", "Buildings for food production."),
        "Logistics" => ("Logistics", "Buildings for transportation."),
        
        // Экономика
        "Decrease Tax" => ("Decrease Tax", "Lower the tax rate."),
        "Increase Tax" => ("Increase Tax", "Raise the tax rate."),
        "Balanced Food Policy" => ("Balanced Food Policy", "Equal distribution of bread and fish."),
        "Bread First Policy" => ("Bread First Policy", "Prioritize bread distribution."),
        "Fish First Policy" => ("Fish First Policy", "Prioritize fish distribution."),
        
        _ => (button_text, "Click to interact."),
    };
    
    // Вычисляем размер тултипа
    let name_w = name.len() as f32 * 4.0 * 2.0 * scale;
    let desc_w = description.len() as f32 * 4.0 * 2.0 * scale;
    
    let tooltip_w = [name_w, desc_w].iter().fold(0.0_f32, |a, &b| a.max(b)) + pad * 2.0;
    let tooltip_h = line_height * 2.0 + pad * 2.0;
    
    // Позиционируем тултип рядом с курсором (с проверкой границ экрана)
    let tooltip_x = (x + 20.0).min(screen_width - tooltip_w - 10.0);
    let tooltip_y = (y - tooltip_h - 10.0).max(10.0);
    
    // Фон тултипа
    gpu.add_ui_rect(tooltip_x, tooltip_y, tooltip_w, tooltip_h, [0.1, 0.1, 0.1, 0.9]);
    gpu.add_ui_rect(tooltip_x + 1.0, tooltip_y + 1.0, tooltip_w - 2.0, tooltip_h - 2.0, [0.2, 0.2, 0.2, 0.8]);
    
    // Текст тултипа
    let mut text_y = tooltip_y + pad;
    
    // Название кнопки
    gpu.draw_text(tooltip_x + pad, text_y, name.as_bytes(), [1.0, 1.0, 1.0, 1.0], scale);
    text_y += line_height;
    
    // Описание
    gpu.draw_text(tooltip_x + pad, text_y, description.as_bytes(), [0.8, 0.8, 0.8, 1.0], scale);
}

/// Рендеринг тултипа для ресурса
pub fn draw_resource_tooltip(
    gpu: &mut GpuRenderer,
    x: f32,
    y: f32,
    resource_name: &str,
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
    scale: f32,
    screen_width: f32,
    screen_height: f32,
) {
    let s = scale as i32;
    let pad = (4 * s) as f32;
    let line_height = (12 * s) as f32;
    
    // Получаем информацию о ресурсе
    let (name, description, current_value) = match resource_name {
        "Population" => ("Population", "Total number of citizens in your city.", population),
        "Gold" => ("Gold", "Currency used for building construction and maintenance.", resources.gold),
        "Happiness" => ("Happiness", "Overall citizen satisfaction. Affects productivity.", avg_happiness.round() as i32),
        "Tax" => ("Tax Rate", "Percentage of income collected as taxes.", (tax_rate * 100.0).round() as i32),
        "Idle" => ("Idle Citizens", "Citizens without assigned work.", citizens_idle),
        "Working" => ("Working Citizens", "Citizens currently employed in buildings.", citizens_working),
        "Sleeping" => ("Sleeping Citizens", "Citizens resting at home.", citizens_sleeping),
        "Hauling" => ("Hauling Citizens", "Citizens transporting goods.", citizens_hauling),
        "Fetching" => ("Fetching Citizens", "Citizens gathering resources.", citizens_fetching),
        "Wood" => ("Wood", "Basic construction material. Produced by lumberjacks.", total_wood),
        "Stone" => ("Stone", "Building material. Mined from stone quarries.", resources.stone),
        "Clay" => ("Clay", "Raw material for bricks. Mined from clay pits.", resources.clay),
        "Bricks" => ("Bricks", "Processed clay. Made in kilns.", resources.bricks),
        "Wheat" => ("Wheat", "Grain crop. Grown in wheat fields.", resources.wheat),
        "Flour" => ("Flour", "Processed wheat. Made in mills.", resources.flour),
        "Bread" => ("Bread", "Food for citizens. Baked in bakeries.", resources.bread),
        "Fish" => ("Fish", "Food for citizens. Caught by fisheries.", resources.fish),
        "Iron Ore" => ("Iron Ore", "Raw metal. Mined from iron mines.", resources.iron_ore),
        "Iron Ingots" => ("Iron Ingots", "Processed metal. Made in smelters.", resources.iron_ingots),
        _ => (resource_name, "Resource information.", 0),
    };
    
    // Вычисляем размер тултипа
    let name_w = name.len() as f32 * 4.0 * 2.0 * scale;
    let desc_w = description.len() as f32 * 4.0 * 2.0 * scale;
    let value_w = (format!("Current: {}", current_value).len() as f32 * 4.0 * 2.0 * scale);
    
    let tooltip_w = [name_w, desc_w, value_w].iter().fold(0.0_f32, |a, &b| a.max(b)) + pad * 2.0;
    let tooltip_h = line_height * 3.0 + pad * 2.0;
    
    // Позиционируем тултип рядом с курсором (с проверкой границ экрана)
    let tooltip_x = (x + 20.0).min(screen_width - tooltip_w - 10.0);
    let tooltip_y = (y - tooltip_h - 10.0).max(10.0);
    
    // Фон тултипа
    gpu.add_ui_rect(tooltip_x, tooltip_y, tooltip_w, tooltip_h, [0.1, 0.1, 0.1, 0.9]);
    gpu.add_ui_rect(tooltip_x + 1.0, tooltip_y + 1.0, tooltip_w - 2.0, tooltip_h - 2.0, [0.2, 0.2, 0.2, 0.8]);
    
    // Текст тултипа
    let mut text_y = tooltip_y + pad;
    
    // Название ресурса
    gpu.draw_text(tooltip_x + pad, text_y, name.as_bytes(), [1.0, 1.0, 1.0, 1.0], scale);
    text_y += line_height;
    
    // Описание
    gpu.draw_text(tooltip_x + pad, text_y, description.as_bytes(), [0.8, 0.8, 0.8, 1.0], scale);
    text_y += line_height;
    
    // Текущее значение
    let value_text = format!("Current: {}", current_value);
    gpu.draw_text(tooltip_x + pad, text_y, value_text.as_bytes(), [1.0, 1.0, 0.7, 1.0], scale);
}

