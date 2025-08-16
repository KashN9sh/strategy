use crate::types::{Resources, BuildingKind, building_cost, ResourceKind, FoodPolicy, Building};
use crate::world::World;
use glam::Vec2;
use crate::palette::resource_color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UICategory { Housing, Storage, Forestry, Mining, Food, Logistics }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UITab { Build, Economy }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonStyle { Default, Primary, Danger }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiDir { Row, Column }

#[derive(Clone, Copy, Debug)]
pub struct UiGroup { pub x: i32, pub y: i32, pub dir: UiDir, pub cursor_x: i32, pub cursor_y: i32, pub gap: i32, pub item_h: i32, pub s: i32 }

pub fn ui_gap(s: i32) -> i32 { 6 * s }
pub fn ui_pad(s: i32) -> i32 { 8 * s }
pub fn ui_item_h(s: i32) -> i32 { 18 * s }

pub fn ui_row(x: i32, y: i32, s: i32) -> UiGroup { UiGroup { x, y, dir: UiDir::Row, cursor_x: x, cursor_y: y, gap: ui_gap(s), item_h: ui_item_h(s), s } }
pub fn ui_column(x: i32, y: i32, s: i32) -> UiGroup { UiGroup { x, y, dir: UiDir::Column, cursor_x: x, cursor_y: y, gap: ui_gap(s), item_h: ui_item_h(s), s } }

pub fn ui_set_gap(g: &mut UiGroup, gap: i32) { g.gap = gap; }

pub fn ui_button_group(
    frame: &mut [u8], fw: i32, fh: i32,
    g: &mut UiGroup,
    label: &[u8], active: bool, hovered: bool, enabled: bool, label_col: [u8;4], style: ButtonStyle,
) -> (i32, i32, i32, i32) {
    let s = g.s; let w = button_w_for(label, s); let h = g.item_h;
    let (x, y) = match g.dir { UiDir::Row => (g.cursor_x, g.y), UiDir::Column => (g.x, g.cursor_y) };
    draw_button(frame, fw, fh, x, y, w, h, active, hovered, enabled, label, label_col, s, style);
    match g.dir { UiDir::Row => { g.cursor_x += w + g.gap; }, UiDir::Column => { g.cursor_y += h + g.gap; } }
    (x, y, w, h)
}

pub fn ui_text_group(frame: &mut [u8], fw: i32, fh: i32, g: &mut UiGroup, text: &[u8], color: [u8;4]) -> (i32, i32, i32, i32) {
    let s = g.s; let px = 2 * s; let text_h = 5 * px; let w = text_w(text, s) + 12 * s; let h = g.item_h;
    let (x, y) = match g.dir { UiDir::Row => (g.cursor_x, g.y), UiDir::Column => (g.x, g.cursor_y) };
    let ty = y + (h - text_h) / 2;
    draw_text_mini(frame, fw, fh, x + 6 * s, ty, text, color, s);
    match g.dir { UiDir::Row => { g.cursor_x += w + g.gap; }, UiDir::Column => { g.cursor_y += h + g.gap; } }
    (x, y, w, h)
}

/// Зарезервировать прямоугольник в группе фиксированной ширины и высоты item_h, с продвижением курсора
pub fn ui_slot(g: &mut UiGroup, w: i32) -> (i32, i32, i32, i32) {
    let h = g.item_h;
    let (x, y) = match g.dir { UiDir::Row => (g.cursor_x, g.y), UiDir::Column => (g.x, g.cursor_y) };
    match g.dir { UiDir::Row => { g.cursor_x += w + g.gap; }, UiDir::Column => { g.cursor_y += h + g.gap; } }
    (x, y, w, h)
}

/// Резерв прямоугольника произвольной высоты в группе
pub fn ui_slot_wh(g: &mut UiGroup, w: i32, h: i32) -> (i32, i32, i32, i32) {
    let (x, y) = match g.dir { UiDir::Row => (g.cursor_x, g.y), UiDir::Column => (g.x, g.cursor_y) };
    match g.dir { UiDir::Row => { g.cursor_x += w + g.gap; }, UiDir::Column => { g.cursor_y += h + g.gap; } }
    (x, y, w, h)
}

pub fn ui_scale(fh: i32, k: f32) -> i32 { (((fh as f32) / 720.0) * k).clamp(1.0, 5.0) as i32 }
// удалено: ui_bar_height (не используется)
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
pub struct BuildingPanelLayout { pub x: i32, pub y: i32, pub w: i32, pub h: i32, pub minus_x: i32, pub minus_y: i32, pub minus_w: i32, pub minus_h: i32, pub plus_x: i32, pub plus_y: i32, pub plus_w: i32, pub plus_h: i32, pub dem_x: i32, pub dem_y: i32, pub dem_w: i32, pub dem_h: i32 }

pub fn layout_building_panel(fw: i32, fh: i32, s: i32) -> BuildingPanelLayout {
    let padb = 8 * s;
    let bottom_h = bottom_panel_height(s);
    // Компактная плашка слева, не на всю ширину
    let w = (fw as f32 * 0.33) as i32; // треть экрана
    // Высота панели из 3 строк: row_h*3 + vgap*2 + верх/низ отступы
    let row_h = ui_item_h(s); let pad_top = ui_pad(s) - 2 * s; let pad_bottom = ui_pad(s) - 2 * s; let vgap = ui_gap(s);
    let panel_h = pad_top + row_h * 3 + vgap * 2 + pad_bottom;
    let x = padb;
    let y = fh - bottom_h - panel_h - 6 * s;
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
    BuildingPanelLayout { x, y, w, h: panel_h, minus_x, minus_y, minus_w, minus_h, plus_x, plus_y, plus_w, plus_h, dem_x, dem_y, dem_w, dem_h }
}

fn resource_colors_for_building(kind: BuildingKind) -> (Option<[u8;4]>, Vec<[u8;4]>) {
    // цвета из верхней панели
    let wood = [110,70,30,255];
    let stone = [120,120,120,255];
    let clay = [150,90,70,255];
    let bricks = [180,120,90,255];
    let wheat = [200,180,80,255];
    let flour = [210,210,180,255];
    let bread = [200,160,120,255];
    let fish = [100,140,200,255];
    let iron_ore = [90,90,110,255];
    let iron_ingot = [190,190,210,255];
    match kind {
        BuildingKind::Lumberjack => (Some(wood), vec![]),
        BuildingKind::StoneQuarry => (Some(stone), vec![]),
        BuildingKind::ClayPit => (Some(clay), vec![]),
        BuildingKind::IronMine => (Some(iron_ore), vec![]),
        BuildingKind::WheatField => (Some(wheat), vec![]),
        BuildingKind::Mill => (Some(flour), vec![wheat]),
        BuildingKind::Kiln => (Some(bricks), vec![clay, wood]),
        BuildingKind::Bakery => (Some(bread), vec![flour, wood]),
        BuildingKind::Fishery => (Some(fish), vec![]),
        BuildingKind::Smelter => (Some(iron_ingot), vec![iron_ore, wood]),
        _ => (None, vec![]),
    }
}

