// GPU версия UI модуля
// Использует всю логику из ui.rs, но рендерит через GpuRenderer

use crate::gpu_renderer::GpuRenderer;
use crate::types::{Resources, BuildingKind, FoodPolicy};
use crate::ui::{self, UICategory, UITab};
use glam;

// Маппинг ресурсов на индексы спрайтов в props.png (сетка 5x4)
// Индексы можно легко изменить, если структура props.png отличается
fn get_props_index_for_resource(resource_name: &str) -> u32 {
    match resource_name {
        "Population" => 0,   // (0, 0) - первая строка, первая колонка
        "Gold" => 1,        // (1, 0) - первая строка, вторая колонка
        "Happiness" => 2,   // (2, 0) - первая строка, третья колонка
        "Tax" => 3,         // (3, 0) - первая строка, четвертая колонка
        "Idle" => 4,        // (4, 0) - первая строка, пятая колонка
        "Working" => 5,     // (0, 1) - вторая строка, первая колонка
        "Sleeping" => 6,    // (1, 1) - вторая строка, вторая колонка
        "Hauling" => 7,     // (2, 1) - вторая строка, третья колонка
        "Fetching" => 8,   // (3, 1) - вторая строка, четвертая колонка
        "Wood" => 9,       // (4, 1) - вторая строка, пятая колонка
        "Stone" => 10,      // (0, 2) - третья строка, первая колонка
        "Clay" => 11,       // (1, 2) - третья строка, вторая колонка
        "Bricks" => 12,     // (2, 2) - третья строка, третья колонка
        "Wheat" => 13,      // (3, 2) - третья строка, четвертая колонка
        "Flour" => 14,      // (4, 2) - третья строка, пятая колонка
        "Bread" => 15,      // (0, 3) - четвертая строка, первая колонка
        "Fish" => 16,       // (1, 3) - четвертая строка, вторая колонка
        "Iron Ore" => 17,   // (2, 3) - четвертая строка, третья колонка
        "Iron Ingots" => 18, // (3, 3) - четвертая строка, четвертая колонка
        _ => 0,             // По умолчанию первый спрайт
    }
}

/// Вычисляет ширину, которую займет список ресурсов
fn calculate_resources_list_width(
    cost: &crate::types::Resources,
    scale: f32,
) -> f32 {
    let icon_size = (10.0 * scale).max(8.0);
    let gap = 4.0 * scale;
    let mut width = 0.0;
    
    // Список ресурсов для отображения (в порядке приоритета)
    let resources_to_show = [
        ("Wood", cost.wood),
        ("Gold", cost.gold),
        ("Stone", cost.stone),
        ("Clay", cost.clay),
        ("Bricks", cost.bricks),
        ("Wheat", cost.wheat),
        ("Flour", cost.flour),
        ("Bread", cost.bread),
        ("Fish", cost.fish),
        ("Iron Ore", cost.iron_ore),
        ("Iron Ingots", cost.iron_ingots),
    ];
    
    for (_name, amount) in resources_to_show.iter() {
        if *amount > 0 {
            width += icon_size + 2.0 * scale; // иконка + отступ
            width += (amount.to_string().len() as f32 * 4.0 * 2.0 * scale * 0.8) + gap; // число + отступ
        }
    }
    
    width
}

/// Вспомогательная функция для отображения списка ресурсов с иконками
/// Если available_resources передан, проверяет достаточность и красит недостающие ресурсы красным
fn draw_resources_list(
    gpu: &mut GpuRenderer,
    x: f32,
    y: f32,
    cost: &crate::types::Resources,
    scale: f32,
    available_resources: Option<&crate::types::Resources>,
) {
    let icon_size = (10.0 * scale).max(8.0);
    let gap = 4.0 * scale;
    let mut current_x = x;
    
    // Список ресурсов для отображения (в порядке приоритета)
    let resources_to_show = [
        ("Wood", cost.wood),
        ("Gold", cost.gold),
        ("Stone", cost.stone),
        ("Clay", cost.clay),
        ("Bricks", cost.bricks),
        ("Wheat", cost.wheat),
        ("Flour", cost.flour),
        ("Bread", cost.bread),
        ("Fish", cost.fish),
        ("Iron Ore", cost.iron_ore),
        ("Iron Ingots", cost.iron_ingots),
    ];
    
    for (name, amount) in resources_to_show.iter() {
        if *amount > 0 {
            // Иконка ресурса
            gpu.draw_ui_props_icon(current_x, y, icon_size, get_props_index_for_resource(name));
            current_x += icon_size + 2.0 * scale;
            
            // Определяем цвет числа: красный если ресурсов не хватает, белый если хватает
            let number_color = if let Some(available) = available_resources {
                let available_amount = match *name {
                    "Wood" => available.wood,
                    "Gold" => available.gold,
                    "Stone" => available.stone,
                    "Clay" => available.clay,
                    "Bricks" => available.bricks,
                    "Wheat" => available.wheat,
                    "Flour" => available.flour,
                    "Bread" => available.bread,
                    "Fish" => available.fish,
                    "Iron Ore" => available.iron_ore,
                    "Iron Ingots" => available.iron_ingots,
                    _ => *amount,
                };
                if available_amount < *amount {
                    [1.0, 0.3, 0.3, 1.0] // красный цвет для недостающих ресурсов
                } else {
                    [1.0, 1.0, 1.0, 1.0] // белый цвет если ресурсов достаточно
                }
            } else {
                [1.0, 1.0, 1.0, 1.0] // белый по умолчанию
            };
            
            // Число
            gpu.draw_number(current_x, y + (icon_size - 8.0 * scale) / 2.0, *amount as u32, number_color, scale * 0.8);
            current_x += (amount.to_string().len() as f32 * 4.0 * 2.0 * scale * 0.8) + gap;
        }
    }
}

