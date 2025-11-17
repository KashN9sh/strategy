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
            gpu.draw_button(current_x, build_y, btn_w, btn_h, label, selected == Some(*bk), btn_scale);
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
            );
        }
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
        BuildingKind::ResearchLab => ("Research Lab", "Research", None),
    };
    
    // Вычисляем размер тултипа
    let name_w = name.len() as f32 * 4.0 * 2.0 * scale;
    let prod_w = prod.len() as f32 * 4.0 * 2.0 * scale;
    let cons_w = cons.map(|c| c.len() as f32 * 4.0 * 2.0 * scale).unwrap_or(0.0);
    let workers_w = format!("Workers: {}/{}", workers_current, workers_target).len() as f32 * 4.0 * 2.0 * scale;
    
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
    _screen_height: f32,
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
        
        "Deposits" => ("Deposits", "Toggle resource deposits display on/off."),
        
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
) -> Option<crate::research::ResearchKind> {
    use crate::research::{ResearchKind, ResearchStatus};
    
    let s = ui::ui_scale(fh, base_scale_k);
    let scale = s as f32;
    
    // Полупрозрачный фон на весь экран
    gpu.add_ui_rect(0.0, 0.0, fw as f32, fh as f32, [0.0, 0.0, 0.0, 0.7]);
    
    // Окно дерева исследований (80% экрана по центру)
    let window_w = (fw as f32 * 0.8).max(800.0);
    let window_h = (fh as f32 * 0.8).max(600.0);
    let window_x = (fw as f32 - window_w) / 2.0;
    let window_y = (fh as f32 - window_h) / 2.0;
    
    // Фон окна
    gpu.add_ui_rect(window_x, window_y, window_w, window_h, [0.1, 0.1, 0.15, 0.95]);
    gpu.add_ui_rect(window_x + 2.0, window_y + 2.0, window_w - 4.0, window_h - 4.0, [0.15, 0.15, 0.2, 1.0]);
    
    let pad = (12 * s) as f32;
    let title_height = (30 * s) as f32;
    
    // Заголовок
    let title = if research_system.has_research_lab {
        "RESEARCH TREE"
    } else {
        "BUILD A LABORATORY FOR RESEARCH"
    };
    gpu.draw_text(window_x + pad, window_y + pad, title.as_bytes(), [1.0, 1.0, 0.0, 1.0], scale * 1.5);
    
    // Подсказка (T или ESC для закрытия)
    let hint = "Press T or ESC to close";
    gpu.draw_text(window_x + window_w - pad - (hint.len() as f32 * 4.0 * scale), 
                  window_y + pad, hint.as_bytes(), [0.7, 0.7, 0.7, 1.0], scale);
    
    if !research_system.has_research_lab {
        return None;
    }
    
    // Информация об активном исследовании
    if let Some(ref active) = research_system.active_research {
        let info = active.kind.info();
        let progress_text = format!("Researching: {} ({} days left)", info.name, active.days_remaining);
        gpu.draw_text(window_x + pad, window_y + title_height + pad, 
                      progress_text.as_bytes(), [0.0, 1.0, 1.0, 1.0], scale);
    } else {
        let hint = "Select research to begin";
        gpu.draw_text(window_x + pad, window_y + title_height + pad, 
                      hint.as_bytes(), [0.7, 0.7, 0.7, 1.0], scale);
    }
    
    // Дерево исследований
    let tree_start_y = window_y + title_height + pad * 3.0;
    let node_w = (120 * s) as f32;
    let node_h = (80 * s) as f32;
    let gap_x = (30 * s) as f32;
    let gap_y = (40 * s) as f32;
    
    let mut clicked_research = None;
    
    for &research_kind in ResearchKind::all() {
        let (col, row) = research_kind.tree_position();
        let status = research_system.get_status(research_kind);
        let info = research_kind.info();
        
        // Пропускаем завершенные базовые исследования (они не отображаются)
        if status == ResearchStatus::Completed && info.days_required == 0 {
            continue;
        }
        
        let node_x = window_x + pad + (col as f32) * (node_w + gap_x);
        let node_y = tree_start_y + (row as f32) * (node_h + gap_y);
        
        // Проверка, находится ли курсор над узлом
        let is_hovered = cursor_x >= node_x as i32 && cursor_x < (node_x + node_w) as i32
            && cursor_y >= node_y as i32 && cursor_y < (node_y + node_h) as i32;
        
        // Цвет узла в зависимости от статуса
        let (bg_color, border_color, text_color) = match status {
            ResearchStatus::Locked => (
                [0.2, 0.2, 0.2, 0.8],
                [0.3, 0.3, 0.3, 1.0],
                [0.5, 0.5, 0.5, 1.0],
            ),
            ResearchStatus::Available => (
                [0.2, 0.3, 0.5, 0.9],
                [0.3, 0.5, 0.8, 1.0],
                [1.0, 1.0, 1.0, 1.0],
            ),
            ResearchStatus::InProgress => (
                [0.5, 0.4, 0.1, 0.9],
                [0.8, 0.6, 0.1, 1.0],
                [1.0, 1.0, 0.0, 1.0],
            ),
            ResearchStatus::Completed => (
                [0.1, 0.5, 0.2, 0.9],
                [0.2, 0.8, 0.3, 1.0],
                [0.8, 1.0, 0.8, 1.0],
            ),
        };
        
        // Подсветка при наведении
        let (bg_final, border_final) = if is_hovered && status == ResearchStatus::Available {
            ([bg_color[0] + 0.1, bg_color[1] + 0.1, bg_color[2] + 0.1, bg_color[3]],
             [border_color[0] + 0.2, border_color[1] + 0.2, border_color[2] + 0.2, border_color[3]])
        } else {
            (bg_color, border_color)
        };
        
        // Рисуем узел
        gpu.add_ui_rect(node_x, node_y, node_w, node_h, border_final);
        gpu.add_ui_rect(node_x + 2.0, node_y + 2.0, node_w - 4.0, node_h - 4.0, bg_final);
        
        // Название (многострочное, если не помещается)
        let name_lines = split_text(info.name, (node_w / (4.0 * scale)) as usize);
        let mut text_y = node_y + pad / 2.0;
        for line in name_lines.iter().take(2) {
            gpu.draw_text(node_x + pad / 2.0, text_y, line.as_bytes(), text_color, scale * 0.9);
            text_y += (6 * s) as f32;
        }
        
        // Информация о стоимости и времени
        if status != ResearchStatus::Completed {
            let cost_text = format!("{}W {}G", info.cost.wood, info.cost.gold);
            gpu.draw_text(node_x + pad / 2.0, node_y + node_h - (20 * s) as f32, 
                          cost_text.as_bytes(), [0.8, 0.8, 0.0, 1.0], scale * 0.7);
            
            if info.days_required > 0 {
                let time_text = format!("{}d", info.days_required);
                gpu.draw_text(node_x + pad / 2.0, node_y + node_h - (10 * s) as f32, 
                              time_text.as_bytes(), [0.6, 0.8, 1.0, 1.0], scale * 0.7);
            }
        } else {
            gpu.draw_text(node_x + pad / 2.0, node_y + node_h - (12 * s) as f32, 
                          "ГОТОВО".as_bytes(), [0.0, 1.0, 0.0, 1.0], scale * 0.8);
        }
        
        // Проверка клика
        if is_hovered && status == ResearchStatus::Available {
            clicked_research = Some(research_kind);
        }
        
        // Рисуем линии связи к пререквизитам
        for &prereq in info.prerequisites {
            let (prereq_col, prereq_row) = prereq.tree_position();
            let prereq_x = window_x + pad + (prereq_col as f32) * (node_w + gap_x) + node_w / 2.0;
            let prereq_y = tree_start_y + (prereq_row as f32) * (node_h + gap_y) + node_h;
            let current_x = node_x + node_w / 2.0;
            let current_y = node_y;
            
            // Простая линия (можно улучшить)
            let line_color = if research_system.get_status(prereq) == ResearchStatus::Completed {
                [0.2, 0.8, 0.3, 0.6]
            } else {
                [0.3, 0.3, 0.3, 0.4]
            };
            
            // Вертикальная линия от пререквизита
            let mid_y = (prereq_y + current_y) / 2.0;
            gpu.add_ui_rect(prereq_x - 1.0, prereq_y, 2.0, mid_y - prereq_y, line_color);
            // Горизонтальная линия
            let h_width = (current_x - prereq_x).abs();
            let h_x = prereq_x.min(current_x);
            gpu.add_ui_rect(h_x, mid_y - 1.0, h_width, 2.0, line_color);
            // Вертикальная линия к текущему
            gpu.add_ui_rect(current_x - 1.0, mid_y, 2.0, current_y - mid_y, line_color);
        }
    }
    
    clicked_research
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