pub fn draw_building_panel(
    frame: &mut [u8], fw: i32, fh: i32, s: i32,
    kind: BuildingKind, workers_current: i32, workers_target: i32,
    prod_label: &[u8], cons_label: Option<&[u8]>,
) {
    let layout = layout_building_panel(fw, fh, s);
    // фон
    fill_rect(frame, fw, fh, layout.x + 2 * s, layout.y + 2 * s, layout.w, layout.h, [0,0,0,100]);
    fill_rect(frame, fw, fh, layout.x, layout.y, layout.w, layout.h, [0,0,0,160]);
    let pad = 8 * s; let px = 2 * s; let icon = 10 * s;
    // 1) Заголовок — отдельный ui_row
    let mut row = ui_row(layout.x + pad, layout.y + ui_pad(s) - 2 * s, s);
    let title = match kind {
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
    ui_text_group(frame, fw, fh, &mut row, title, [230,230,230,255]);

    // 2) Workers + +/- — отдельный ui_row, с вертикальным зазором между строками
    let mut row2 = ui_row(layout.x + pad, layout.y + ui_pad(s) - 2 * s + ui_item_h(s) + ui_gap(s), s);
    ui_set_gap(&mut row2, 10 * s);
    ui_text_group(frame, fw, fh, &mut row2, b"Workers", [200,200,200,255]);
    let cur_txt = format!("{}", workers_current.max(0));
    ui_text_group(frame, fw, fh, &mut row2, cur_txt.as_bytes(), [255,255,255,255]);
    ui_text_group(frame, fw, fh, &mut row2, b"/", [180,180,180,255]);
    let tgt_txt = format!("{}", workers_target.max(0));
    ui_text_group(frame, fw, fh, &mut row2, tgt_txt.as_bytes(), [220,220,220,255]);
    // справа — две кнопки в том же ряду
    let right_x = layout.plus_x - (button_w_for(b"-", s) + 10 * s);
    let mut controls = ui_row(right_x, layout.minus_y, s);
    ui_button_group(frame, fw, fh, &mut controls, b"-", false, false, true, [230,230,230,255], ButtonStyle::Default);
    ui_button_group(frame, fw, fh, &mut controls, b"+", false, false, true, [230,230,230,255], ButtonStyle::Default);

    // 3) Производство слева + Demolish справа — один ui_row, ещё ниже с тем же вертикальным шагом
    let mut row3 = ui_row(layout.x + pad, layout.y + ui_pad(s) - 2 * s + (ui_item_h(s) + ui_gap(s)) * 2, s);
    ui_set_gap(&mut row3, 12 * s);
    if let Some(col) = resource_colors_for_building(kind).0 {
        let (x, y, _w, _h) = ui_slot(&mut row3, icon + 6 * s);
        fill_rect(frame, fw, fh, x, y + (row3.item_h - icon)/2, icon, icon, col);
    }
    ui_text_group(frame, fw, fh, &mut row3, prod_label, [220,220,220,255]);
    if let Some(c) = cons_label { ui_text_group(frame, fw, fh, &mut row3, c, [200,200,200,255]); }
    // Demolish в конце ряда (справа)
    let dem_left = layout.dem_x; // вычислен в layout на этой же строке
    let mut right_row = ui_row(dem_left, layout.dem_y, s);
    ui_button_group(frame, fw, fh, &mut right_row, b"DEMOLISH", false, false, true, [240,230,230,255], ButtonStyle::Danger);
}

#[derive(Clone, Copy)]
pub struct EconomyLayout { pub x:i32, pub y:i32, pub w:i32, pub h:i32, pub tax_minus_x:i32, pub tax_minus_y:i32, pub tax_minus_w:i32, pub tax_minus_h:i32, pub tax_plus_x:i32, pub tax_plus_y:i32, pub tax_plus_w:i32, pub tax_plus_h:i32, pub policy_bal_x:i32, pub policy_bal_y:i32, pub policy_bal_w:i32, pub policy_bal_h:i32, pub policy_bread_x:i32, pub policy_bread_y:i32, pub policy_bread_w:i32, pub policy_bread_h:i32, pub policy_fish_x:i32, pub policy_fish_y:i32, pub policy_fish_w:i32, pub policy_fish_h:i32 }

pub fn layout_economy_panel(fw: i32, fh: i32, s: i32) -> EconomyLayout {
    let padb = 8 * s; let btn_h = 18 * s; let bottom_h = bottom_panel_height(s); let by0 = fh - bottom_h;
    let x = padb; let y = by0 + padb + btn_h + 6 * s; let w = (fw - 2 * padb).max(0); let h = bottom_h - (y - by0) - padb;
    // размеры и стиль как у кнопок в меню строительства
    let minus_w = button_w_for(b"-", s); let minus_h = btn_h; let plus_w = button_w_for(b"+", s); let plus_h = btn_h;
    // Tax label width influences button positions
    let tax_label_w = text_w(b"TAX", s) + 6 * s; // запас под ':'
    let tax_minus_x = x + tax_label_w + 8 * s; let tax_minus_y = y;
    let tax_plus_x = tax_minus_x + minus_w + 6 * s; let tax_plus_y = y;
    // Политики — справа от блока налога (в одну строку)
    let reserved_num_w = number_w(100, s); // до 100%
    let policy_label_w = text_w(b"FOOD POLICY", s);
    let policy_label_x = tax_plus_x + plus_w + 8 * s + reserved_num_w + 20 * s; // старт позиции метки
    let policy_bal_x = policy_label_x + policy_label_w + 8 * s; let policy_bal_y = y;
    let policy_bal_w = button_w_for(b"Balanced", s); let policy_bal_h = btn_h;
    let policy_bread_x = policy_bal_x + policy_bal_w + 6 * s; let policy_bread_y = policy_bal_y; let policy_bread_w = button_w_for(b"Bread", s); let policy_bread_h = btn_h;
    let policy_fish_x = policy_bread_x + policy_bread_w + 6 * s; let policy_fish_y = policy_bal_y; let policy_fish_w = button_w_for(b"Fish", s); let policy_fish_h = btn_h;
    EconomyLayout { x, y, w, h, tax_minus_x, tax_minus_y, tax_minus_w: minus_w, tax_minus_h: minus_h, tax_plus_x, tax_plus_y, tax_plus_w: plus_w, tax_plus_h: plus_h, policy_bal_x, policy_bal_y, policy_bal_w, policy_bal_h, policy_bread_x, policy_bread_y, policy_bread_w, policy_bread_h, policy_fish_x, policy_fish_y, policy_fish_w, policy_fish_h }
}

pub fn draw_ui(
    frame: &mut [u8], fw: i32, fh: i32,
    resources: &Resources, total_wood: i32, population: i32, selected: BuildingKind,
    fps: f32, speed: f32, paused: bool, base_scale_k: f32, category: UICategory, day_progress_01: f32,
    citizens_idle: i32, citizens_working: i32, citizens_sleeping: i32, citizens_hauling: i32, citizens_fetching: i32,
    cursor_x: i32, cursor_y: i32,
    avg_happiness: f32, tax_rate: f32,
    ui_tab: UITab, food_policy: FoodPolicy,
    last_income: i32, last_upkeep: i32,
    housing_used: i32, housing_cap: i32,
    weather_label: &[u8], weather_icon_col: [u8;4],
) {
    let s = ui_scale(fh, base_scale_k);
    let bar_h = top_panel_height(s);
    fill_rect(frame, fw, fh, 0, 0, fw, bar_h, [0, 0, 0, 160]);

    let pad = 8 * s;
    let icon_size = 10 * s;
    let mut tooltip: Option<(i32, Vec<u8>)> = None;
    // Левая группа верхней панели: Population, Gold, Happiness, Tax
    let mut left = ui_row(pad, pad, s);
    // население
    {
        let (x, y, w, h) = ui_slot(&mut left, icon_size + 4 + number_w(9999, s) + 6 * s);
        fill_rect(frame, fw, fh, x, y, icon_size, icon_size, [180,60,60,255]);
        draw_number(frame, fw, fh, x + icon_size + 4, y, population.max(0) as u32, [255,255,255,255], s);
        if point_in_rect(cursor_x, cursor_y, x, y, icon_size, icon_size) { tooltip = Some((x + icon_size / 2, b"Population".to_vec())); }
    }
    // золото
    {
        let (x, y, _w, _h) = ui_slot(&mut left, icon_size + 4 + number_w(9999, s) + 6 * s);
        fill_rect(frame, fw, fh, x, y, icon_size, icon_size, [220,180,60,255]);
        draw_number(frame, fw, fh, x + icon_size + 4, y, resources.gold.max(0) as u32, [255,255,255,255], s);
        if point_in_rect(cursor_x, cursor_y, x, y, icon_size, icon_size) { tooltip = Some((x + icon_size / 2, b"Gold".to_vec())); }
    }
    // счастье
    {
        let (x, y, _w, _h) = ui_slot(&mut left, icon_size + 4 + number_w(100, s) + 6 * s);
        fill_rect(frame, fw, fh, x, y, icon_size, icon_size, [220,120,160,255]);
        let hap = avg_happiness.round().clamp(0.0, 100.0) as u32;
        draw_number(frame, fw, fh, x + icon_size + 4, y, hap, [255,255,255,255], s);
        if point_in_rect(cursor_x, cursor_y, x, y, icon_size, icon_size) { tooltip = Some((x + icon_size / 2, b"Happiness".to_vec())); }
    }
    // налог
    {
        let (x, y, _w, _h) = ui_slot(&mut left, icon_size + 4 + number_w(100, s) + 6 * s);
        fill_rect(frame, fw, fh, x, y, icon_size, icon_size, [200,180,90,255]);
        let taxp = (tax_rate * 100.0).round().clamp(0.0, 100.0) as u32;
        draw_number(frame, fw, fh, x + icon_size + 4, y, taxp, [255,255,255,255], s);
        if point_in_rect(cursor_x, cursor_y, x, y, icon_size, icon_size) { tooltip = Some((x + icon_size / 2, b"Tax %  [ [ ] to change ]".to_vec())); }
    }

    // погода (иконка + короткий лейбл) — рендер справа, возле блока TIME, а не в левом блоке
    let px = 2 * s; let gap_info = 2 * px; let y_info = 8 * s; let pad_right = 8 * s;
    let mut minutes_tmp = (day_progress_01.clamp(0.0, 1.0) * 1440.0).round() as i32 % 1440;
    let _hours_tmp = (minutes_tmp / 60) as u32; minutes_tmp = minutes_tmp % 60; let _minutes_u_tmp = minutes_tmp as u32;
    let fps_n_tmp = fps.round() as u32;
    let fps_w_tmp = text_w(b"FPS", s) + gap_info + number_w(fps_n_tmp, s);
    let speed_w_tmp = if paused { text_w(b"PAUSE", s) } else { let sp = (speed * 10.0).round() as u32; text_w(b"SPEED", s) + gap_info + number_w(sp, s) };
    let time_w_tmp = text_w(b"TIME", s) + gap_info + (5 * 4 * px);
    let total_w_tmp = time_w_tmp + gap_info + fps_w_tmp + gap_info + speed_w_tmp;
    let right_x0 = fw - pad_right - total_w_tmp;
    // погода слева от правого инфо-блока — через ui_row
    let wx = (right_x0 - (icon_size + 4 + text_w(weather_label, s) + 10 * s)).max(left.cursor_x);
    let mut weather_row = ui_row(wx, pad, s);
    let (sx, sy, _w, _h) = ui_slot(&mut weather_row, icon_size + 4 + text_w(weather_label, s) + 6 * s);
    fill_rect(frame, fw, fh, sx, sy, icon_size, icon_size, weather_icon_col);
    draw_text_mini(frame, fw, fh, sx + icon_size + 4, sy, weather_label, [230,230,230,255], s);
    if point_in_rect(cursor_x, cursor_y, sx, sy, icon_size, icon_size) { let mut tip = Vec::from(b"Weather: "); tip.extend_from_slice(weather_label); tooltip = Some((sx + icon_size / 2, tip)); }

    // Рассчитаем занятый справа блок (TIME/FPS/SPEED), чтобы не наезжать на него слева
    let px = 2 * s; let gap_info = 2 * px; let _y_info = 8 * s; let pad_right = 8 * s;
    let mut minutes_tmp = (day_progress_01.clamp(0.0, 1.0) * 1440.0).round() as i32 % 1440;
    let _hours_tmp = (minutes_tmp / 60) as u32; minutes_tmp = minutes_tmp % 60; let _minutes_u_tmp = minutes_tmp as u32;
    let fps_n_tmp = fps.round() as u32;
    let fps_w_tmp = text_w(b"FPS", s) + gap_info + number_w(fps_n_tmp, s);
    let speed_w_tmp = if paused { text_w(b"PAUSE", s) } else { let sp = (speed * 10.0).round() as u32; text_w(b"SPEED", s) + gap_info + number_w(sp, s) };
    let time_w_tmp = text_w(b"TIME", s) + gap_info + (5 * 4 * px);
    let total_w_tmp = time_w_tmp + gap_info + fps_w_tmp + gap_info + speed_w_tmp;
    let right_x0 = fw - pad_right - total_w_tmp;

    // Правый инфо-блок: TIME | FPS | SPEED/PAUSE — единым рядом
    {
        let mut info_row = ui_row(right_x0, pad, s);
        // TIME HH:MM
        let (ix, iy, _iw, _ih) = ui_slot(&mut info_row, time_w_tmp);
        draw_text_mini(frame, fw, fh, ix, iy, b"TIME", [220,220,220,255], s);
        // вычисление текущего времени повторно, чтобы не полагаться на подчёркнутые переменные
        let mut minutes_tmp = (day_progress_01.clamp(0.0, 1.0) * 1440.0).round() as i32 % 1440;
        let hours_u = (minutes_tmp / 60) as u32; minutes_tmp = minutes_tmp % 60; let minutes_u = minutes_tmp as u32;
        let label_w = text_w(b"TIME", s) + gap_info;
        let mut cx = ix + label_w;
        cx += draw_two_digits(frame, fw, fh, cx, iy, hours_u, [230,230,230,255], s);
        draw_glyph_3x5(frame, fw, fh, cx, iy, b':', [230,230,230,255], 2 * s); cx += 4 * (2 * s);
        cx += draw_two_digits(frame, fw, fh, cx, iy, minutes_u, [230,230,230,255], s);
        // FPS
        let (fx, fy, _fw2, _fh2) = ui_slot(&mut info_row, fps_w_tmp);
        draw_text_mini(frame, fw, fh, fx, fy, b"FPS", [220,220,220,255], s);
        let cx_fps = fx + text_w(b"FPS", s) + gap_info;
        draw_number(frame, fw, fh, cx_fps, fy, fps_n_tmp, [230,230,230,255], s);
        // SPEED / PAUSE
        let (sx, sy, _sw2, _sh2) = ui_slot(&mut info_row, speed_w_tmp);
        if paused {
            draw_text_mini(frame, fw, fh, sx, sy, b"PAUSE", [230,200,200,255], s);
        } else {
            draw_text_mini(frame, fw, fh, sx, sy, b"SPEED", [220,220,220,255], s);
            let sp = (speed * 10.0).round() as u32;
            let cx_sp = sx + text_w(b"SPEED", s) + gap_info;
            draw_number(frame, fw, fh, cx_sp, sy, sp, [230,230,230,255], s);
        }
    }

    // Блок статистики жителей (idle/working/sleeping/haul/fetch)
    // блок статусов жителей — ui_row на правой части, до right_x0
    let mut stat_row = ui_row(left.cursor_x, pad, s);
    let mut stat = |col: [u8;4], val: i32, label: &[u8]| {
        let (x, y, _w, _h) = ui_slot(&mut stat_row, icon_size + 4 + number_w(999, s) + 6 * s);
        if x + icon_size > right_x0 { return; }
        fill_rect(frame, fw, fh, x, y, icon_size, icon_size, col);
        draw_number(frame, fw, fh, x + icon_size + 4, y, val.max(0) as u32, [255,255,255,255], s);
        if point_in_rect(cursor_x, cursor_y, x, y, icon_size, icon_size) { tooltip = Some((x + icon_size / 2, label.to_vec())); }
    };
    stat([70,160,70,255], citizens_idle, b"Idle");
    stat([70,110,200,255], citizens_working, b"Working");
    stat([120,120,120,255], citizens_sleeping, b"Sleeping");
    stat([210,150,70,255], citizens_hauling, b"Hauling");
    stat([80,200,200,255], citizens_fetching, b"Fetching");

    // Вторая строка: ресурсы — сразу под первой строкой
    // Панель дополнительных ресурсов (камень, глина, кирпич, пшеница, мука, хлеб, рыба)
    let rx = pad;
    let ry = pad + icon_size + 6 * s;
    let mut entry = |frame: &mut [u8], x: &mut i32, _label_color: [u8;4], val: i32, label: &[u8], s: i32, kind: ResourceKind| {
        let px = 2 * s; let reserve_digits = 4; let reserved_w = reserve_digits * 4 * px; let gap = 6 * s;
        let need_w = icon_size + 4 + reserved_w + gap;
        if *x + need_w > right_x0 { return; }
        fill_rect(frame, fw, fh, *x, ry, icon_size, icon_size, resource_color(kind));
        let num_x = *x + icon_size + 4;
        draw_number(frame, fw, fh, num_x, ry, val.max(0) as u32, [255,255,255,255], s);
        if point_in_rect(cursor_x, cursor_y, *x, ry, icon_size, icon_size) {
            let mut text = Vec::new();
            text.extend_from_slice(label);
            text.extend_from_slice(b": ");
            let s_val = format!("{}", val.max(0));
            text.extend_from_slice(s_val.as_bytes());
            tooltip = Some((*x + icon_size / 2, text));
        }
        *x = num_x + reserved_w + gap;
    };
    let mut xcur = rx;
    // Дерево (из складов или видимое общее)
    entry(frame, &mut xcur, [0,0,0,0], total_wood, b"Wood", s, ResourceKind::Wood);
    entry(frame, &mut xcur, [0,0,0,0], resources.stone, b"Stone", s, ResourceKind::Stone);
    entry(frame, &mut xcur, [0,0,0,0], resources.clay, b"Clay", s, ResourceKind::Clay);
    entry(frame, &mut xcur, [0,0,0,0], resources.bricks, b"Bricks", s, ResourceKind::Bricks);
    entry(frame, &mut xcur, [0,0,0,0], resources.wheat, b"Wheat", s, ResourceKind::Wheat);
    entry(frame, &mut xcur, [0,0,0,0], resources.flour, b"Flour", s, ResourceKind::Flour);
    entry(frame, &mut xcur, [0,0,0,0], resources.bread, b"Bread", s, ResourceKind::Bread);
    entry(frame, &mut xcur, [0,0,0,0], resources.fish, b"Fish", s, ResourceKind::Fish);
    // металлургия
    entry(frame, &mut xcur, [0,0,0,0], resources.iron_ore, b"Iron Ore", s, ResourceKind::IronOre);
    entry(frame, &mut xcur, [0,0,0,0], resources.iron_ingots, b"Iron Ingot", s, ResourceKind::IronIngot);

    // Рисуем tooltip, если есть
    if let Some((anchor_x, text)) = tooltip.take() {
        // Аккуратное позиционирование: центрируем под курсором, под верхней панелью
        let width = (text.len() as i32) * 4 * s + 8 * s;
        let tx = (anchor_x - width / 2).clamp(8 * s, fw - width - 8 * s);
        let ty = bar_h + 4 * s;
        draw_tooltip(frame, fw, fh, tx, ty, &text, s);
    }

    // Линейка прогресса дня в самом низу верхней панели (под второй строкой)
    let pb_h = 2 * s; let pb_y = bar_h - pb_h; let pb_w = (fw as f32 * day_progress_01.clamp(0.0, 1.0)) as i32;
    fill_rect(frame, fw, fh, 0, pb_y, fw, pb_h, [0, 0, 0, 120]);
    fill_rect(frame, fw, fh, 0, pb_y, pb_w, pb_h, [220, 200, 120, 200]);

    // Нижняя панель с вкладками
    let bottom_h = bottom_panel_height(s); // компактная высота под 2 строки кнопок
    let by0 = fh - bottom_h;
    fill_rect(frame, fw, fh, 0, by0, fw, bottom_h, [0, 0, 0, 160]);
    let padb = 8 * s; let btn_h = 18 * s;
    // Вкладки: Build | Economy — через row API
    let tabs = [(b"Build".as_ref(), ui_tab == UITab::Build), (b"Economy".as_ref(), ui_tab == UITab::Economy)];
    let mut row = ui_row(padb, by0 + padb, s);
    for (label, active) in tabs {
        let hovered = point_in_rect(cursor_x, cursor_y, row.cursor_x, row.y, button_w_for(label, s), row.item_h);
        ui_button_group(frame, fw, fh, &mut row, label, active, hovered, true, [220,220,220,255], ButtonStyle::Primary);
    }
    if ui_tab == UITab::Build {
        // Категории (верхняя строка нижней панели)
        // Категории — row API с переносом на 2 строки
        let cats: &[(UICategory, &[u8])] = &[
            (UICategory::Housing, b"Housing"),
            (UICategory::Storage, b"Storage"),
            (UICategory::Forestry, b"Forestry"),
            (UICategory::Mining, b"Mining"),
            (UICategory::Food, b"Food"),
            (UICategory::Logistics, b"Logistics"),
        ];
        // Рисуем двумя колонками в 2 строки, чтобы не упираться в ширину
        let row_y = [by0 + padb + btn_h + 6 * s, by0 + padb + (btn_h + 6 * s) * 2];
        let mut ridx = 0; let mut cx = padb;
        for (cat, label) in cats.iter() {
            let bw = button_w_for(label, s);
            if cx + bw > fw - padb { ridx += 1; cx = padb; }
            if ridx >= row_y.len() { break; }
            let mut r = ui_row(cx, row_y[ridx], s);
            let active = *cat == category;
            let hovered = point_in_rect(cursor_x, cursor_y, r.cursor_x, r.y, bw, r.item_h);
            ui_button_group(frame, fw, fh, &mut r, label, active, hovered, true, [220,220,220,255], ButtonStyle::Default);
            cx = r.cursor_x;
        }
        // Здания по выбранной категории (нижняя строка)
        let mut row_items = ui_row(padb, by0 + padb + (btn_h + 6 * s) * 2, s);
        let buildings_for_cat: &[BuildingKind] = match category {
            UICategory::Housing => &[BuildingKind::House],
            UICategory::Storage => &[BuildingKind::Warehouse],
            UICategory::Forestry => &[BuildingKind::Lumberjack, BuildingKind::Forester],
            UICategory::Mining => &[BuildingKind::StoneQuarry, BuildingKind::ClayPit, BuildingKind::IronMine, BuildingKind::Kiln, BuildingKind::Smelter],
            UICategory::Food => &[BuildingKind::WheatField, BuildingKind::Mill, BuildingKind::Bakery, BuildingKind::Fishery],
            UICategory::Logistics => &[],
        };
        for &bk in buildings_for_cat.iter() {
            let label = label_for_building(bk);
            let active = selected == bk;
            let bw = button_w_for(label, s);
            if row_items.cursor_x + bw > fw - padb { break; }
            let cost = building_cost(bk);
            let can_afford = resources.gold >= cost.gold && resources.wood >= cost.wood;
            let text_col = if can_afford { [220,220,220,255] } else { [140,140,140,220] };
            let hovered_btn = point_in_rect(cursor_x, cursor_y, row_items.cursor_x, row_items.y, bw, row_items.item_h);
            let (bx, by, bw, _bh) = ui_button_group(frame, fw, fh, &mut row_items, label, active, hovered_btn, can_afford, text_col, ButtonStyle::Default);
            if hovered_btn {
                let px = 2 * s; let pad = 5 * s; let gap = 6 * s; let icon = 10 * s;
                let cost_w = { let w_wood = icon + 4 + number_w(cost.wood.max(0) as u32, s); let w_gold = icon + 4 + number_w(cost.gold.max(0) as u32, s); w_wood + gap + w_gold };
                let (prod_label, prod_color, note) = production_info(bk);
                let prod_w = icon + 4 + text_w(prod_label, s);
                let note_w = note.map(|t| text_w(t, s)).unwrap_or(0);
                let tip_w = (cost_w.max(prod_w).max(note_w)) + 2 * pad;
                let line_h = icon.max(5 * px) + 2 * s;
                let lines = if note.is_some() { 3 } else { 2 };
                let tip_h = lines * line_h + 2 * s;
                let tx_mid = bx + bw / 2; let x_tip = (tx_mid - tip_w / 2).clamp(8 * s, fw - tip_w - 8 * s); let y_tip = by0 - tip_h - 4 * s;
                fill_rect(frame, fw, fh, x_tip + 2 * s, y_tip + 2 * s, tip_w, tip_h, [0,0,0,100]);
                fill_rect(frame, fw, fh, x_tip, y_tip, tip_w, tip_h, [0,0,0,160]);
                let mut cx = x_tip + pad; let cy = y_tip + s + (line_h - icon) / 2;
                fill_rect(frame, fw, fh, cx, cy, icon, icon, [110,70,30,255]); cx += icon + 4; draw_number(frame, fw, fh, cx, cy - (icon - 5*px)/2, cost.wood.max(0) as u32, [230,230,230,255], s); cx += number_w(cost.wood.max(0) as u32, s) + gap;
                fill_rect(frame, fw, fh, cx, cy, icon, icon, [220,180,60,255]); cx += icon + 4; draw_number(frame, fw, fh, cx, cy - (icon - 5*px)/2, cost.gold.max(0) as u32, [230,230,230,255], s);
                let cy2 = y_tip + line_h + s + (line_h - icon) / 2; let mut cx2 = x_tip + pad;
                fill_rect(frame, fw, fh, cx2, cy2, icon, icon, prod_color); cx2 += icon + 4; draw_text_mini(frame, fw, fh, cx2, cy2 - (icon - 5*px)/2, prod_label, [230,230,230,255], s);
                if let Some(t) = note { let cy3 = y_tip + 2*line_h + s + (line_h - 5*px)/2; draw_text_mini(frame, fw, fh, x_tip + pad, cy3, t, [200,200,200,255], s); }
            }
        }
    } else {
        // Economy panel
        let lay = layout_economy_panel(fw, fh, s);
        // Вертикальное выравнивание лейблов по центру кнопок
        let px = 2 * s; let btn_h = 18 * s; let text_h = 5 * px; let label_y = lay.y + (btn_h - text_h) / 2;
        // Tax controls (ряд под вкладками, слева направо)
        draw_text_mini(frame, fw, fh, lay.x, label_y, b"TAX", [200,200,200,255], s);
        // -/+ кнопки как в меню строительства
        let hovered_minus = point_in_rect(cursor_x, cursor_y, lay.tax_minus_x, lay.tax_minus_y, lay.tax_minus_w, lay.tax_minus_h);
        let hovered_plus  = point_in_rect(cursor_x, cursor_y, lay.tax_plus_x,  lay.tax_plus_y,  lay.tax_plus_w,  lay.tax_plus_h);
        draw_button(frame, fw, fh, lay.tax_minus_x, lay.tax_minus_y, lay.tax_minus_w, lay.tax_minus_h, false, hovered_minus, true, b"-", [230,230,230,255], s, ButtonStyle::Default);
        draw_button(frame, fw, fh, lay.tax_plus_x,  lay.tax_plus_y,  lay.tax_plus_w,  lay.tax_plus_h,  false, hovered_plus,  true, b"+", [230,230,230,255], s, ButtonStyle::Default);
        // значение налога (по центру вертикали строки)
        let taxp = (tax_rate.round().max(0.0)) as u32; draw_number(frame, fw, fh, lay.tax_plus_x + lay.tax_plus_w + 8 * s, label_y, taxp, [255,255,255,255], s);
        // Policy label и кнопки справа
        draw_text_mini(frame, fw, fh, lay.policy_bal_x - (text_w(b"FOOD", s) + 8 * s), label_y, b"FOOD", [200,200,200,255], s);
        let draw_toggle = |frame: &mut [u8], x:i32,y:i32,w:i32,h:i32, active:bool, label:&[u8], hovered: bool| {
            draw_button(frame, fw, fh, x, y, w, h, active, hovered, true, label, [220,220,220,255], s, ButtonStyle::Default);
        };
        let by = lay.y; // выравнивание по базовой линии текста
        draw_toggle(frame, lay.policy_bal_x, by, lay.policy_bal_w, lay.policy_bal_h, food_policy == FoodPolicy::Balanced, b"Balanced", point_in_rect(cursor_x, cursor_y, lay.policy_bal_x, by, lay.policy_bal_w, lay.policy_bal_h));
        draw_toggle(frame, lay.policy_bread_x, by, lay.policy_bread_w, lay.policy_bread_h, food_policy == FoodPolicy::BreadFirst, b"Bread", point_in_rect(cursor_x, cursor_y, lay.policy_bread_x, by, lay.policy_bread_w, lay.policy_bread_h));
        draw_toggle(frame, lay.policy_fish_x, by, lay.policy_fish_w, lay.policy_fish_h, food_policy == FoodPolicy::FishFirst, b"Fish", point_in_rect(cursor_x, cursor_y, lay.policy_fish_x, by, lay.policy_fish_w, lay.policy_fish_h));
        // Вторая строка: Housing слева, как TAX; справа — при наличии net-статистика
        let line2_y = lay.y + 20 * s;
        let housing_text = format!("HOUSING {} / {}", housing_used.max(0), housing_cap.max(0));
        draw_text_mini(frame, fw, fh, lay.x, line2_y, housing_text.as_bytes(), [200,220,220,255], s);
        // после Housing — дополнительная информация доход/расход, если есть
        if last_income != 0 || last_upkeep != 0 {
            let mut info_x = lay.x + text_w(housing_text.as_bytes(), s) + 12 * s;
            let net = last_income - last_upkeep;
            let info = format!("  |  INCOME +{}  UPKEEP -{}  NET {}", last_income.max(0), last_upkeep.max(0), net).into_bytes();
            draw_text_mini(frame, fw, fh, info_x, line2_y, &info, [220,220,200,255], s);
        }
    }
}

// Консоль разработчика: простое окно внизу экрана
pub fn draw_console(frame: &mut [u8], fw: i32, fh: i32, s: i32, input: &str, log: &[String]) {
    let pad = 8 * s; let px = 2 * s; let line_h = 5 * px + 4 * s;
    let lines_visible = 6usize;
    let height = pad + (lines_visible as i32) * line_h + pad + line_h; // лог + строка ввода
    let y0 = fh - height;
    // фон
    fill_rect(frame, fw, fh, 0, y0 + 2 * s, fw, height, [0, 0, 0, 120]);
    fill_rect(frame, fw, fh, 0, y0, fw, height, [0, 0, 0, 180]);
    // последние строки лога
    let start = log.len().saturating_sub(lines_visible);
    let mut y = y0 + pad;
    for line in &log[start..] {
        draw_text_mini(frame, fw, fh, pad, y, line.as_bytes(), [220,220,220,255], s);
        y += line_h;
    }
    // строка ввода с префиксом
    draw_text_mini(frame, fw, fh, pad, y, b"> ", [220,220,180,255], s);
    draw_text_mini(frame, fw, fh, pad + text_w(b"> ", s), y, input.as_bytes(), [230,230,230,255], s);
}

pub fn point_in_rect(px: i32, py: i32, x: i32, y: i32, w: i32, h: i32) -> bool { px >= x && py >= y && px < x + w && py < y + h }

fn draw_button(
    frame: &mut [u8], fw: i32, fh: i32,
    x: i32, y: i32, w: i32, h: i32,
    active: bool, hovered: bool, enabled: bool,
    label: &[u8], col: [u8;4], s: i32,
    style: ButtonStyle,
) {
    // Палитры по стилю
    let (bg_base, bg_hover, bg_active, top_hi, bot_shadow) = match style {
        ButtonStyle::Default => (
            [140,105,75,180], [160,120,85,200], [185,140,95,220], [255,255,255,70], [0,0,0,60]
        ),
        ButtonStyle::Danger => (
            [200,60,50,230], [220,80,60,240], [240,95,70,255], [255,255,255,110], [0,0,0,90]
        ),
        ButtonStyle::Primary => (
            [170,130,60,200], [190,150,80,220], [210,170,95,235], [255,255,255,90], [0,0,0,70]
        ),
    };
    let bg_disabled = [115, 95, 75, 150];
    let bg = if !enabled { bg_disabled } else if active { bg_active } else if hovered { bg_hover } else { bg_base };
    fill_rect(frame, fw, fh, x, y, w, h, bg);
    // верхний блик и нижняя тень для объёма (чуть сильнее, чтобы быть заметнее на тёмной плашке)
    let band = (2 * s).max(2);
    fill_rect(frame, fw, fh, x, y, w, band, top_hi);
    fill_rect(frame, fw, fh, x, y + h - band, w, band, bot_shadow);
    // Центрируем однобуквенные служебные ярлыки ("+"/"-") по кнопке,
    // а обычные — оставляем с левым отступом как в строительном меню
    if label == b"+" || label == b"-" {
        let px = 2 * s; let text_w = 4 * px; let text_h = 5 * px;
        let cx = x + (w - text_w) / 2; let cy = y + (h - text_h) / 2;
        draw_text_mini(frame, fw, fh, cx, cy, label, col, s);
    } else {
        // Вертикально центрируем, чтобы паддинги сверху/снизу были равны
        let px = 2 * s; let text_h = 5 * px;
        let ty = y + (h - text_h) / 2;
        draw_text_mini(frame, fw, fh, x + 6 * s, ty, label, col, s);
    }
}

fn label_for_building(bk: BuildingKind) -> &'static [u8] {
    match bk {
        BuildingKind::Lumberjack => b"Lumberjack".as_ref(),
        BuildingKind::House => b"House".as_ref(),
        BuildingKind::Warehouse => b"Warehouse".as_ref(),
        BuildingKind::Forester => b"Forester".as_ref(),
        BuildingKind::StoneQuarry => b"Quarry".as_ref(),
        BuildingKind::ClayPit => b"Clay Pit".as_ref(),
        BuildingKind::Kiln => b"Kiln".as_ref(),
        BuildingKind::IronMine => b"Iron Mine".as_ref(),
        BuildingKind::WheatField => b"Wheat Field".as_ref(),
        BuildingKind::Mill => b"Mill".as_ref(),
        BuildingKind::Bakery => b"Bakery".as_ref(),
        BuildingKind::Fishery => b"Fishery".as_ref(),
        BuildingKind::Smelter => b"Smelter".as_ref(),
    }
}