/// GPU версия draw_ui - использует GpuRenderer вместо CPU frame buffer
pub fn draw_ui_gpu(
    gpu: &mut GpuRenderer,
    fw: i32,
    fh: i32,
    resources: &Resources,
    total_wood: i32,
    population: i32,
    selected: Option<BuildingKind>,
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
    current_weather: crate::types::WeatherKind, // Текущая погода для тултипа
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
    // Данные для консоли
    console_open: bool,
    console_input: &str,
    console_log: &[String],
    // Данные для отладки биома
    biome_debug_mode: bool,
    show_deposits: bool,
    zoom: f32,
    atlas_half_w: i32,
    atlas_half_h: i32,
    // Видимые границы тайлов для миникарты
    visible_min_tx: i32,
    visible_min_ty: i32,
    visible_max_tx: i32,
    visible_max_ty: i32,
    // Система исследований для проверки разблокировки зданий
    research_system: &crate::research::ResearchSystem,
) {
    gpu.clear_ui();
    
    let s = ui::ui_scale(fh, base_scale_k);
    let scale = s as f32;
    
    // === ВЕРХНЯЯ ПАНЕЛЬ ===
    let panel_height = ui::top_panel_height(s) as f32;
    gpu.draw_ui_panel(0.0, 0.0, fw as f32, panel_height);
    
    let pad = (8 * s) as f32;
    let icon_size = (12 * s) as f32;
    
    // Высота текста: 5 строк * 2 пикселя * scale = 10 * scale
    let text_height = 10.0 * scale;
    // Выравнивание: сдвигаем текст вниз на половину разницы высот для центрирования с иконкой
    let text_y_offset = (icon_size - text_height) / 2.0;
    
    // Первая строка: Population, Gold, Happiness, Tax, статусы граждан
    let row1_y = pad;
    let gap = (6 * s) as f32;
    let mut x = pad;
    
    // Population
    gpu.draw_ui_props_icon(x, row1_y, icon_size, get_props_index_for_resource("Population"));
    x += icon_size + 4.0;
    gpu.draw_number(x, row1_y + text_y_offset, population.max(0) as u32, [1.0, 1.0, 1.0, 1.0], scale);
    x += ((population.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * scale) + gap;
    
    // Gold
    gpu.draw_ui_props_icon(x, row1_y, icon_size, get_props_index_for_resource("Gold"));
    x += icon_size + 4.0;
    gpu.draw_number(x, row1_y + text_y_offset, resources.gold.max(0) as u32, [1.0, 1.0, 1.0, 1.0], scale);
    x += ((resources.gold.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * scale) + gap;
    
    // Happiness
    gpu.draw_ui_props_icon(x, row1_y, icon_size, get_props_index_for_resource("Happiness"));
    x += icon_size + 4.0;
    let hap = avg_happiness.round().clamp(0.0, 100.0) as u32;
    gpu.draw_number(x, row1_y + text_y_offset, hap, [1.0, 1.0, 1.0, 1.0], scale);
    x += (hap.to_string().len() as f32 * 4.0 * 2.0 * scale) + gap;
    
    // Tax
    gpu.draw_ui_props_icon(x, row1_y, icon_size, get_props_index_for_resource("Tax"));
    x += icon_size + 4.0;
    let taxp = (tax_rate * 100.0).round().clamp(0.0, 100.0) as u32;
    gpu.draw_number(x, row1_y + text_y_offset, taxp, [1.0, 1.0, 1.0, 1.0], scale);
    x += (taxp.to_string().len() as f32 * 4.0 * 2.0 * scale) + gap;
    
    // Citizen status icons
    let stat_icons = [
        ("Idle", citizens_idle),
        ("Working", citizens_working),
        ("Sleeping", citizens_sleeping),
        ("Hauling", citizens_hauling),
        ("Fetching", citizens_fetching),
    ];
    
    for (name, count) in stat_icons {
        gpu.draw_ui_props_icon(x, row1_y, icon_size, get_props_index_for_resource(name));
        x += icon_size + 4.0;
        gpu.draw_number(x, row1_y + text_y_offset, count.max(0) as u32, [1.0, 1.0, 1.0, 1.0], scale);
        x += (count.max(0) as u32).to_string().len() as f32 * 4.0 * 2.0 * scale + gap;
    }
    
    // Правая часть в одну строку: Weather | Day | Speed | FPS
    let mut right_x = fw as f32 - pad;
    
    // FPS (самый правый)
    let fps_rounded = fps.round() as u32;
    let fps_num_w = fps_rounded.to_string().len() as f32 * 4.0 * 2.0 * scale;
    right_x -= fps_num_w;
    gpu.draw_number(right_x, row1_y, fps_rounded, [220.0/255.0, 220.0/255.0, 220.0/255.0, 1.0], scale);
    right_x -= 80.0;
    gpu.draw_text(right_x, row1_y, b"FPS", [180.0/255.0, 180.0/255.0, 180.0/255.0, 1.0], scale);
    right_x -= gap;
    
    // Speed (слева от FPS)
    let speed_val = (speed * 10.0).round() as u32;
    let speed_num_w = speed_val.to_string().len() as f32 * 4.0 * 2.0 * scale;
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
    // Погода пока оставляем как цветной квадратик (можно будет заменить позже)
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
        gpu.draw_ui_props_icon(x, row2_y, icon_size, get_props_index_for_resource(name));
        x += icon_size + 4.0;
        gpu.draw_number(x, row2_y + text_y_offset, amount.max(0) as u32, [1.0, 1.0, 1.0, 1.0], scale);
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
    current_x += build_w + 6.0 * scale;
    
    gpu.draw_button(current_x, tab_y, economy_w, btn_h, b"Economy", ui_tab == UITab::Economy, btn_scale);
    current_x += economy_w + 6.0 * scale;
    
    // Кнопка для депозитов ресурсов
    let deposits_w = (ui::button_w_for(b"Deposits", s) as f32).max(80.0);
    gpu.draw_button(current_x, tab_y, deposits_w, btn_h, b"Deposits", show_deposits, btn_scale);
    current_x += deposits_w + 6.0 * scale;
    
    // Кнопка для открытия окна исследований (только если есть лаборатория)
    let has_lab = buildings.iter().any(|b| b.kind == crate::types::BuildingKind::ResearchLab);
    if has_lab {
        let research_w = (ui::button_w_for(b"Research (T)", s) as f32).max(100.0);
        gpu.draw_button(current_x, tab_y, research_w, btn_h, b"Research (T)", false, btn_scale);
    }
    
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
            (UICategory::Research, b"Research"),
        ];
        
        for (cat, label) in categories.iter() {
            let btn_w = (label.len() as f32 * 4.0 * 2.0 * scale + 12.0).max(60.0);
            if current_x + btn_w > fw as f32 - pad {
                break;
            }
            gpu.draw_button(current_x, cat_y, btn_w, btn_h, *label, *cat == category, btn_scale);
            current_x += btn_w + 6.0 * scale;
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
            UICategory::Research => &[(BuildingKind::ResearchLab, b"Research Lab")],
        };
        
        for (bk, label) in buildings_for_cat.iter() {
            let btn_w = (label.len() as f32 * 4.0 * 2.0 * scale + 12.0).max(70.0);
            if current_x + btn_w > fw as f32 - pad {
                break;
            }
            
            // Проверяем разблокировку здания (ResearchLab всегда разблокирована)
            let is_unlocked = *bk == BuildingKind::ResearchLab 
                || research_system.is_building_unlocked(*bk);
            
            gpu.draw_button_disabled(
                current_x, 
                build_y, 
                btn_w, 
                btn_h, 
                label, 
                selected == Some(*bk), 
                !is_unlocked, // disabled если не разблокировано
                btn_scale
            );
            current_x += btn_w + 6.0 * scale;
        }
    } else {
        // Economy panel - упрощенная версия с динамическим расчетом
        
        // === ВТОРАЯ СТРОКА: Контролы налогов ===
        current_x = pad;
        let control_y = tab_y + btn_h + 6.0;
        
        // TAX контролы - динамический расчет
        let tax_label_w = (3.0 * 4.0 * 2.0 * scale + 12.0).max(40.0); // "TAX"
        gpu.draw_text(current_x, control_y + 5.0, b"TAX", [200.0/255.0, 200.0/255.0, 200.0/255.0, 1.0], btn_scale);
        current_x += tax_label_w + 6.0 * scale;
        
        let taxp = tax_rate as u32;
        let tax_num_w = (taxp.to_string().len() as f32 * 4.0 * 2.0 * scale + 12.0).max(60.0);
        gpu.draw_number(current_x, control_y + 5.0, taxp, [1.0, 1.0, 1.0, 1.0], btn_scale);
        current_x += tax_num_w + 6.0 * scale;
        
        // Кнопки изменения налогов - динамический расчет
        let minus_btn_w = (1.0 * 4.0 * 2.0 * scale + 12.0).max(30.0); // "-"
        let plus_btn_w = (1.0 * 4.0 * 2.0 * scale + 12.0).max(30.0); // "+"
        
        gpu.draw_button(current_x, control_y, minus_btn_w, btn_h, b"-", false, btn_scale);
        current_x += minus_btn_w + 6.0 * scale;
        gpu.draw_button(current_x, control_y, plus_btn_w, btn_h, b"+", false, btn_scale);
        
        // === ТРЕТЬЯ СТРОКА: Политика еды ===
        let _ = current_x; // значение присваивается, но сразу переопределяется ниже
        current_x = pad;
        let policy_y = control_y + btn_h + 6.0;
        
        let policy_label_w = (11.0 * 4.0 * 2.0 * scale + 12.0).max(100.0); // "FOOD POLICY"
        gpu.draw_text(current_x, policy_y + 5.0, b"FOOD POLICY", [200.0/255.0, 200.0/255.0, 200.0/255.0, 1.0], btn_scale);
        current_x += policy_label_w + 6.0 * scale;
        
        // Food policy buttons - динамический расчет
        let food_policies: &[(FoodPolicy, &[u8])] = &[
            (FoodPolicy::Balanced, b"Balanced"),
            (FoodPolicy::BreadFirst, b"Bread"),
            (FoodPolicy::FishFirst, b"Fish"),
        ];
        
        for (policy, label) in food_policies.iter() {
            let btn_w = (label.len() as f32 * 4.0 * 2.0 * scale + 12.0).max(50.0);
            if current_x + btn_w > fw as f32 - pad {
                break;
            }
            gpu.draw_button(current_x, policy_y, btn_w, btn_h, *label, *policy == food_policy, btn_scale);
            current_x += btn_w + 6.0 * scale;
        }
    }
    
    // === МИНИКАРТА ===
    // Рендерим миникарту в правом нижнем углу, отступаем дальше от края
    let pad = (8 * s) as f32;
    let minimap_pad = (30 * s) as f32; // дополнительный отступ для миникарты (уменьшено, чтобы было правее)
    let minimap_vertical_offset = (50 * s) as f32; // дополнительный отступ вниз, чтобы опустить миникарту
    let base_cell = (2 * s) as f32;
    // Делаем миникарту квадратной - используем одинаковый размер для ширины и высоты
    let base_size = 60.0;  // размер квадрата
    let widget_w = base_size * base_cell;
    let widget_h = base_size * base_cell;
    // Центр миникарты для поворота
    let minimap_x = fw as f32 - pad - minimap_pad - widget_w;
    let minimap_y = fh as f32 - ui::bottom_panel_height(s) as f32 - pad - widget_h + minimap_vertical_offset;
    let minimap_center_x = minimap_x + widget_w * 0.5;
    let minimap_center_y = minimap_y + widget_h * 0.5;
    
    // Поворот на 45 градусов для подложки
    let rotation_45 = glam::Quat::from_rotation_z(std::f32::consts::PI / 4.0);
    
    // Рамка миникарты (повернутая на 45 градусов)
    // Внешняя рамка (чуть больше для обводки)
    let frame_size = widget_w.max(widget_h) * 1.1; // увеличенный размер для обводки
    gpu.add_ui_rect_rotated(
        minimap_center_x - frame_size * 0.5,
        minimap_center_y - frame_size * 0.5,
        frame_size,
        frame_size,
        rotation_45,
        [0.2, 0.2, 0.2, 1.0]
    );
    
    // Внутренняя подложка (повернутая на 45 градусов)
    gpu.add_ui_rect_rotated(
        minimap_center_x - widget_w * 0.5,
        minimap_center_y - widget_h * 0.5,
        widget_w,
        widget_h,
        rotation_45,
        [0.1, 0.1, 0.1, 0.8]
    );
    
    // Подготавливаем миникарту (используем atlas_half_w и atlas_half_h из параметров функции)
    gpu.prepare_minimap_with_atlas(
        world, buildings,
        cam_x, cam_y,
        minimap_x as i32, minimap_y as i32, 
        widget_w as i32, widget_h as i32,
        cell_size,
        atlas_half_w,
        atlas_half_h,
        visible_min_tx,
        visible_min_ty,
        visible_max_tx,
        visible_max_ty,
    );
    
    // === ТУЛТИПЫ ===
    // Запоминаем текущий размер ui_rects как начало тултипов
    gpu.start_tooltips();
    
    if let Some(building) = hovered_building {
        // Показываем тултип здания только на вкладке Build
        if ui_tab == UITab::Build {
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
            Some(resources), // Передаем доступные ресурсы для проверки достаточности
        );
        }
    } else if let Some(button_text) = hovered_button {
        // Показываем тултипы кнопок только на соответствующих вкладках
        let should_show_tooltip = match ui_tab {
            UITab::Build => true, // На вкладке Build показываем все тултипы кнопок
            UITab::Economy => {
                // На вкладке Economy показываем только тултипы экономических кнопок
                button_text.contains("TAX") || 
                button_text.contains("FOOD") || 
                button_text.contains("BALANCED") || 
                button_text.contains("BREAD") || 
                button_text.contains("FISH")
            }
        };
        
        if should_show_tooltip {
            draw_button_tooltip(
                gpu,
                cursor_x,
                cursor_y,
                button_text,
                scale,
                fw as f32,
                fh as f32,
                Some(resources), // Передаем доступные ресурсы для проверки достаточности
            );
        }
    } else if let Some(resource_name) = hovered_resource {
        if resource_name == "Weather" {
            draw_weather_tooltip(
                gpu,
                cursor_x,
                cursor_y,
                current_weather,
                scale,
                fw as f32,
                fh as f32,
            );
        } else {
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
    
    // Тултип отладки биома
    if biome_debug_mode {
        // Определяем позицию тайла под курсором
        let tile_pos = screen_to_tile_px(
            cursor_x as i32, 
            cursor_y as i32, 
            fw, 
            fh, 
            glam::Vec2::new(cam_x, cam_y), 
            atlas_half_w, // half_w - половина ширины тайла из атласа
            atlas_half_h, // half_h - половина высоты тайла из атласа
            zoom // zoom из параметра функции
        );
        
        if let Some(tp) = tile_pos {
            let biome = world.biome(tp);
            let biome_name = match biome {
                crate::types::BiomeKind::Meadow => "Meadow",
                crate::types::BiomeKind::Swamp => "Swamp", 
                crate::types::BiomeKind::Rocky => "Rocky",
            };
            
            draw_biome_debug_tooltip(
                gpu,
                cursor_x,
                cursor_y,
                biome_name,
                tp.x,
                tp.y,
                scale,
                fw as f32,
                fh as f32,
            );
        }
    }
    
    // Рендерим консоль, если она открыта
    if console_open {
        draw_console_gpu(gpu, fw, fh, s, console_input, console_log);
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
    _screen_height: f32,
    _available_resources: Option<&Resources>, // Не используется для построенных зданий, только для кнопок строительства
) {
    use crate::types::building_cost;
    
    let s = scale as i32;
    let pad = (4 * s) as f32;
    let line_height = (12 * s) as f32;
    let icon_size = (10.0 * scale).max(8.0);
    
    // Получаем стоимость здания
    let cost = building_cost(building_kind);
    
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
        BuildingKind::ResearchLab => ("Research Lab", "Research", None),
    };
    
    // Вычисляем размер тултипа
    let name_w = name.len() as f32 * 4.0 * 2.0 * scale;
    let prod_w = prod.len() as f32 * 4.0 * 2.0 * scale;
    let cons_w = cons.map(|c| c.len() as f32 * 4.0 * 2.0 * scale).unwrap_or(0.0);
    let workers_w = format!("Workers: {}/{}", workers_current, workers_target).len() as f32 * 4.0 * 2.0 * scale;
    
    // Ширина для строки с материалами (без учета доступных ресурсов, так как это только для вычисления размера)
    let cost_w = calculate_resources_list_width(&cost, scale);
    
    let tooltip_w = [name_w, prod_w, cons_w, workers_w, cost_w].iter().fold(0.0_f32, |a, &b| a.max(b)) + pad * 2.0;
    
    // Вычисляем высоту тултипа (учитываем строку с материалами)
    let mut tooltip_h = line_height * 3.0 + pad * 2.0; // название, производство, работники
    if cons.is_some() {
        tooltip_h += line_height; // потребление
    }
    // Добавляем высоту для строки с материалами
    tooltip_h += icon_size + pad;
    
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
    
    // Стоимость материалов (с иконками) - без проверки доступности для построенных зданий
    text_y += pad * 0.5;
    let cost_y = text_y;
    draw_resources_list(gpu, tooltip_x + pad, cost_y, &cost, scale, None);
    text_y += icon_size + pad * 0.5;
    
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
    _screen_height: f32,
    available_resources: Option<&Resources>,
) {
    use crate::types::{building_cost, BuildingKind};
    
    let s = scale as i32;
    let pad = (4 * s) as f32;
    let line_height = (12 * s) as f32;
    let icon_size = (10.0 * scale).max(8.0);
    
    // Определяем BuildingKind из button_text (для кнопок строительства)
    // Важно: названия должны совпадать с теми, что возвращает get_hovered_button
    let building_kind_opt = match button_text {
        "Lumberjack" => Some(BuildingKind::Lumberjack),
        "Forester" => Some(BuildingKind::Forester),
        "Quarry" => Some(BuildingKind::StoneQuarry), // В UI это "Quarry", не "Stone Quarry"
        "Clay Pit" => Some(BuildingKind::ClayPit),
        "Iron Mine" => Some(BuildingKind::IronMine),
        "Wheat Field" => Some(BuildingKind::WheatField),
        "Mill" => Some(BuildingKind::Mill),
        "Bakery" => Some(BuildingKind::Bakery),
        "Kiln" => Some(BuildingKind::Kiln),
        "Smelter" => Some(BuildingKind::Smelter),
        "Fishery" => Some(BuildingKind::Fishery),
        "House" => Some(BuildingKind::House),
        "Warehouse" => Some(BuildingKind::Warehouse),
        "Research Lab" => Some(BuildingKind::ResearchLab),
        _ => None,
    };
    
    // Получаем стоимость, если это кнопка строительства
    let cost_opt = building_kind_opt.map(|bk| building_cost(bk));
    
    // Получаем информацию о кнопке
    let (name, description) = match button_text {
        // Здания (названия должны совпадать с get_hovered_button)
        "Lumberjack" => ("Lumberjack", "Produces wood from trees. Requires workers."),
        "Forester" => ("Forester", "Plants new trees. Requires workers."),
        "Quarry" => ("Stone Quarry", "Mines stone from deposits. Requires workers."),
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
        "Research Lab" => ("Research Lab", "Enables research and unlocks new technologies."),
        
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
        
        "Deposits" => ("Deposits", "Toggle resource deposits display on/off."),
        
        _ => (button_text, "Click to interact."),
    };
    
    // Вычисляем размер тултипа
    let name_w = name.len() as f32 * 4.0 * 2.0 * scale;
    let desc_w = description.len() as f32 * 4.0 * 2.0 * scale;
    
    // Если есть стоимость, учитываем её в ширине
    let cost_w = if let Some(ref cost) = cost_opt {
        calculate_resources_list_width(cost, scale)
    } else {
        0.0
    };
    
    let tooltip_w = [name_w, desc_w, cost_w].iter().fold(0.0_f32, |a, &b| a.max(b)) + pad * 2.0;
    
    // Вычисляем высоту тултипа
    let mut tooltip_h = line_height * 2.0 + pad * 2.0; // название + описание
    if cost_opt.is_some() {
        tooltip_h += icon_size + pad; // строка с материалами
    }
    
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
    text_y += line_height;
    
    // Стоимость материалов (с иконками) - только для кнопок строительства
    // Показываем красным цветом, если ресурсов не хватает
    if let Some(ref cost) = cost_opt {
        text_y += pad * 0.5;
        draw_resources_list(gpu, tooltip_x + pad, text_y, cost, scale, available_resources);
    }
}

/// Рендеринг тултипа для погоды с бафами/дебафами
pub fn draw_weather_tooltip(
    gpu: &mut GpuRenderer,
    x: f32,
    y: f32,
    weather: crate::types::WeatherKind,
    scale: f32,
    screen_width: f32,
    _screen_height: f32,
) {
    use crate::types::{BuildingKind, WeatherKind};
    use crate::game::production_weather_wmul;
    
    let s = scale as i32;
    let pad = (4 * s) as f32;
    let line_height = (12 * s) as f32;
    
    // Название погоды
    let (weather_name, weather_desc) = match weather {
        WeatherKind::Clear => ("Clear", "Normal weather conditions."),
        WeatherKind::Rain => ("Rain", "Rainy weather affects production."),
        WeatherKind::Fog => ("Fog", "Foggy weather reduces visibility."),
        WeatherKind::Snow => ("Snow", "Snowy weather slows down production."),
    };
    
    // Собираем список зданий с их модификаторами
    let buildings_list = [
        BuildingKind::Lumberjack,
        BuildingKind::Forester,
        BuildingKind::StoneQuarry,
        BuildingKind::ClayPit,
        BuildingKind::IronMine,
        BuildingKind::WheatField,
        BuildingKind::Mill,
        BuildingKind::Bakery,
        BuildingKind::Kiln,
        BuildingKind::Smelter,
        BuildingKind::Fishery,
    ];
    
    let mut effects: Vec<(&str, f32)> = Vec::new();
    for &bk in buildings_list.iter() {
        let multiplier = production_weather_wmul(weather, bk);
        if multiplier != 1.0 {
            let building_name = match bk {
                BuildingKind::Lumberjack => "Lumberjack",
                BuildingKind::Forester => "Forester",
                BuildingKind::StoneQuarry => "Quarry",
                BuildingKind::ClayPit => "Clay Pit",
                BuildingKind::IronMine => "Iron Mine",
                BuildingKind::WheatField => "Wheat Field",
                BuildingKind::Mill => "Mill",
                BuildingKind::Bakery => "Bakery",
                BuildingKind::Kiln => "Kiln",
                BuildingKind::Smelter => "Smelter",
                BuildingKind::Fishery => "Fishery",
                _ => "",
            };
            if !building_name.is_empty() {
                effects.push((building_name, multiplier));
            }
        }
    }
    
    // Вычисляем размер тултипа
    let name_w = weather_name.len() as f32 * 4.0 * 2.0 * scale;
    let desc_w = weather_desc.len() as f32 * 4.0 * 2.0 * scale;
    
    // Максимальная ширина для эффектов
    let mut max_effect_w: f32 = 0.0;
    for (name, mult) in effects.iter() {
        let mult_text = if *mult < 1.0 {
            format!("+{}%", ((1.0 - mult) * 100.0).round() as i32)
        } else {
            format!("-{}%", ((mult - 1.0) * 100.0).round() as i32)
        };
        let effect_w = (name.len() + mult_text.len() + 2) as f32 * 4.0 * 2.0 * scale;
        max_effect_w = max_effect_w.max(effect_w);
    }
    
    let tooltip_w = [name_w, desc_w, max_effect_w].iter().fold(0.0_f32, |a, &b| a.max(b)) + pad * 2.0;
    let tooltip_h = line_height * (2.0 + if effects.is_empty() { 1.0 } else { effects.len() as f32 }) + pad * 2.0;
    
    // Позиционируем тултип рядом с курсором (с проверкой границ экрана)
    let tooltip_x = (x + 20.0).min(screen_width - tooltip_w - 10.0);
    let tooltip_y = (y - tooltip_h - 10.0).max(10.0);
    
    // Фон тултипа
    gpu.add_ui_rect(tooltip_x, tooltip_y, tooltip_w, tooltip_h, [0.1, 0.1, 0.1, 0.9]);
    gpu.add_ui_rect(tooltip_x + 1.0, tooltip_y + 1.0, tooltip_w - 2.0, tooltip_h - 2.0, [0.2, 0.2, 0.2, 0.8]);
    
    // Текст тултипа
    let mut text_y = tooltip_y + pad;
    
    // Название погоды
    gpu.draw_text(tooltip_x + pad, text_y, weather_name.as_bytes(), [1.0, 1.0, 1.0, 1.0], scale);
    text_y += line_height;
    
    // Описание
    gpu.draw_text(tooltip_x + pad, text_y, weather_desc.as_bytes(), [0.8, 0.8, 0.8, 1.0], scale);
    text_y += line_height;
    
    // Эффекты на производство
    if !effects.is_empty() {
        text_y += pad * 0.5;
        for (name, mult) in effects.iter() {
            let mult_text = if *mult < 1.0 {
                format!("+{}%", ((1.0 - mult) * 100.0).round() as i32)
            } else {
                format!("-{}%", ((mult - 1.0) * 100.0).round() as i32)
            };
            
            // Цвет: зеленый для бафов (< 1.0), красный для дебафов (> 1.0)
            let effect_color = if *mult < 1.0 {
                [0.3, 1.0, 0.3, 1.0] // зеленый для ускорения
            } else {
                [1.0, 0.3, 0.3, 1.0] // красный для замедления
            };
            
            let effect_line = format!("{}: {}", name, mult_text);
            gpu.draw_text(tooltip_x + pad, text_y, effect_line.as_bytes(), effect_color, scale);
            text_y += line_height;
        }
    } else {
        // Если нет эффектов (Clear)
        gpu.draw_text(tooltip_x + pad, text_y, b"No production effects", [0.7, 0.7, 0.7, 1.0], scale);
    }
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
    _screen_height: f32,
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
    let value_w = format!("Current: {}", current_value).len() as f32 * 4.0 * 2.0 * scale;
    
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

/// Рендеринг консоли разработчика
pub fn draw_console_gpu(
    gpu: &mut GpuRenderer,
    fw: i32,
    fh: i32,
    s: i32,
    input: &str,
    log: &[String],
) {
    let pad = 8 * s;
    let px = 2 * s;
    let line_h = 5 * px + 4 * s;
    let lines_visible = 6usize;
    let height = pad + (lines_visible as i32) * line_h + pad + line_h; // лог + строка ввода
    let y0 = fh - height;
    
    // Фон консоли
    gpu.add_ui_rect(0.0, y0 as f32, fw as f32, height as f32, [0.0, 0.0, 0.0, 0.7]);
    
    // Последние строки лога
    let start = log.len().saturating_sub(lines_visible);
    let mut y = y0 + pad;
    for line in &log[start..] {
        gpu.draw_text(pad as f32, y as f32, line.as_bytes(), [0.86, 0.86, 0.86, 1.0], s as f32);
        y += line_h;
    }
    
    // Строка ввода с префиксом
    gpu.draw_text(pad as f32, y as f32, b"> ", [0.86, 0.86, 0.7, 1.0], s as f32);
    let prefix_w = ui::text_w(b"> ", s);
    gpu.draw_text((pad + prefix_w) as f32, y as f32, input.as_bytes(), [0.9, 0.9, 0.9, 1.0], s as f32);
}

/// Преобразование экранных координат в координаты тайла
fn screen_to_tile_px(
    mx: i32, 
    my: i32, 
    sw: i32, 
    sh: i32, 
    cam_px: glam::Vec2, 
    half_w: i32, 
    half_h: i32, 
    zoom: f32
) -> Option<glam::IVec2> {
    // экран -> мир (с учетом zoom и камеры)
    // GPU: world_x = (screen_x - sw/2) / zoom + cam_x
    //      world_y = -(screen_y - sh/2) / zoom - cam_y  (камера с +cam_y, view матрица)
    let wx = (mx - sw / 2) as f32 / zoom + cam_px.x;
    let wy = (my - sh / 2) as f32 / zoom + cam_px.y;
    
    let a = half_w as f32;
    let b = half_h as f32;
    // обратное к изометрической проекции: iso_x = (mx - my)*a, iso_y = (mx + my)*b
    let tx = 0.5 * (wy / b + wx / a) + 1.0;
    let ty = 0.5 * (wy / b - wx / a) + 1.0;
    let ix = tx.floor() as i32;
    let iy = ty.floor() as i32;
    Some(glam::IVec2::new(ix, iy))
}

/// Рендеринг тултипа отладки биома
pub fn draw_biome_debug_tooltip(
    gpu: &mut GpuRenderer,
    x: f32,
    y: f32,
    biome_name: &str,
    tile_x: i32,
    tile_y: i32,
    scale: f32,
    screen_width: f32,
    _screen_height: f32,
) {
    let pad = 8.0 * scale;
    let line_height = 16.0 * scale;
    
    // Текст тултипа
    let title = "Biome Debug";
    let biome_text = format!("Biome: {}", biome_name);
    let pos_text = format!("Position: ({}, {})", tile_x, tile_y);
    
    // Вычисляем размеры тултипа
    let title_w = ui::text_w(title.as_bytes(), scale as i32) as f32;
    let biome_w = ui::text_w(biome_text.as_bytes(), scale as i32) as f32;
    let pos_w = ui::text_w(pos_text.as_bytes(), scale as i32) as f32;
    let tooltip_w = (title_w.max(biome_w).max(pos_w) + pad * 2.0).max(120.0);
    let tooltip_h = line_height * 3.0 + pad * 2.0;
    
    // Позиционирование тултипа
    let mut tooltip_x = x + 10.0;
    let mut tooltip_y = y - tooltip_h - 10.0;
    
    // Ограничиваем тултип границами экрана
    if tooltip_x + tooltip_w > screen_width {
        tooltip_x = screen_width - tooltip_w - 10.0;
    }
    if tooltip_y < 0.0 {
        tooltip_y = y + 10.0;
    }
    
    // Фон тултипа
    gpu.add_ui_rect(tooltip_x, tooltip_y, tooltip_w, tooltip_h, [0.0, 0.0, 0.0, 0.8]);
    gpu.add_ui_rect(tooltip_x + 1.0, tooltip_y + 1.0, tooltip_w - 2.0, tooltip_h - 2.0, [0.2, 0.2, 0.2, 0.9]);
    
    // Текст тултипа
    let mut text_y = tooltip_y + pad;
    
    // Заголовок
    gpu.draw_text(tooltip_x + pad, text_y, title.as_bytes(), [1.0, 1.0, 0.0, 1.0], scale);
    text_y += line_height;
    
    // Биом
    gpu.draw_text(tooltip_x + pad, text_y, biome_text.as_bytes(), [0.8, 1.0, 0.8, 1.0], scale);
    text_y += line_height;
    
    // Позиция
    gpu.draw_text(tooltip_x + pad, text_y, pos_text.as_bytes(), [0.8, 0.8, 1.0, 1.0], scale);
}

/// Рисование окна дерева исследований
pub fn draw_research_tree_gpu(
    gpu: &mut GpuRenderer,
    fw: i32,
    fh: i32,
    research_system: &crate::research::ResearchSystem,
    _resources: &Resources,
    base_scale_k: f32,
    cursor_x: i32,
    cursor_y: i32,
    scroll_offset: f32,
) -> Option<crate::research::ResearchKind> {
    use crate::research::{ResearchKind, ResearchStatus};
    
    let s = ui::ui_scale(fh, base_scale_k);
    let scale = s as f32;
    
    // Полупрозрачный фон на весь экран
    gpu.add_ui_rect(0.0, 0.0, fw as f32, fh as f32, [0.0, 0.0, 0.0, 0.75]);
    
    // Окно дерева исследований (90% экрана по центру)
    let window_w = (fw as f32 * 0.9).max(900.0);
    let window_h = (fh as f32 * 0.85).max(650.0);
    let window_x = (fw as f32 - window_w) / 2.0;
    let window_y = (fh as f32 - window_h) / 2.0;
    
    // Тень окна для глубины
    let shadow_offset = 8.0;
    gpu.add_ui_rect(window_x + shadow_offset, window_y + shadow_offset, window_w, window_h, [0.0, 0.0, 0.0, 0.5]);
    
    // Фон окна с двойной рамкой
    gpu.add_ui_rect(window_x, window_y, window_w, window_h, [0.2, 0.3, 0.45, 0.98]);
    gpu.add_ui_rect(window_x + 4.0, window_y + 4.0, window_w - 8.0, window_h - 8.0, [0.15, 0.2, 0.3, 1.0]);
    gpu.add_ui_rect(window_x + 6.0, window_y + 6.0, window_w - 12.0, window_h - 12.0, [0.08, 0.1, 0.15, 1.0]);
    
    let pad = (16 * s) as f32;
    let title_height = (28 * s) as f32;
    
    // Полоса заголовка с градиентом
    let header_h = title_height + pad * 2.0;
    gpu.add_ui_rect(window_x + 6.0, window_y + 6.0, window_w - 12.0, header_h, [0.12, 0.18, 0.28, 1.0]);
    gpu.add_ui_rect(window_x + 6.0, window_y + 6.0, window_w - 12.0, header_h * 0.5, [0.15, 0.22, 0.35, 0.8]);
    
    // Разделитель под заголовком
    gpu.add_ui_rect(window_x + pad, window_y + header_h + 6.0, window_w - pad * 2.0, 2.0, [0.3, 0.5, 0.7, 0.6]);
    
    // Заголовок
    let title = if research_system.has_research_lab {
        "RESEARCH TREE"
    } else {
        "BUILD A LABORATORY FOR RESEARCH"
    };
    
    // Декоративные элементы по бокам заголовка
    let deco_w = 8.0;
    let deco_h = title_height * 0.6;
    let deco_y = window_y + pad + title_height * 0.2;
    gpu.add_ui_rect(window_x + pad - deco_w - 4.0, deco_y, deco_w, deco_h, [0.4, 0.7, 1.0, 0.8]);
    
    gpu.draw_text(window_x + pad, window_y + pad, title.as_bytes(), [0.95, 0.95, 0.5, 1.0], scale * 1.8);
    
    // Подсветка заголовка
    gpu.draw_text(window_x + pad - 1.0, window_y + pad - 1.0, title.as_bytes(), [1.0, 1.0, 0.8, 0.3], scale * 1.8);
    
    // Кнопка закрытия (крестик) в правом верхнем углу
    let close_btn_size = (20 * s) as f32;
    let close_btn_x = window_x + window_w - pad - close_btn_size;
    let close_btn_y = window_y + pad;
    
    // Фон кнопки
    let is_close_hovered = cursor_x >= close_btn_x as i32 && cursor_x < (close_btn_x + close_btn_size) as i32
        && cursor_y >= close_btn_y as i32 && cursor_y < (close_btn_y + close_btn_size) as i32;
    
    let close_bg_color = if is_close_hovered { [0.8, 0.2, 0.2, 0.9] } else { [0.5, 0.2, 0.2, 0.8] };
    gpu.add_ui_rect(close_btn_x, close_btn_y, close_btn_size, close_btn_size, close_bg_color);
    
    // Крестик X
    gpu.draw_text(close_btn_x + close_btn_size * 0.25, close_btn_y + close_btn_size * 0.2, 
                  b"X", [1.0, 1.0, 1.0, 1.0], scale * 1.5);
    
    if !research_system.has_research_lab {
        return None;
    }
    
    // Информация об активном исследовании с улучшенным дизайном
    let info_y = window_y + pad + title_height + pad + 12.0;
    if let Some(ref active) = research_system.active_research {
        let info = active.kind.info();
        
        let text_w = window_w - pad * 2.0;
        let info_h = (32 * s) as f32;
        
        // Фон панели активного исследования с градиентом
        gpu.add_ui_rect(window_x + pad, info_y, text_w, info_h, [0.15, 0.3, 0.45, 0.95]);
        gpu.add_ui_rect(window_x + pad + 2.0, info_y + 2.0, text_w - 4.0, info_h - 4.0, [0.08, 0.15, 0.25, 1.0]);
        
        // Акцентная полоска слева
        gpu.add_ui_rect(window_x + pad + 2.0, info_y + 2.0, 4.0, info_h - 4.0, [0.4, 0.8, 1.0, 0.9]);
        
        // Название исследования
        let progress_text = format!("ACTIVE: {}", info.name.to_uppercase());
        gpu.draw_text(window_x + pad + 12.0, info_y + 4.0, 
                      progress_text.as_bytes(), [0.5, 1.0, 1.0, 1.0], scale * 0.8);
        
        // Прогресс бар с улучшенным дизайном
        let total_days = info.days_required;
        let progress = if total_days > 0 {
            1.0 - (active.days_remaining as f32 / total_days as f32)
        } else {
            1.0
        };
        let bar_y = info_y + (16 * s) as f32;
        let bar_w = text_w - 20.0;
        let bar_h = (8 * s) as f32;
        
        // Фон прогресс бара с рамкой
        gpu.add_ui_rect(window_x + pad + 10.0, bar_y, bar_w, bar_h, [0.2, 0.25, 0.3, 0.9]);
        gpu.add_ui_rect(window_x + pad + 11.0, bar_y + 1.0, bar_w - 2.0, bar_h - 2.0, [0.05, 0.08, 0.12, 1.0]);
        
        // Заполненная часть с градиентом
        let filled_w = (bar_w - 2.0) * progress;
        if filled_w > 0.0 {
            gpu.add_ui_rect(window_x + pad + 11.0, bar_y + 1.0, filled_w, bar_h - 2.0, [0.3, 0.8, 1.0, 0.9]);
            gpu.add_ui_rect(window_x + pad + 11.0, bar_y + 1.0, filled_w, (bar_h - 2.0) * 0.4, [0.5, 0.95, 1.0, 0.7]);
        }
        
        // Текст прогресса
        let days_text = format!("{}/{} days ({}%)", 
            total_days - active.days_remaining, total_days, (progress * 100.0) as i32);
        let text_x = window_x + pad + 10.0 + (bar_w - days_text.len() as f32 * 4.0 * scale * 0.7) / 2.0;
        gpu.draw_text(text_x, bar_y + 1.0, days_text.as_bytes(), [1.0, 1.0, 1.0, 1.0], scale * 0.7);
    } else {
        // Подсказка когда нет активного исследования
        let hint_h = (20 * s) as f32;
        let text_w = window_w - pad * 2.0;
        
        gpu.add_ui_rect(window_x + pad, info_y, text_w, hint_h, [0.1, 0.15, 0.2, 0.7]);
        gpu.add_ui_rect(window_x + pad + 1.0, info_y + 1.0, text_w - 2.0, hint_h - 2.0, [0.08, 0.12, 0.18, 0.9]);
        
        let hint = "Select an available research to begin...";
        let hint_x = window_x + pad + (text_w - hint.len() as f32 * 4.0 * scale * 0.7) / 2.0;
        gpu.draw_text(hint_x, info_y + 5.0, 
                      hint.as_bytes(), [0.6, 0.7, 0.8, 1.0], scale * 0.7);
    }
    
    // Дерево исследований
    let tree_start_y = info_y + (40 * s) as f32; // Увеличенный отступ после панели
    let node_w = (140 * s) as f32; // Ширина узла (увеличена для лучших пропорций)
    let node_h = (80 * s) as f32;  // Высота узла (увеличена для лучших пропорций)
    let gap_x = (35 * s) as f32;   // Горизонтальный отступ между узлами
    let gap_y = (45 * s) as f32;   // Вертикальный отступ между узлами
    
    // Вычисляем общую ширину дерева
    let tree_cols = 4; // Максимум 4 колонки (от 0 до 3)
    let total_tree_width = (tree_cols as f32 * node_w) + ((tree_cols - 1) as f32 * gap_x);
    
    // Центрируем дерево по горизонтали
    let tree_left_margin = ((window_w - pad * 2.0) - total_tree_width) / 2.0;
    let tree_area_left = window_x + pad + tree_left_margin.max(0.0);
    
    // Границы области для отрисовки дерева (clipping)
    let tree_area_top = tree_start_y;
    let tree_area_bottom = window_y + window_h - pad * 2.0; // Отступ снизу
    let tree_area_height = tree_area_bottom - tree_area_top;
    let tree_area_right = window_x + window_w - pad;
    
    // Вычисляем максимальную высоту дерева
    let mut max_row = 0;
    for &research_kind in ResearchKind::all() {
        let (_, row) = research_kind.tree_position();
        if row > max_row {
            max_row = row;
        }
    }
    let total_tree_height = (max_row as f32 + 1.0) * (node_h + gap_y);
    
    let mut hovered_research = None;
    
    // Устанавливаем область клиппинга для дерева исследований
    gpu.set_clip_rect(
        tree_area_left,
        tree_area_top,
        tree_area_right - tree_area_left,
        tree_area_height
    );
    
    // Сначала рисуем все линии связей (чтобы они были под узлами)
    for &research_kind in ResearchKind::all() {
        let (col, row) = research_kind.tree_position();
        let status = research_system.get_status(research_kind);
        let info = research_kind.info();
        
        if status == ResearchStatus::Completed && info.days_required == 0 {
            continue;
        }
        
        let node_x = tree_area_left + (col as f32) * (node_w + gap_x);
        let node_y = tree_start_y + (row as f32) * (node_h + gap_y) - scroll_offset;
        
        for &prereq in info.prerequisites {
            let (prereq_col, prereq_row) = prereq.tree_position();
            let prereq_x = tree_area_left + (prereq_col as f32) * (node_w + gap_x) + node_w / 2.0;
            let prereq_y = tree_start_y + (prereq_row as f32) * (node_h + gap_y) + node_h - scroll_offset;
            let current_x = node_x + node_w / 2.0;
            let current_y = node_y;
            
            let prereq_status = research_system.get_status(prereq);
            let (line_color, line_glow) = if prereq_status == ResearchStatus::Completed {
                ([0.35, 1.0, 0.5, 0.85], [0.2, 0.6, 0.3, 0.3])
            } else {
                ([0.3, 0.3, 0.35, 0.6], [0.15, 0.15, 0.18, 0.2])
            };
            
            let line_width = 2.5 * scale;
            let glow_width = 5.0 * scale;
            let mid_y = (prereq_y + current_y) / 2.0;
            
            // Рисуем свечение (более широкую линию снизу)
            gpu.add_ui_rect(prereq_x - glow_width / 2.0, prereq_y, glow_width, mid_y - prereq_y, line_glow);
            let h_width = (current_x - prereq_x).abs();
            let h_x = prereq_x.min(current_x);
            gpu.add_ui_rect(h_x, mid_y - glow_width / 2.0, h_width, glow_width, line_glow);
            gpu.add_ui_rect(current_x - glow_width / 2.0, mid_y, glow_width, current_y - mid_y, line_glow);
            
            // Рисуем основную линию поверх свечения
            gpu.add_ui_rect(prereq_x - line_width / 2.0, prereq_y, line_width, mid_y - prereq_y, line_color);
            gpu.add_ui_rect(h_x, mid_y - line_width / 2.0, h_width, line_width, line_color);
            gpu.add_ui_rect(current_x - line_width / 2.0, mid_y, line_width, current_y - mid_y, line_color);
            
            // Рисуем стрелку на конце линии
            let arrow_size = 4.0 * scale;
            gpu.add_ui_rect(current_x - arrow_size, current_y - arrow_size, arrow_size * 2.0, arrow_size, line_color);
        }
    }
    
    // Теперь рисуем узлы
    for &research_kind in ResearchKind::all() {
        let (col, row) = research_kind.tree_position();
        let status = research_system.get_status(research_kind);
        let info = research_kind.info();
        
        if status == ResearchStatus::Completed && info.days_required == 0 {
            continue;
        }
        
        let node_x = tree_area_left + (col as f32) * (node_w + gap_x);
        let node_y = tree_start_y + (row as f32) * (node_h + gap_y) - scroll_offset;
        
        let is_hovered = cursor_x >= node_x as i32 && cursor_x < (node_x + node_w) as i32
            && cursor_y >= node_y as i32 && cursor_y < (node_y + node_h) as i32;
        
        if is_hovered {
            hovered_research = Some((research_kind, status, node_x, node_y + node_h));
        }
        
        // Цвета в зависимости от статуса
        let (bg_color, bg_color_top, border_color, text_color, status_text, status_color) = match status {
            ResearchStatus::Locked => (
                [0.12, 0.12, 0.15, 0.95],
                [0.18, 0.18, 0.22, 0.95],
                [0.3, 0.3, 0.35, 1.0],
                [0.5, 0.5, 0.55, 1.0],
                "LOCKED",
                [0.4, 0.4, 0.45, 1.0],
            ),
            ResearchStatus::Available => (
                [0.12, 0.22, 0.42, 0.98],
                [0.18, 0.32, 0.55, 0.98],
                [0.4, 0.65, 1.0, 1.0],
                [1.0, 1.0, 1.0, 1.0],
                "READY",
                [0.5, 1.0, 0.5, 1.0],
            ),
            ResearchStatus::InProgress => (
                [0.45, 0.3, 0.08, 0.98],
                [0.6, 0.42, 0.15, 0.98],
                [1.0, 0.75, 0.3, 1.0],
                [1.0, 1.0, 0.9, 1.0],
                "ACTIVE",
                [1.0, 0.95, 0.4, 1.0],
            ),
            ResearchStatus::Completed => (
                [0.08, 0.35, 0.12, 0.98],
                [0.12, 0.45, 0.18, 0.98],
                [0.35, 1.0, 0.45, 1.0],
                [0.85, 1.0, 0.85, 1.0],
                "DONE",
                [0.5, 1.0, 0.6, 1.0],
            ),
        };
        
        let hover_mult = if is_hovered && status == ResearchStatus::Available { 1.15 } else { 1.0 };
        let bg_bottom = [bg_color[0] * hover_mult, bg_color[1] * hover_mult, bg_color[2] * hover_mult, bg_color[3]];
        let bg_top = [bg_color_top[0] * hover_mult, bg_color_top[1] * hover_mult, bg_color_top[2] * hover_mult, bg_color_top[3]];
        let border_final = if is_hovered && status == ResearchStatus::Available {
            [border_color[0] * 1.15, border_color[1] * 1.15, border_color[2] * 1.15, border_color[3]]
        } else {
            border_color
        };
        
        // Тень для объемности
        let shadow_offset = 3.0;
        gpu.add_ui_rect(node_x + shadow_offset, node_y + shadow_offset, node_w, node_h, [0.0, 0.0, 0.0, 0.4]);
        
        // Рамка с увеличенной толщиной при наведении
        let border_width = if is_hovered { 3.5 } else { 2.5 };
        gpu.add_ui_rect(node_x, node_y, node_w, node_h, border_final);
        
        // Градиент фона (симуляция через несколько прямоугольников)
        let inner_x = node_x + border_width;
        let inner_y = node_y + border_width;
        let inner_w = node_w - border_width * 2.0;
        let inner_h = node_h - border_width * 2.0;
        
        // Делим на 3 части для градиента
        let grad_h = inner_h / 3.0;
        gpu.add_ui_rect(inner_x, inner_y, inner_w, grad_h, bg_top);
        gpu.add_ui_rect(inner_x, inner_y + grad_h, inner_w, grad_h, 
            [(bg_top[0] + bg_bottom[0]) / 2.0, (bg_top[1] + bg_bottom[1]) / 2.0, 
             (bg_top[2] + bg_bottom[2]) / 2.0, (bg_top[3] + bg_bottom[3]) / 2.0]);
        gpu.add_ui_rect(inner_x, inner_y + grad_h * 2.0, inner_w, grad_h, bg_bottom);
        
        // Внутренние отступы карточки
        let card_pad = (6 * s) as f32;
        
        // Полоска статуса в верхней части
        let status_bar_h = 3.0;
        gpu.add_ui_rect(inner_x, inner_y, inner_w, status_bar_h, status_color);
        
        // Название (более крупный шрифт)
        let name_lines = split_text(info.name, ((inner_w - card_pad * 2.0) / (4.0 * scale * 0.85)) as usize);
        let mut text_y = inner_y + status_bar_h + card_pad;
        for line in name_lines.iter().take(2) {
            gpu.draw_text(inner_x + card_pad, text_y, line.as_bytes(), text_color, scale * 0.85);
            text_y += (11 * s) as f32;
        }
        
        // Разделитель (на 2/3 высоты карточки)
        let separator_y = inner_y + inner_h * 0.62;
        gpu.add_ui_rect(inner_x + card_pad, separator_y, inner_w - card_pad * 2.0, 1.0, [1.0, 1.0, 1.0, 0.25]);
        
        // Информация о стоимости и времени
        if status != ResearchStatus::Completed {
            // Секция ресурсов
            let cost_y = separator_y + card_pad;
            
            // Иконки ресурсов
            let icon_size = (8 * s) as f32;
            let icon_gap = (4 * s) as f32;
            let mut icon_x = inner_x + card_pad;
            
            if info.cost.wood > 0 {
                // Рамка вокруг иконки
                gpu.add_ui_rect(icon_x, cost_y, icon_size, icon_size, [0.4, 0.3, 0.15, 1.0]);
                gpu.add_ui_rect(icon_x + 1.0, cost_y + 1.0, icon_size - 2.0, icon_size - 2.0, [0.7, 0.5, 0.3, 1.0]);
                
                let wood_text = format!("{}", info.cost.wood);
                let text_x = icon_x + icon_size + 3.0;
                gpu.draw_text(text_x, cost_y + 1.0, 
                              wood_text.as_bytes(), [1.0, 0.9, 0.6, 1.0], scale * 0.65);
                icon_x += icon_size + 3.0 + (wood_text.len() as f32 * 4.0 * scale * 0.65) + icon_gap;
            }
            
            if info.cost.gold > 0 {
                // Рамка вокруг иконки
                gpu.add_ui_rect(icon_x, cost_y, icon_size, icon_size, [0.7, 0.6, 0.0, 1.0]);
                gpu.add_ui_rect(icon_x + 1.0, cost_y + 1.0, icon_size - 2.0, icon_size - 2.0, [1.0, 0.9, 0.2, 1.0]);
                
                let gold_text = format!("{}", info.cost.gold);
                let text_x = icon_x + icon_size + 3.0;
                gpu.draw_text(text_x, cost_y + 1.0, 
                              gold_text.as_bytes(), [1.0, 1.0, 0.8, 1.0], scale * 0.65);
                icon_x += icon_size + 3.0 + (gold_text.len() as f32 * 4.0 * scale * 0.65) + icon_gap;
            }
            
            if info.cost.stone > 0 {
                // Рамка вокруг иконки
                gpu.add_ui_rect(icon_x, cost_y, icon_size, icon_size, [0.35, 0.35, 0.35, 1.0]);
                gpu.add_ui_rect(icon_x + 1.0, cost_y + 1.0, icon_size - 2.0, icon_size - 2.0, [0.6, 0.6, 0.6, 1.0]);
                
                let stone_text = format!("{}", info.cost.stone);
                let text_x = icon_x + icon_size + 3.0;
                gpu.draw_text(text_x, cost_y + 1.0, 
                              stone_text.as_bytes(), [0.9, 0.9, 0.9, 1.0], scale * 0.65);
            }
            
            // Время исследования (под иконками)
            if info.days_required > 0 {
                let time_y = cost_y + icon_size + (3 * s) as f32;
                let time_text = format!("{} days", info.days_required);
                let time_x = inner_x + card_pad;
                
                // Иконка часов (декоративный квадратик)
                let clock_size = (6 * s) as f32;
                gpu.add_ui_rect(time_x, time_y + 1.0, clock_size, clock_size, [0.4, 0.7, 1.0, 0.8]);
                
                gpu.draw_text(time_x + clock_size + 3.0, time_y, 
                              time_text.as_bytes(), [0.7, 0.9, 1.0, 1.0], scale * 0.6);
            }
        } else {
            // Для завершенных - показываем статус с иконкой
            let status_y = separator_y + card_pad + (2 * s) as f32;
            let check_size = (8 * s) as f32;
            
            // Иконка галочки (зеленый квадрат)
            gpu.add_ui_rect(inner_x + card_pad, status_y, check_size, check_size, [0.3, 0.8, 0.4, 1.0]);
            gpu.add_ui_rect(inner_x + card_pad + 1.0, status_y + 1.0, check_size - 2.0, check_size - 2.0, [0.5, 1.0, 0.6, 1.0]);
            
            gpu.draw_text(inner_x + card_pad + check_size + 4.0, status_y + 1.0, 
                          status_text.as_bytes(), status_color, scale * 0.7);
        }
        
    }
    
    // Очищаем область клиппинга перед рисованием тултипов и скроллбара
    gpu.clear_clip_rect();
    
    // Компактный тултип для наведенного исследования
    if let Some((kind, status, _x, _y)) = hovered_research {
        let info = kind.info();
        
        let tooltip_w = (240.0 * scale).min(fw as f32 * 0.32);
        let tooltip_pad = (6 * s) as f32;
        let section_gap = (4 * s) as f32;
        
        // Вычисляем высоту тултипа
        let mut tooltip_h = tooltip_pad * 2.0;
        
        // Заголовок
        let header_h = (10 * s) as f32;
        tooltip_h += header_h + section_gap;
        
        // Описание (одна строка, обрезается если нужно)
        let desc_h = (8 * s) as f32;
        tooltip_h += desc_h + section_gap;
        
        // Разблокирует (если есть)
        if !info.unlocks_buildings.is_empty() {
            tooltip_h += (7 * s) as f32 + (info.unlocks_buildings.len() as f32 * (7 * s) as f32) + section_gap;
        }
        
        // Требования (если есть)
        if !info.prerequisites.is_empty() {
            tooltip_h += (7 * s) as f32 + (info.prerequisites.len() as f32 * (7 * s) as f32) + section_gap;
        }
        
        // Стоимость и время
        if status != ResearchStatus::Completed {
            tooltip_h += (14 * s) as f32;
        }
        
        let tooltip_x = (cursor_x as f32 + 15.0).min(fw as f32 - tooltip_w - 10.0);
        let tooltip_y = (cursor_y as f32 + 15.0).min(fh as f32 - tooltip_h - 10.0);
        
        // Тень тултипа
        gpu.add_ui_rect(tooltip_x + 3.0, tooltip_y + 3.0, tooltip_w, tooltip_h, [0.0, 0.0, 0.0, 0.5]);
        
        // Фон тултипа
        gpu.add_ui_rect(tooltip_x, tooltip_y, tooltip_w, tooltip_h, [0.15, 0.22, 0.35, 0.98]);
        gpu.add_ui_rect(tooltip_x + 2.0, tooltip_y + 2.0, tooltip_w - 4.0, tooltip_h - 4.0, [0.08, 0.12, 0.18, 1.0]);
        
        let mut current_y = tooltip_y + tooltip_pad;
        
        // === ЗАГОЛОВОК ===
        let (status_text, status_color) = match status {
            ResearchStatus::Available => ("READY", [0.5, 1.0, 0.5, 1.0]),
            ResearchStatus::InProgress => ("ACTIVE", [1.0, 0.9, 0.4, 1.0]),
            ResearchStatus::Completed => ("DONE", [0.4, 1.0, 0.5, 1.0]),
            ResearchStatus::Locked => ("LOCKED", [0.6, 0.6, 0.6, 1.0]),
        };
        
        // Цветная полоска статуса
        gpu.add_ui_rect(tooltip_x + 2.0, current_y, tooltip_w - 4.0, 2.0, status_color);
        current_y += 4.0;
        
        // Название исследования
        gpu.draw_text(tooltip_x + tooltip_pad, current_y, 
                      info.name.as_bytes(), [1.0, 1.0, 1.0, 1.0], scale * 0.7);
        
        // Статус справа
        let status_x = tooltip_x + tooltip_w - tooltip_pad - (status_text.len() as f32 * 4.0 * scale * 0.5);
        gpu.draw_text(status_x, current_y + 1.0, 
                      status_text.as_bytes(), status_color, scale * 0.5);
        current_y += header_h + section_gap;
        
        // === ОПИСАНИЕ ===
        let max_desc_len = ((tooltip_w - tooltip_pad * 2.0) / (4.0 * scale * 0.55)) as usize;
        let desc = if info.description.len() > max_desc_len {
            format!("{}...", &info.description[..max_desc_len - 3])
        } else {
            info.description.to_string()
        };
        gpu.draw_text(tooltip_x + tooltip_pad, current_y, 
                      desc.as_bytes(), [0.8, 0.85, 0.9, 1.0], scale * 0.55);
        current_y += desc_h + section_gap;
        
        // === РАЗБЛОКИРУЕТ ===
        if !info.unlocks_buildings.is_empty() {
            // Разделитель
            gpu.add_ui_rect(tooltip_x + tooltip_pad, current_y, tooltip_w - tooltip_pad * 2.0, 1.0, [0.3, 0.4, 0.5, 0.5]);
            current_y += 2.0;
            
            gpu.draw_text(tooltip_x + tooltip_pad, current_y, 
                          b"UNLOCKS:", [0.5, 1.0, 0.6, 1.0], scale * 0.55);
            current_y += (7 * s) as f32;
            
            for building in info.unlocks_buildings {
                // Маркер
                gpu.add_ui_rect(tooltip_x + tooltip_pad + 3.0, current_y + 1.5, 2.5, 2.5, [0.5, 1.0, 0.6, 0.8]);
                
                let building_name = format!("{:?}", building);
                gpu.draw_text(tooltip_x + tooltip_pad + 8.0, current_y, 
                              building_name.as_bytes(), [0.8, 1.0, 0.85, 1.0], scale * 0.5);
                current_y += (7 * s) as f32;
            }
            current_y += section_gap;
        }
        
        // === ТРЕБОВАНИЯ ===
        if !info.prerequisites.is_empty() {
            // Разделитель
            gpu.add_ui_rect(tooltip_x + tooltip_pad, current_y, tooltip_w - tooltip_pad * 2.0, 1.0, [0.3, 0.4, 0.5, 0.5]);
            current_y += 2.0;
            
            gpu.draw_text(tooltip_x + tooltip_pad, current_y, 
                          b"REQUIRES:", [1.0, 0.7, 0.5, 1.0], scale * 0.55);
            current_y += (7 * s) as f32;
            
            for prereq in info.prerequisites {
                let prereq_info = prereq.info();
                let prereq_status = research_system.get_status(*prereq);
                let prereq_done = prereq_status == ResearchStatus::Completed;
                
                // Маркер (галочка или крестик)
                let marker_color = if prereq_done { 
                    [0.5, 1.0, 0.6, 0.9] 
                } else { 
                    [1.0, 0.5, 0.5, 0.9] 
                };
                gpu.add_ui_rect(tooltip_x + tooltip_pad + 3.0, current_y + 1.5, 2.5, 2.5, marker_color);
                
                let text_color = if prereq_done { 
                    [0.8, 1.0, 0.85, 1.0] 
                } else { 
                    [1.0, 0.8, 0.7, 1.0] 
                };
                gpu.draw_text(tooltip_x + tooltip_pad + 8.0, current_y, 
                              prereq_info.name.as_bytes(), text_color, scale * 0.5);
                current_y += (7 * s) as f32;
            }
            current_y += section_gap;
        }
        
        // === СТОИМОСТЬ И ВРЕМЯ ===
        if status != ResearchStatus::Completed {
            // Разделитель
            gpu.add_ui_rect(tooltip_x + tooltip_pad, current_y, tooltip_w - tooltip_pad * 2.0, 1.0, [0.3, 0.4, 0.5, 0.5]);
            current_y += 3.0;
            
            let icon_size = (8 * s) as f32;
            let mut icon_x = tooltip_x + tooltip_pad;
            
            // Иконки ресурсов
            if info.cost.wood > 0 {
                gpu.add_ui_rect(icon_x, current_y, icon_size, icon_size, [0.4, 0.3, 0.15, 1.0]);
                gpu.add_ui_rect(icon_x + 0.5, current_y + 0.5, icon_size - 1.0, icon_size - 1.0, [0.7, 0.5, 0.3, 1.0]);
                let text = format!("{}", info.cost.wood);
                gpu.draw_text(icon_x + icon_size + 2.0, current_y + 1.0, 
                              text.as_bytes(), [1.0, 0.9, 0.6, 1.0], scale * 0.55);
                icon_x += icon_size + 2.0 + (text.len() as f32 * 4.0 * scale * 0.55) + 6.0;
            }
            
            if info.cost.gold > 0 {
                gpu.add_ui_rect(icon_x, current_y, icon_size, icon_size, [0.7, 0.6, 0.0, 1.0]);
                gpu.add_ui_rect(icon_x + 0.5, current_y + 0.5, icon_size - 1.0, icon_size - 1.0, [1.0, 0.9, 0.2, 1.0]);
                let text = format!("{}", info.cost.gold);
                gpu.draw_text(icon_x + icon_size + 2.0, current_y + 1.0, 
                              text.as_bytes(), [1.0, 1.0, 0.8, 1.0], scale * 0.55);
                icon_x += icon_size + 2.0 + (text.len() as f32 * 4.0 * scale * 0.55) + 6.0;
            }
            
            if info.cost.stone > 0 {
                gpu.add_ui_rect(icon_x, current_y, icon_size, icon_size, [0.35, 0.35, 0.35, 1.0]);
                gpu.add_ui_rect(icon_x + 0.5, current_y + 0.5, icon_size - 1.0, icon_size - 1.0, [0.6, 0.6, 0.6, 1.0]);
                let text = format!("{}", info.cost.stone);
                gpu.draw_text(icon_x + icon_size + 2.0, current_y + 1.0, 
                              text.as_bytes(), [0.9, 0.9, 0.9, 1.0], scale * 0.55);
            }
            
            // Время справа
            if info.days_required > 0 {
                let time_text = format!("{}d", info.days_required);
                let time_x = tooltip_x + tooltip_w - tooltip_pad - (time_text.len() as f32 * 4.0 * scale * 0.55);
                
                // Иконка часов
                let clock_size = (6 * s) as f32;
                gpu.add_ui_rect(time_x - clock_size - 2.0, current_y + 1.0, clock_size, clock_size, [0.4, 0.7, 1.0, 0.8]);
                
                gpu.draw_text(time_x, current_y + 1.0, 
                              time_text.as_bytes(), [0.7, 0.9, 1.0, 1.0], scale * 0.55);
            }
        }
    }
    
    // Рисуем скроллбар если дерево больше чем доступная область
    if total_tree_height > tree_area_height {
        let scrollbar_width = (10 * s) as f32;
        let scrollbar_x = window_x + window_w - pad - scrollbar_width - 4.0;
        let scrollbar_track_y = tree_area_top;
        let scrollbar_track_h = tree_area_height;
        
        // Фон скроллбара (трек) с рамкой
        gpu.add_ui_rect(scrollbar_x, scrollbar_track_y, scrollbar_width, scrollbar_track_h, [0.15, 0.18, 0.22, 0.95]);
        gpu.add_ui_rect(scrollbar_x + 1.0, scrollbar_track_y + 1.0, scrollbar_width - 2.0, scrollbar_track_h - 2.0, [0.08, 0.1, 0.12, 1.0]);
        
        // Вычисляем размер и позицию ползунка
        let max_scroll = (total_tree_height - tree_area_height).max(0.0);
        let scroll_ratio = if max_scroll > 0.0 {
            scroll_offset / max_scroll
        } else {
            0.0
        };
        
        let thumb_height = (tree_area_height * (tree_area_height / total_tree_height)).max(40.0);
        let thumb_y = scrollbar_track_y + 2.0 + (scrollbar_track_h - thumb_height - 4.0) * scroll_ratio;
        
        // Ползунок скроллбара с градиентом
        gpu.add_ui_rect(scrollbar_x + 2.0, thumb_y, scrollbar_width - 4.0, thumb_height, [0.3, 0.4, 0.5, 1.0]);
        gpu.add_ui_rect(scrollbar_x + 3.0, thumb_y + 1.0, scrollbar_width - 6.0, thumb_height - 2.0, [0.4, 0.55, 0.7, 0.95]);
        gpu.add_ui_rect(scrollbar_x + 3.0, thumb_y + 1.0, scrollbar_width - 6.0, thumb_height * 0.4, [0.5, 0.65, 0.85, 0.7]);
        
        // Индикаторы на ползунке (декоративные линии)
        let line_y = thumb_y + thumb_height / 2.0 - 2.0;
        for i in 0..3 {
            let ly = line_y + (i as f32 * 2.0);
            gpu.add_ui_rect(scrollbar_x + 3.5, ly, scrollbar_width - 7.0, 1.0, [0.7, 0.8, 0.9, 0.5]);
        }
    }
    
    None // Функция больше не возвращает клики, только рендерит
}

/// Разделить текст на строки по максимальной ширине
fn split_text(text: &str, max_width: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in words {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else {
            let test_line = format!("{} {}", current_line, word);
            if test_line.len() <= max_width {
                current_line = test_line;
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    if lines.is_empty() {
        lines.push(text.to_string());
    }
    
    lines
}

/// Рисование уведомлений
pub fn draw_notifications_gpu(
    gpu: &mut GpuRenderer,
    fw: i32,
    fh: i32,
    notifications: &[crate::notifications::Notification],
    base_scale_k: f32,
) {
    let s = ui::ui_scale(fh, base_scale_k);
    let scale = s as f32;
    
    let pad = (8 * s) as f32;
    let notification_h = (40 * s) as f32;
    let notification_w = (400 * s) as f32;
    let gap = (8 * s) as f32;
    
    // Уведомления отображаются в верхнем правом углу
    let mut y = pad;
    
    for notification in notifications.iter().take(5) {
        let alpha = notification.alpha();
        let text = notification.text();
        
        // Цвет фона в зависимости от типа уведомления
        let bg_color = match &notification.kind {
            crate::notifications::NotificationKind::ResearchCompleted { .. } => {
                [0.1, 0.5, 0.8, 0.9 * alpha]
            }
            crate::notifications::NotificationKind::BuildingUnlocked { .. } => {
                [0.1, 0.7, 0.3, 0.9 * alpha]
            }
            crate::notifications::NotificationKind::Warning { .. } => {
                [0.8, 0.5, 0.1, 0.9 * alpha]
            }
            crate::notifications::NotificationKind::Info { .. } => {
                [0.3, 0.3, 0.3, 0.9 * alpha]
            }
        };
        
        let x = fw as f32 - notification_w - pad;
        
        // Фон уведомления
        gpu.add_ui_rect(x, y, notification_w, notification_h, [0.0, 0.0, 0.0, 0.8 * alpha]);
        gpu.add_ui_rect(x + 2.0, y + 2.0, notification_w - 4.0, notification_h - 4.0, bg_color);
        
        // Текст уведомления
        let text_color = [1.0, 1.0, 1.0, alpha];
        gpu.draw_text(x + pad, y + pad, text.as_bytes(), text_color, scale);
        
        y += notification_h + gap;
    }
}

