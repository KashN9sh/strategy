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
}