fn production_info(bk: BuildingKind) -> (&'static [u8], [u8;4], Option<&'static [u8]>) {
    match bk {
        BuildingKind::Lumberjack => (b"Produces: Wood".as_ref(), [110,70,30,255], None),
        BuildingKind::Forester => (b"Plants trees".as_ref(), [90,140,90,255], None),
        BuildingKind::StoneQuarry => (b"Produces: Stone".as_ref(), [120,120,120,255], None),
        BuildingKind::ClayPit => (b"Produces: Clay".as_ref(), [150,90,70,255], None),
        BuildingKind::Kiln => (b"Produces: Bricks".as_ref(), [180,120,90,255], Some(b"Uses Clay + Wood".as_ref())),
        BuildingKind::WheatField => (b"Produces: Wheat".as_ref(), [200,180,80,255], None),
        BuildingKind::Mill => (b"Produces: Flour".as_ref(), [210,210,180,255], Some(b"Uses Wheat".as_ref())),
        BuildingKind::Bakery => (b"Produces: Bread".as_ref(), [200,160,120,255], Some(b"Uses Flour + Wood".as_ref())),
        BuildingKind::Fishery => (b"Produces: Fish".as_ref(), [100,140,200,255], Some(b"Near water".as_ref())),
        BuildingKind::Warehouse => (b"Stores resources".as_ref(), [150,120,80,255], None),
        BuildingKind::House => (b"Generates: Gold".as_ref(), [220,180,60,255], Some(b"Consumes Bread/Fish".as_ref())),
        BuildingKind::IronMine => (b"Produces: Iron Ore".as_ref(), [90,90,110,255], None),
        BuildingKind::Smelter => (b"Produces: Iron Ingot".as_ref(), [190, 190, 210,255], Some(b"Uses Iron Ore + Wood".as_ref())),
    }
}

