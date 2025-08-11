use crate::types::{Resources, BuildingKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UICategory { Housing, Storage, Forestry, Mining, Food, Logistics }

pub fn ui_scale(fh: i32, k: f32) -> i32 { (((fh as f32) / 720.0) * k).clamp(1.0, 5.0) as i32 }
pub fn ui_bar_height(fh: i32, s: i32) -> i32 { ((fh as f32 * 0.06).max(24.0) as i32) * s }
pub fn bottom_panel_height(s: i32) -> i32 { let padb = 8 * s; let btn_h = 18 * s; padb * 2 + btn_h * 2 + 6 * s }
pub fn top_panel_height(s: i32) -> i32 {
    let pad = 8 * s; let icon = 10 * s; let px = 2 * s; let glyph_h = 5 * px;
    // Две строки контента (иконки/цифры) + отступ между ними
    pad * 2 + (icon.max(glyph_h)) * 2 + 6 * s
}

pub fn draw_ui(
    frame: &mut [u8], fw: i32, fh: i32,
    resources: &Resources, total_wood: i32, population: i32, selected: BuildingKind,
    fps: f32, speed: f32, paused: bool, base_scale_k: f32, category: UICategory, day_progress_01: f32,
    citizens_idle: i32, citizens_working: i32, citizens_sleeping: i32, citizens_hauling: i32, citizens_fetching: i32,
    cursor_x: i32, cursor_y: i32,
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
        tooltip = Some((x + icon_size / 2, b"Gold".to_vec()));
    }
    x = num_x_gold + reserved_w + gap;

    // Рассчитаем занятый справа блок (TIME/FPS/SPEED), чтобы не наезжать на него слева
    let px = 2 * s; let gap_info = 2 * px; let y_info = 8 * s; let pad_right = 8 * s;
    let mut minutes_tmp = (day_progress_01.clamp(0.0, 1.0) * 1440.0).round() as i32 % 1440;
    let hours_tmp = (minutes_tmp / 60) as u32; minutes_tmp = minutes_tmp % 60; let _minutes_u_tmp = minutes_tmp as u32;
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
    x = sx + 6 * s;

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
    let mut rx = pad;
    let ry = pad + icon_size + 6 * s;
    let mut entry = |frame: &mut [u8], x: &mut i32, label_color: [u8;4], val: i32, label: &[u8], s: i32| {
        let px = 2 * s; let reserve_digits = 4; let reserved_w = reserve_digits * 4 * px; let gap = 6 * s;
        let need_w = icon_size + 4 + reserved_w + gap;
        if *x + need_w > right_x0 { return; }
        fill_rect(frame, fw, fh, *x, ry, icon_size, icon_size, label_color);
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
    entry(frame, &mut xcur, [110,70,30,255], total_wood, b"Wood", s);
    entry(frame, &mut xcur, [120,120,120,255], resources.stone, b"Stone", s);
    entry(frame, &mut xcur, [150,90,70,255], resources.clay, b"Clay", s);
    entry(frame, &mut xcur, [180,120,90,255], resources.bricks, b"Bricks", s);
    entry(frame, &mut xcur, [200,180,80,255], resources.wheat, b"Wheat", s);
    entry(frame, &mut xcur, [210,210,180,255], resources.flour, b"Flour", s);
    entry(frame, &mut xcur, [200,160,120,255], resources.bread, b"Bread", s);
    entry(frame, &mut xcur, [100,140,200,255], resources.fish, b"Fish", s);
    // металлургия
    entry(frame, &mut xcur, [90,90,110,255], resources.iron_ore, b"Iron Ore", s);
    entry(frame, &mut xcur, [190,190,210,255], resources.iron_ingots, b"Iron Ingot", s);

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

    // Нижняя панель с категориями/зданиями
    let bottom_h = bottom_panel_height(s); // компактная высота под 2 строки кнопок
    let by0 = fh - bottom_h;
    fill_rect(frame, fw, fh, 0, by0, fw, bottom_h, [0, 0, 0, 160]);
    let padb = 8 * s; let btn_h = 18 * s;
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
    for (cat, label) in cats.iter() {
        let active = *cat == category;
        let bw = button_w_for(label, s);
        let hovered = point_in_rect(cursor_x, cursor_y, cx, by0 + padb, bw, btn_h);
        draw_button(frame, fw, fh, cx, by0 + padb, bw, btn_h, active, hovered, true, label, [200,200,200,255], s);
        cx += bw + 6 * s;
    }
    // Здания по выбранной категории (нижняя строка)
    let mut bx = padb;
    let by = by0 + padb + btn_h + 6 * s;
    let buildings_for_cat: &[BuildingKind] = match category {
        UICategory::Housing => &[BuildingKind::House],
        UICategory::Storage => &[BuildingKind::Warehouse],
        UICategory::Forestry => &[BuildingKind::Lumberjack, BuildingKind::Forester],
        UICategory::Mining => &[BuildingKind::StoneQuarry, BuildingKind::ClayPit, BuildingKind::IronMine, BuildingKind::Kiln, BuildingKind::Smelter],
        UICategory::Food => &[BuildingKind::WheatField, BuildingKind::Mill, BuildingKind::Bakery, BuildingKind::Fishery],
        UICategory::Logistics => &[],
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
            BuildingKind::IronMine => b"Iron Mine".as_ref(),
            BuildingKind::WheatField => b"Wheat Field".as_ref(),
            BuildingKind::Mill => b"Mill".as_ref(),
            BuildingKind::Bakery => b"Bakery".as_ref(),
            BuildingKind::Fishery => b"Fishery".as_ref(),
            BuildingKind::Smelter => b"Smelter".as_ref(),
        };
        let active = selected == bk;
        let bw = button_w_for(label, s);
        if bx + bw > fw - padb { break; }
        // Отрисовка с учётом доступности по стоимости
        let cost = building_cost_ui(bk);
        // Доступность считаем по сумме: ресурсы на руках + на складах (в верхней панели уже передан суммарный Resources)
        let can_afford = resources.gold >= cost.gold && resources.wood >= cost.wood;
        let text_col = if can_afford { [220,220,220,255] } else { [140,140,140,220] };
        let hovered_btn = point_in_rect(cursor_x, cursor_y, bx, by, bw, btn_h);
        draw_button(frame, fw, fh, bx, by, bw, btn_h, active, hovered_btn, can_afford, label, text_col, s);
        // Тултип по наведению: стоимость и требования
        if point_in_rect(cursor_x, cursor_y, bx, by, bw, btn_h) {
            // Построим тултип с иконками стоимости и описанием производства
            let px = 2 * s; let char_w = 4 * px; let pad = 5 * s; let gap = 6 * s;
            let icon = 10 * s;
            let cost_w = {
                let w_wood = icon + 4 + number_w(cost.wood.max(0) as u32, s);
                let w_gold = icon + 4 + number_w(cost.gold.max(0) as u32, s);
                w_wood + gap + w_gold
            };
            // Описание производства
            let (prod_label, prod_color, note) = match bk {
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
            };
            let prod_w = icon + 4 + text_w(prod_label, s);
            let note_w = note.map(|t| text_w(t, s)).unwrap_or(0);
            let tip_w = (cost_w.max(prod_w).max(note_w)) + 2 * pad;
            let line_h = icon.max(5 * px) + 2 * s;
            let lines = if note.is_some() { 3 } else { 2 };
            let tip_h = lines * line_h + 2 * s;
            // Позиция над панелью
            let tx_mid = bx + bw / 2;
            let x_tip = (tx_mid - tip_w / 2).clamp(8 * s, fw - tip_w - 8 * s);
            let y_tip = by0 - tip_h - 4 * s;
            // Фон
            fill_rect(frame, fw, fh, x_tip + 2 * s, y_tip + 2 * s, tip_w, tip_h, [0,0,0,100]);
            fill_rect(frame, fw, fh, x_tip, y_tip, tip_w, tip_h, [0,0,0,160]);
            // Строка 1: Cost (иконки + числа)
            let mut cx = x_tip + pad; let cy = y_tip + s + (line_h - icon) / 2;
            // wood
            fill_rect(frame, fw, fh, cx, cy, icon, icon, [110,70,30,255]);
            cx += icon + 4; draw_number(frame, fw, fh, cx, cy - (icon - 5*px)/2, cost.wood.max(0) as u32, [230,230,230,255], s);
            cx += number_w(cost.wood.max(0) as u32, s) + gap;
            // gold
            fill_rect(frame, fw, fh, cx, cy, icon, icon, [220,180,60,255]);
            cx += icon + 4; draw_number(frame, fw, fh, cx, cy - (icon - 5*px)/2, cost.gold.max(0) as u32, [230,230,230,255], s);
            // Строка 2: Produces
            let cy2 = y_tip + line_h + s + (line_h - icon) / 2; let mut cx2 = x_tip + pad;
            fill_rect(frame, fw, fh, cx2, cy2, icon, icon, prod_color);
            cx2 += icon + 4; draw_text_mini(frame, fw, fh, cx2, cy2 - (icon - 5*px)/2, prod_label, [230,230,230,255], s);
            // Строка 3: note (если есть)
            if let Some(t) = note { let cy3 = y_tip + 2*line_h + s + (line_h - 5*px)/2; draw_text_mini(frame, fw, fh, x_tip + pad, cy3, t, [200,200,200,255], s); }
        }
        bx += bw + 6 * s;
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
    // лёгкая верхняя кромка (блик) и нижняя тень для объёма
    fill_rect(frame, fw, fh, x, y, w, (2 * s).max(2), [255, 255, 255, 45]);
    fill_rect(frame, fw, fh, x, y + h - (2 * s).max(2), w, (2 * s).max(2), [0, 0, 0, 40]);
    draw_text_mini(frame, fw, fh, x + 6 * s, y + 4 * s, label, col, s);
}

// Косты для UI: дублируем из игровой логики для отображения
fn building_cost_ui(kind: BuildingKind) -> Resources {
    match kind {
        BuildingKind::Lumberjack => Resources { wood: 5, gold: 10, ..Default::default() },
        BuildingKind::House => Resources { wood: 10, gold: 15, ..Default::default() },
        BuildingKind::Warehouse => Resources { wood: 20, gold: 30, ..Default::default() },
        BuildingKind::Forester => Resources { wood: 15, gold: 20, ..Default::default() },
        BuildingKind::StoneQuarry => Resources { wood: 10, gold: 10, ..Default::default() },
        BuildingKind::ClayPit => Resources { wood: 10, gold: 10, ..Default::default() },
        BuildingKind::Kiln => Resources { wood: 15, gold: 15, ..Default::default() },
        BuildingKind::WheatField => Resources { wood: 5, gold: 5, ..Default::default() },
        BuildingKind::Mill => Resources { wood: 20, gold: 20, ..Default::default() },
        BuildingKind::Bakery => Resources { wood: 20, gold: 25, ..Default::default() },
        BuildingKind::Fishery => Resources { wood: 15, gold: 10, ..Default::default() },
        BuildingKind::IronMine => Resources { wood: 15, gold: 20, ..Default::default() },
        BuildingKind::Smelter => Resources { wood: 20, gold: 25, ..Default::default() },
    }
}

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
        if ch == b'[' || ch == b']' || ch == b'/' || ch == b'\\' || (ch >= b'0' && ch <= b'9') || (ch >= b'A' && ch <= b'Z') || (ch >= b'a' && ch <= b'z') {
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


