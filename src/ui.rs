use crate::types::{Resources, BuildingKind, building_cost, ResourceKind, FoodPolicy};
use crate::palette::resource_color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UICategory { Housing, Storage, Forestry, Mining, Food, Logistics }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UITab { Build, Economy }

pub fn ui_scale(fh: i32, k: f32) -> i32 { (((fh as f32) / 720.0) * k).clamp(1.0, 5.0) as i32 }
// удалено: ui_bar_height (не используется)
pub fn bottom_panel_height(s: i32) -> i32 { let padb = 8 * s; let btn_h = 18 * s; // Tabs + Categories + Items (межстрочный отступ один)
    padb * 2 + btn_h * 3 + 6 * s * 2 }
pub fn top_panel_height(s: i32) -> i32 {
    let pad = 8 * s; let icon = 10 * s; let px = 2 * s; let glyph_h = 5 * px;
    // Две строки контента (иконки/цифры) + отступ между ними
    pad * 2 + (icon.max(glyph_h)) * 2 + 6 * s
}

#[derive(Clone, Copy, Debug)]
pub struct BuildingPanelLayout { pub x: i32, pub y: i32, pub w: i32, pub h: i32, pub minus_x: i32, pub minus_y: i32, pub minus_w: i32, pub minus_h: i32, pub plus_x: i32, pub plus_y: i32, pub plus_w: i32, pub plus_h: i32 }