// удалено: building_cost_ui — используем общий `types::building_cost`

pub fn draw_tooltip(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, text: &[u8], s: i32) {
    // ширина глифа = 4*px, где px = 2*s → 8*s на символ
    let px = 2 * s;
    let char_w = 4 * px;
    let width = (text.len() as i32) * char_w + 10 * s; // паддинги по 5*s слева/справа
    let text_h = 5 * px; // высота глифа 3x5 в пикселях
    let height = text_h + 6 * s; // по 3*s сверху/снизу
    // подложка + лёгкая тень сверху (стиль панелей: альфа ≈160)
    fill_rect(frame, fw, fh, x + 2 * s, y + 2 * s, width, height, [0, 0, 0, 100]);
    fill_rect(frame, fw, fh, x, y, width, height, [0, 0, 0, 160]);
    draw_text_mini(frame, fw, fh, x + 5 * s, y + 3 * s, text, [230,230,230,255], s);
}

pub fn button_w_for(label: &[u8], s: i32) -> i32 {
    let px = 2 * s; // ширина «пикселя» глифа
    let text_w = (label.len() as i32) * 4 * px; // 3x5 глиф с шагом 4
    text_w + 12 * s // паддинги
}

fn text_w(label: &[u8], s: i32) -> i32 { (label.len() as i32) * 4 * (2 * s) }
fn number_w(mut n: u32, s: i32) -> i32 {
    let mut len = 0; if n == 0 { len = 1; }
    while n > 0 { len += 1; n /= 10; }
    len * 4 * (2 * s)
}

