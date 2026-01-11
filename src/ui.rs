// Удалены неиспользуемые импорты (BuildingKind, FoodPolicy больше не нужны в layout функциях)

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum UICategory { 
    Housing, 
    Storage, 
    #[default]
    Forestry, 
    Mining, 
    Food, 
    Logistics, 
    Research 
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UITab { Build, Economy }

pub fn ui_gap(s: i32) -> i32 { 6 * s }
pub fn ui_pad(s: i32) -> i32 { 8 * s }
pub fn ui_item_h(s: i32) -> i32 { 18 * s }

pub fn ui_scale(fh: i32, k: f32) -> i32 { (((fh as f32) / 720.0) * k).clamp(1.0, 5.0) as i32 }

pub fn bottom_panel_height(s: i32) -> i32 {
    let padb = ui_pad(s); let btn_h = ui_item_h(s); let gap = ui_gap(s);
    // Tabs + Categories + Items (две вертикальные щели между тремя рядами)
    padb * 2 + btn_h * 3 + gap * 2
}

pub fn top_panel_height(s: i32) -> i32 {
    let pad = ui_pad(s); let icon = 10 * s; let px = 2 * s; let glyph_h = 5 * px; let gap = ui_gap(s);
    // Две строки контента (иконки/цифры) + отступ между ними
    pad * 2 + (icon.max(glyph_h)) * 2 + gap
}

#[derive(Clone, Copy, Debug)]
pub struct BuildingPanelLayout { pub minus_x: i32, pub minus_y: i32, pub minus_w: i32, pub minus_h: i32, pub plus_x: i32, pub plus_y: i32, pub plus_w: i32, pub plus_h: i32, pub dem_x: i32, pub dem_y: i32, pub dem_w: i32, pub dem_h: i32 }

pub fn layout_building_panel(fw: i32, fh: i32, s: i32) -> BuildingPanelLayout {
    let padb = 8 * s;
    let bottom_h = bottom_panel_height(s);
    // Компактная плашка слева, не на всю ширину
    let w = (fw as f32 * 0.33) as i32; // треть экрана
    // Высота панели из 4 строк: заголовок, workers, production, biome — с одинаковыми вертикальными зазорами
    let row_h = ui_item_h(s); let pad_top = ui_pad(s) - 2 * s; let pad_bottom = ui_pad(s) - 2 * s; let vgap = ui_gap(s);
    let panel_h = pad_top + row_h * 4 + vgap * 3 + pad_bottom;
    let x = padb;
    // Поднимем панель выше, чтобы не конфликтовала с миникартой и нижней панелью
    let y = fh - bottom_h - panel_h - 24 * s;
    // Кнопки +/- (высота как у общих кнопок)
    let minus_w = button_w_for(b"-", s); let minus_h = row_h; let plus_w = button_w_for(b"+", s); let plus_h = row_h;
    let minus_x = x + w - (plus_w + minus_w + 16 * s);
    // выравниваем по строке Workers — (row2)
    let workers_row_y = y + pad_top + row_h + vgap;
    let minus_y = workers_row_y;
    let plus_x = x + w - (plus_w + 10 * s);
    let plus_y = workers_row_y;
    // кнопка сноса — в той же строке, что и блок производства (row3)
    let dem_w = button_w_for(b"DEMOLISH", s); let dem_h = row_h;
    let dem_x = x + w - dem_w - 10 * s;
    let dem_y = y + pad_top + (row_h + vgap) * 2; // row3 y
    BuildingPanelLayout { minus_x, minus_y, minus_w, minus_h, plus_x, plus_y, plus_w, plus_h, dem_x, dem_y, dem_w, dem_h }
}


pub fn point_in_rect(px: i32, py: i32, x: i32, y: i32, w: i32, h: i32) -> bool { px >= x && py >= y && px < x + w && py < y + h }

pub fn button_w_for(label: &[u8], s: i32) -> i32 {
    let px = 2 * s; // ширина «пикселя» глифа
    let text_w = (label.len() as i32) * 4 * px; // 3x5 глиф с шагом 4
    text_w + 12 * s // паддинги
}

pub fn text_w(label: &[u8], s: i32) -> i32 { (label.len() as i32) * 4 * (2 * s) }


// ============================================================
// УДАЛЕНЫ CPU ФУНКЦИИ РЕНДЕРИНГА (больше не нужны в GPU версии)
// ============================================================
// - draw_building_panel
// - draw_ui  
// - draw_console
// - draw_button
// - draw_tooltip
// - draw_two_digits
// - draw_text_mini
// - draw_glyph_3x5
// - draw_number
// - fill_rect
// - draw_minimap_widget
// - ui_button_group
// - ui_text_group
// - resource_colors_for_building
//
// Вся логика UI теперь в ui_gpu.rs, которая использует GpuRenderer
// Сохранены только вспомогательные функции (ui_scale, button_w_for, layout_* и т.д.)
// ============================================================