pub fn layout_building_panel(fw: i32, fh: i32, s: i32) -> BuildingPanelLayout {
    let padb = 8 * s;
    let bottom_h = bottom_panel_height(s);
    // Компактная плашка слева, не на всю ширину
    let w = (fw as f32 * 0.33) as i32; // треть экрана
    let panel_h = 64 * s; // достаточно для 3 строк
    let x = padb;
    let y = fh - bottom_h - panel_h - 6 * s;
    // Кнопки +/-
    let minus_w = 14 * s; let minus_h = 14 * s; let plus_w = 14 * s; let plus_h = 14 * s;
    let minus_x = x + w - (plus_w + minus_w + 12 * s);
    // выравниваем по строке Workers (см. draw_building_panel: y + 6*s + line_h)
    let px = 2 * s; let line_h = (5 * px).max(10 * s) + 4 * s; let workers_y = y + 6 * s + line_h;
    let minus_y = workers_y - 2 * s;
    let plus_x = x + w - (plus_w + 4 * s);
    let plus_y = workers_y - 2 * s;
    BuildingPanelLayout { x, y, w, h: panel_h, minus_x, minus_y, minus_w, minus_h, plus_x, plus_y, plus_w, plus_h }
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
    let cx = layout.x + pad;
    // название
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
    let mut yline = layout.y + 6 * s;
    draw_text_mini(frame, fw, fh, cx, yline, title, [230,230,230,255], s);
    // шаг строк
    let line_h = (5 * px).max(icon) + 4 * s; // высота глифа + отступ
    // workers
    yline += line_h;
    let workers_text = b"Workers";
    draw_text_mini(frame, fw, fh, cx, yline, workers_text, [200,200,200,255], s);
    let mut valx = cx + text_w(workers_text, s) + 6 * s;
    draw_number(frame, fw, fh, valx, yline, workers_current.max(0) as u32, [255,255,255,255], s);
    valx += number_w(workers_current.max(0) as u32, s);
    draw_glyph_3x5(frame, fw, fh, valx, yline, b'/', [180,180,180,255], px);
    valx += 4 * px;
    draw_number(frame, fw, fh, valx, yline, workers_target.max(0) as u32, [220,220,220,255], s);
    // +/-
    fill_rect(frame, fw, fh, layout.minus_x, layout.minus_y, layout.minus_w, layout.minus_h, [120, 100, 80, 220]);
    fill_rect(frame, fw, fh, layout.plus_x, layout.plus_y, layout.plus_w, layout.plus_h, [120, 100, 80, 220]);
    // минус знак
    fill_rect(frame, fw, fh, layout.minus_x + 3 * s, layout.minus_y + layout.minus_h/2 - s, layout.minus_w - 6 * s, 2 * s, [230,230,230,255]);
    // плюс знак
    let cxp = layout.plus_x + layout.plus_w/2; let cyp = layout.plus_y + layout.plus_h/2;
    fill_rect(frame, fw, fh, cxp - (layout.plus_w/2 - 3 * s), cyp - s, layout.plus_w - 6 * s, 2 * s, [230,230,230,255]);
    fill_rect(frame, fw, fh, cxp - s, cyp - (layout.plus_h/2 - 3 * s), 2 * s, layout.plus_h - 6 * s, [230,230,230,255]);
    // production/consumption c иконками ресурсов
    yline += line_h;
    let (prod_col_opt, cons_cols) = resource_colors_for_building(kind);
    let mut prod_x = cx;
    if let Some(col) = prod_col_opt {
        fill_rect(frame, fw, fh, prod_x, yline, icon, icon, col);
        prod_x += icon + 6 * s;
    }
    draw_text_mini(frame, fw, fh, prod_x, yline, prod_label, [220,220,220,255], s);
    if let Some(c) = cons_label {
        let mut con_x = cx + text_w(prod_label, s) + 12 * s + (if prod_col_opt.is_some() { icon + 6 * s } else { 0 });
        for (idx, col) in cons_cols.iter().enumerate() {
            if idx > 0 { con_x += 2 * s; }
            fill_rect(frame, fw, fh, con_x, yline, icon, icon, *col);
            con_x += icon + 4 * s;
        }
        draw_text_mini(frame, fw, fh, con_x, yline, c, [200,200,200,255], s);
    }
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
) {
    let s = ui_scale(fh, base_scale_k);
    let bar_h = top_panel_height(s);
    fill_rect(frame, fw, fh, 0, 0, fw, bar_h, [0, 0, 0, 160]);

    let pad = 8 * s;
    let icon_size = 10 * s;
    let mut x = pad;
    // Слева — население и золото, с резервом под 4 цифры
    let px = 2 * s; let reserve_digits = 4; let reserved_w = reserve_digits * 4 * px; let gap = 6 * s;
    // население
    fill_rect(frame, fw, fh, x, pad, icon_size, icon_size, [180, 60, 60, 255]);
    let num_x_pop = x + icon_size + 4;
    draw_number(frame, fw, fh, num_x_pop, pad, population.max(0) as u32, [255,255,255,255], s);
    let mut tooltip: Option<(i32, Vec<u8>)> = None;
    if point_in_rect(cursor_x, cursor_y, x, pad, icon_size, icon_size) {
        tooltip = Some((x + icon_size / 2, b"Population".to_vec()));
    }
    x = num_x_pop + reserved_w + gap;
    // золото
    fill_rect(frame, fw, fh, x, pad, icon_size, icon_size, [220, 180, 60, 255]);
    let num_x_gold = x + icon_size + 4;
    draw_number(frame, fw, fh, num_x_gold, pad, resources.gold.max(0) as u32, [255,255,255,255], s);
    if point_in_rect(cursor_x, cursor_y, x, pad, icon_size, icon_size) {
        // Простой тултип: золото. В будущем можно добавить доход/апкип за последний день
        tooltip = Some((x + icon_size / 2, b"Gold".to_vec()));
    }
    x = num_x_gold + reserved_w + gap;
    // среднее счастье
    fill_rect(frame, fw, fh, x, pad, icon_size, icon_size, [220, 120, 160, 255]);
    let num_x_hap = x + icon_size + 4;
    let hap = avg_happiness.round().clamp(0.0, 100.0) as u32;
    draw_number(frame, fw, fh, num_x_hap, pad, hap, [255,255,255,255], s);
    if point_in_rect(cursor_x, cursor_y, x, pad, icon_size, icon_size) { tooltip = Some((x + icon_size / 2, b"Happiness".to_vec())); }
    x = num_x_hap + reserved_w + gap;
    // налоговая ставка (в процентах)
    fill_rect(frame, fw, fh, x, pad, icon_size, icon_size, [200, 180, 90, 255]);
    let num_x_tax = x + icon_size + 4;
    let taxp = (tax_rate * 100.0).round().clamp(0.0, 100.0) as u32;
    draw_number(frame, fw, fh, num_x_tax, pad, taxp, [255,255,255,255], s);
    if point_in_rect(cursor_x, cursor_y, x, pad, icon_size, icon_size) { tooltip = Some((x + icon_size / 2, b"Tax %  [ [ ] to change ]".to_vec())); }
    x = num_x_tax + reserved_w + gap;

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

    // Блок статистики жителей (idle/working/sleeping/haul/fetch)
    let mut sx = x;
    let sy = pad;
    let mut entry_stat = |frame: &mut [u8], x: &mut i32, label_color: [u8;4], val: i32, label: &[u8], s: i32| {
        let px = 2 * s; let reserve_digits = 3; let reserved_w = reserve_digits * 4 * px; let gap = 6 * s;
        let need_w = icon_size + 4 + reserved_w + gap;
        if *x + need_w > right_x0 { return; }
        fill_rect(frame, fw, fh, *x, sy, icon_size, icon_size, label_color);
        let num_x = *x + icon_size + 4;
        draw_number(frame, fw, fh, num_x, sy, val.max(0) as u32, [255,255,255,255], s);
        if point_in_rect(cursor_x, cursor_y, *x, sy, icon_size, icon_size) {
            tooltip = Some((*x + icon_size / 2, label.to_vec()));
        }
        *x = num_x + reserved_w + gap;
    };
    entry_stat(frame, &mut sx, [70, 160, 70, 255], citizens_idle, b"Idle", s);
    entry_stat(frame, &mut sx, [70, 110, 200, 255], citizens_working, b"Working", s);
    entry_stat(frame, &mut sx, [120, 120, 120, 255], citizens_sleeping, b"Sleeping", s);
    entry_stat(frame, &mut sx, [210, 150, 70, 255], citizens_hauling, b"Hauling", s);
    entry_stat(frame, &mut sx, [80, 200, 200, 255], citizens_fetching, b"Fetching", s);
    let _ = sx; // x больше не используется, не присваиваем

    // верх: больше не рисуем кнопки строительства — они в нижней панели

    // Инфо справа: одна строка, правое выравнивание: "TIME HH:MM  FPS <n>  SPEED <m>" или "TIME HH:MM  FPS <n>  PAUSE"
    let px = 2 * s; let gap_info = 2 * px; let y_info = 8 * s; let pad_right = 8 * s;
    // Время суток из day_progress_01 (0..1)
    let mut minutes = (day_progress_01.clamp(0.0, 1.0) * 1440.0).round() as i32 % 1440;
    let hours = (minutes / 60) as u32; minutes = minutes % 60; let minutes_u = minutes as u32;
    let fps_n = fps.round() as u32;
    let fps_w = text_w(b"FPS", s) + gap_info + number_w(fps_n, s);
    let speed_w = if paused { text_w(b"PAUSE", s) } else { let sp = (speed * 10.0).round() as u32; text_w(b"SPEED", s) + gap_info + number_w(sp, s) };
    let time_w = text_w(b"TIME", s) + gap_info + (5 * 4 * px); // HH:MM — 5 глифов
    let total_w = time_w + gap_info + fps_w + gap_info + speed_w;
    let mut ix = fw - pad_right - total_w;
    // TIME
    draw_text_mini(frame, fw, fh, ix, y_info, b"TIME", [200,200,200,255], s);
    ix += text_w(b"TIME", s) + gap_info;
    // HH:MM
    ix += draw_two_digits(frame, fw, fh, ix, y_info, hours, [255,255,255,255], s);
    draw_glyph_3x5(frame, fw, fh, ix, y_info, b':', [255,255,255,255], px);
    ix += 4 * px;
    ix += draw_two_digits(frame, fw, fh, ix, y_info, minutes_u, [255,255,255,255], s);
    ix += gap_info;
    // FPS
    draw_text_mini(frame, fw, fh, ix, y_info, b"FPS", [200,200,200,255], s);
    ix += text_w(b"FPS", s) + gap_info;
    draw_number(frame, fw, fh, ix, y_info, fps_n, [255,255,255,255], s);
    ix += number_w(fps_n, s) + gap_info;
    // SPEED/PAUSE
    if paused {
        draw_text_mini(frame, fw, fh, ix, y_info, b"PAUSE", [200,200,200,255], s);
    } else {
        let sp = (speed * 10.0).round() as u32;
        draw_text_mini(frame, fw, fh, ix, y_info, b"SPEED", [200,200,200,255], s);
        ix += text_w(b"SPEED", s) + gap_info;
        draw_number(frame, fw, fh, ix, y_info, sp, [255,255,255,255], s);
    }

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
    // Вкладки: Build | Economy
    let tabs = [(b"Build".as_ref(), ui_tab == UITab::Build), (b"Economy".as_ref(), ui_tab == UITab::Economy)];
    let mut tx = padb;
    for (label, active) in tabs { let bw = button_w_for(label, s); draw_button(frame, fw, fh, tx, by0 + padb, bw, btn_h, active, point_in_rect(cursor_x, cursor_y, tx, by0 + padb, bw, btn_h), true, label, [220,220,220,255], s); tx += bw + 6 * s; }
    if ui_tab == UITab::Build {
        // Категории (верхняя строка нижней панели)
        let mut cx = padb;
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
        let mut row = 0; cx = padb;
        for (idx, (cat, label)) in cats.iter().enumerate() {
            let active = *cat == category; let bw = button_w_for(label, s);
            if cx + bw > fw - padb { row += 1; cx = padb; }
            if row > 0 && row as usize >= row_y.len() { break; }
            let y = row_y[row as usize];
            let hovered = point_in_rect(cursor_x, cursor_y, cx, y, bw, btn_h);
            draw_button(frame, fw, fh, cx, y, bw, btn_h, active, hovered, true, label, [200,200,200,255], s);
            cx += bw + 6 * s;
        }
        // Здания по выбранной категории (нижняя строка)
        let mut bx = padb;
        let by = by0 + padb + (btn_h + 6 * s) * 2;
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
            if bx + bw > fw - padb { break; }
            let cost = building_cost(bk);
            let can_afford = resources.gold >= cost.gold && resources.wood >= cost.wood;
            let text_col = if can_afford { [220,220,220,255] } else { [140,140,140,220] };
            let hovered_btn = point_in_rect(cursor_x, cursor_y, bx, by, bw, btn_h);
            draw_button(frame, fw, fh, bx, by, bw, btn_h, active, hovered_btn, can_afford, label, text_col, s);
            if point_in_rect(cursor_x, cursor_y, bx, by, bw, btn_h) {
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
            bx += bw + 6 * s;
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
        draw_button(frame, fw, fh, lay.tax_minus_x, lay.tax_minus_y, lay.tax_minus_w, lay.tax_minus_h, false, hovered_minus, true, b"-", [230,230,230,255], s);
        draw_button(frame, fw, fh, lay.tax_plus_x,  lay.tax_plus_y,  lay.tax_plus_w,  lay.tax_plus_h,  false, hovered_plus,  true, b"+", [230,230,230,255], s);
        // значение налога (по центру вертикали строки)
        let taxp = (tax_rate.round().max(0.0)) as u32; draw_number(frame, fw, fh, lay.tax_plus_x + lay.tax_plus_w + 8 * s, label_y, taxp, [255,255,255,255], s);
        // Policy label и кнопки справа
        draw_text_mini(frame, fw, fh, lay.policy_bal_x - (text_w(b"FOOD", s) + 8 * s), label_y, b"FOOD", [200,200,200,255], s);
        let draw_toggle = |frame: &mut [u8], x:i32,y:i32,w:i32,h:i32, active:bool, label:&[u8], hovered: bool| {
            draw_button(frame, fw, fh, x, y, w, h, active, hovered, true, label, [220,220,220,255], s);
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

pub fn point_in_rect(px: i32, py: i32, x: i32, y: i32, w: i32, h: i32) -> bool { px >= x && py >= y && px < x + w && py < y + h }

fn draw_button(
    frame: &mut [u8], fw: i32, fh: i32,
    x: i32, y: i32, w: i32, h: i32,
    active: bool, hovered: bool, enabled: bool,
    label: &[u8], col: [u8;4], s: i32,
) {
    // Деревянная палитра (светлее)
    let bg_base = [140, 105, 75, 180];
    let bg_hover = [160, 120, 85, 200];
    let bg_active = [185, 140, 95, 220];
    let bg_disabled = [115, 95, 75, 150];
    let bg = if !enabled { bg_disabled } else if active { bg_active } else if hovered { bg_hover } else { bg_base };
    fill_rect(frame, fw, fh, x, y, w, h, bg);
    // верхний блик и нижняя тень для объёма (чуть сильнее, чтобы быть заметнее на тёмной плашке)
    let band = (2 * s).max(2);
    fill_rect(frame, fw, fh, x, y, w, band, [255, 255, 255, 70]);
    fill_rect(frame, fw, fh, x, y + h - band, w, band, [0, 0, 0, 60]);
    // Центрируем однобуквенные служебные ярлыки ("+"/"-") по кнопке,
    // а обычные — оставляем с левым отступом как в строительном меню
    if label == b"+" || label == b"-" {
        let px = 2 * s; let text_w = 4 * px; let text_h = 5 * px;
        let cx = x + (w - text_w) / 2; let cy = y + (h - text_h) / 2;
        draw_text_mini(frame, fw, fh, cx, cy, label, col, s);
    } else {
        draw_text_mini(frame, fw, fh, x + 6 * s, y + 4 * s, label, col, s);
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