fn draw_two_digits(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, val: u32, col: [u8;4], s: i32) -> i32 {
    // Рисует двузначное число с ведущим нулём, возвращает ширину
    let px = 2 * s;
    let tens = (val / 10) % 10; let ones = val % 10;
    let mut ix = x;
    draw_glyph_3x5(frame, fw, fh, ix, y, b'0' + tens as u8, col, px); ix += 4 * px;
    draw_glyph_3x5(frame, fw, fh, ix, y, b'0' + ones as u8, col, px); ix += 4 * px;
    ix - x
}

fn draw_text_mini(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, text: &[u8], color: [u8;4], s: i32) {
    // масштаб пикселя глифа: 2*s → глиф 6x10
    let px = 2 * s;
    let mut cx = x; let cy = y;
    for &ch in text {
        if ch == b' ' { cx += 4 * px; continue; }
        if ch == b'[' || ch == b']' || ch == b'/' || ch == b'\\' || ch == b'+' || ch == b'-' || (ch >= b'0' && ch <= b'9') || (ch >= b'A' && ch <= b'Z') || (ch >= b'a' && ch <= b'z') {
            draw_glyph_3x5(frame, fw, fh, cx, cy, ch, color, px);
            cx += 4 * px;
        } else { cx += 4 * px; }
    }
}

fn draw_glyph_3x5(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, ch: u8, color: [u8;4], px: i32) {
    let pattern: [u8; 15] = match ch.to_ascii_uppercase() {
        b'0' => [1,1,1, 1,0,1, 1,0,1, 1,0,1, 1,1,1],
        b'1' => [0,1,0, 1,1,0, 0,1,0, 0,1,0, 1,1,1],
        b'2' => [1,1,1, 0,0,1, 1,1,1, 1,0,0, 1,1,1],
        b'3' => [1,1,1, 0,0,1, 0,1,1, 0,0,1, 1,1,1],
        b'4' => [1,0,1, 1,0,1, 1,1,1, 0,0,1, 0,0,1],
        b'5' => [1,1,1, 1,0,0, 1,1,1, 0,0,1, 1,1,1],
        b'6' => [1,1,1, 1,0,0, 1,1,1, 1,0,1, 1,1,1],
        b'7' => [1,1,1, 0,0,1, 0,1,0, 0,1,0, 0,1,0],
        b'8' => [1,1,1, 1,0,1, 1,1,1, 1,0,1, 1,1,1],
        b'9' => [1,1,1, 1,0,1, 1,1,1, 0,0,1, 1,1,1],
        b'A' => [0,1,0, 1,0,1, 1,1,1, 1,0,1, 1,0,1],
        b'B' => [1,1,0, 1,0,1, 1,1,0, 1,0,1, 1,1,0],
        b'C' => [0,1,1, 1,0,0, 1,0,0, 1,0,0, 0,1,1],
        b'D' => [1,1,0, 1,0,1, 1,0,1, 1,0,1, 1,1,0],
        b'E' => [1,1,1, 1,0,0, 1,1,0, 1,0,0, 1,1,1],
        b'F' => [1,1,1, 1,0,0, 1,1,0, 1,0,0, 1,0,0],
        b'G' => [0,1,1, 1,0,0, 1,0,1, 1,0,1, 0,1,1],
        b'H' => [1,0,1, 1,0,1, 1,1,1, 1,0,1, 1,0,1],
        b'I' => [1,1,1, 0,1,0, 0,1,0, 0,1,0, 1,1,1],
        b'J' => [0,1,1, 0,0,1, 0,0,1, 1,0,1, 0,1,0],
        b'K' => [1,0,1, 1,0,0, 1,1,0, 1,0,0, 1,0,1],
        b'L' => [1,0,0, 1,0,0, 1,0,0, 1,0,0, 1,1,1],
        b'M' => [1,0,1, 1,1,1, 1,0,1, 1,0,1, 1,0,1],
        b'N' => [1,0,1, 1,1,1, 1,1,1, 1,0,1, 1,0,1],
        b'O' => [0,1,0, 1,0,1, 1,0,1, 1,0,1, 0,1,0],
        b'P' => [1,1,0, 1,0,1, 1,1,0, 1,0,0, 1,0,0],
        b'Q' => [0,1,0, 1,0,1, 1,0,1, 1,1,1, 0,1,1],
        b'R' => [1,1,0, 1,0,1, 1,1,0, 1,0,1, 1,0,1],
        b'S' => [0,1,1, 1,0,0, 0,1,0, 0,0,1, 1,1,0],
        b'T' => [1,1,1, 0,1,0, 0,1,0, 0,1,0, 0,1,0],
        b'U' => [1,0,1, 1,0,1, 1,0,1, 1,0,1, 1,1,1],
        b'V' => [1,0,1, 1,0,1, 1,0,1, 1,0,1, 0,1,0],
        b'W' => [1,0,1, 1,0,1, 1,0,1, 1,1,1, 1,0,1],
        b'Y' => [1,0,1, 1,0,1, 0,1,0, 0,1,0, 0,1,0],
        b'Z' => [1,1,1, 0,0,1, 0,1,0, 1,0,0, 1,1,1],
        b'X' => [1,0,1, 1,0,1, 0,1,0, 1,0,1, 1,0,1],
        // Симметричные, ровные глифы для + и -
        b'+' => [0,0,0, 0,1,0, 1,1,1, 0,1,0, 0,0,0],
        b'-' => [0,0,0, 0,0,0, 1,1,1, 0,0,0, 0,0,0],
        b'[' => [1,1,0, 1,0,0, 1,0,0, 1,0,0, 1,1,0],
        b']' => [0,1,1, 0,0,1, 0,0,1, 0,0,1, 0,1,1],
        b'/' => [0,0,1, 0,1,0, 0,1,0, 1,0,0, 1,0,0],
        b'\\' => [1,0,0, 1,0,0, 0,1,0, 0,1,0, 0,0,1],
        _ => [1,1,1, 1,1,1, 1,1,1, 1,1,1, 1,1,1],
    };
    for row in 0..5 { for cx_i in 0..3 { if pattern[row*3 + cx_i] == 1 { fill_rect(frame, fw, fh, x + cx_i as i32 * px, y + row as i32 * px, px, px, color); } } }
}

fn draw_number(frame: &mut [u8], fw: i32, fh: i32, mut x: i32, y: i32, mut n: u32, col: [u8;4], s: i32) {
    let mut digits: [u8; 12] = [0; 12]; let mut len = 0; if n == 0 { digits[0] = b'0'; len = 1; }
    while n > 0 && len < digits.len() { let d = (n % 10) as u8; n /= 10; digits[len] = b'0' + d; len += 1; }
    let px = 2 * s;
    for i in (0..len).rev() { draw_glyph_3x5(frame, fw, fh, x, y, digits[i], col, px); x += 4 * px; }
}

fn fill_rect(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
    let x0 = x.max(0); let y0 = y.max(0); let x1 = (x + w).min(fw); let y1 = (y + h).min(fh);
    if x0 >= x1 || y0 >= y1 { return; }
    for yy in y0..y1 { let row = (yy as usize) * (fw as usize) * 4; for xx in x0..x1 {
        let idx = row + (xx as usize) * 4; let a = color[3] as u32; let na = 255 - a; let dr = frame[idx] as u32; let dg = frame[idx+1] as u32; let db = frame[idx+2] as u32;
        frame[idx] = ((a * color[0] as u32 + na * dr) / 255) as u8; frame[idx+1] = ((a * color[1] as u32 + na * dg) / 255) as u8; frame[idx+2] = ((a * color[2] as u32 + na * db) / 255) as u8; frame[idx+3] = 255;
    }}
}

// Виджет мини-карты: вычисляет позицию/масштаб и отдаёт отрисовку в render::map::draw_minimap
pub fn draw_minimap_widget(
    frame: &mut [u8], fw: i32, fh: i32, s: i32,
    world: &mut World,
    buildings: &Vec<Building>,
    cam_px: Vec2,
    atlas_half_w: i32, atlas_half_h: i32,
    vis_min_tx: i32, vis_min_ty: i32, vis_max_tx: i32, vis_max_ty: i32,
    cell_px: i32,
    cursor_x: i32, cursor_y: i32,
) {
    let pad = ui_pad(s);
    // Бейзлайн: 96x64 клеток по 2*s (внешний размер фиксированный)
    let base_cell = 2 * s; let base_w_tiles = 96; let base_h_tiles = 64;
    let widget_w = base_w_tiles * base_cell; let widget_h = base_h_tiles * base_cell;
    // Создаём группу-строку в правом нижнем углу и резервируем слот под миникарту — в духе API
    let mut grp = ui_row(fw - pad - widget_w, fh - bottom_panel_height(s) - pad - widget_h, s);
    let (x, y, w_slot, h_slot) = ui_slot_wh(&mut grp, widget_w, widget_h);
    // Подложка виджета в стиле панелей (тень + фон)
    fill_rect(frame, fw, fh, x + 2 * s, y + 2 * s, w_slot, h_slot, [0, 0, 0, 120]);
    fill_rect(frame, fw, fh, x, y, w_slot, h_slot, [0, 0, 0, 180]);
    // Детализация: используем переданный cell_px, охват — по размеру слота
    let cell = cell_px.max(1);
    let mini_w_tiles = (w_slot / cell).max(8);
    let mini_h_tiles = (h_slot / cell).max(6);
    // центр вокруг текущего смещения камеры (переход px->тайлы приблизительно)
    let cam_cx = ((cam_px.x / (atlas_half_w as f32)).round() as i32) / 2;
    let cam_cy = ((cam_px.y / (atlas_half_h as f32)).round() as i32) / 2;
    let half_w = mini_w_tiles / 2; let half_h = mini_h_tiles / 2;
    let mm_min_tx = cam_cx - half_w; let mm_min_ty = cam_cy - half_h;
    let mm_max_tx = cam_cx + half_w - 1; let mm_max_ty = cam_cy + half_h - 1;
    // фон и миникарта
    crate::render::map::draw_minimap(
        frame, fw, fh, world, buildings,
        mm_min_tx, mm_min_ty, mm_max_tx, mm_max_ty,
        x, y, cell,
        vis_min_tx, vis_min_ty, vis_max_tx, vis_max_ty,
    );

    // Кнопки зума у левого края миникарты, вертикально
    let btn_h = ui_item_h(s); let btn_w = button_w_for(b"+", s);
    let gap = ui_gap(s);
    let plus_x = x - (btn_w + gap); let plus_y = y; // сверху слева
    let minus_x = plus_x; let minus_y = plus_y + btn_h + gap; // под плюсом
    let hovered_plus = point_in_rect(cursor_x, cursor_y, plus_x, plus_y, btn_w, btn_h);
    let hovered_minus = point_in_rect(cursor_x, cursor_y, minus_x, minus_y, btn_w, btn_h);
    draw_button(frame, fw, fh, minus_x, minus_y, btn_w, btn_h, false, hovered_minus, true, b"-", [230,230,230,255], s, ButtonStyle::Default);
    draw_button(frame, fw, fh, plus_x, plus_y, btn_w, btn_h, false, hovered_plus, true, b"+", [230,230,230,255], s, ButtonStyle::Default);
}


