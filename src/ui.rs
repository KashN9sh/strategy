use crate::types::{Resources, BuildingKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UICategory { Housing, Storage, Forestry, Mining, Food, Logistics }

pub fn ui_scale(fh: i32, k: f32) -> i32 { (((fh as f32) / 720.0) * k).clamp(1.0, 5.0) as i32 }
pub fn ui_bar_height(fh: i32, s: i32) -> i32 { ((fh as f32 * 0.06).max(24.0) as i32) * s }
pub fn bottom_panel_height(s: i32) -> i32 { let padb = 8 * s; let btn_h = 18 * s; padb * 2 + btn_h * 2 + 6 * s }
pub fn top_panel_height(s: i32) -> i32 { let pad = 8 * s; let icon = 10 * s; let px = 2 * s; let glyph_h = 5 * px; pad * 2 + icon.max(glyph_h) }

pub fn draw_ui(frame: &mut [u8], fw: i32, fh: i32, resources: &Resources, total_wood: i32, population: i32, selected: BuildingKind, fps: f32, speed: f32, paused: bool, base_scale_k: f32, category: UICategory) {
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
    x = num_x_pop + reserved_w + gap;
    // золото
    fill_rect(frame, fw, fh, x, pad, icon_size, icon_size, [220, 180, 60, 255]);
    let num_x_gold = x + icon_size + 4;
    draw_number(frame, fw, fh, num_x_gold, pad, resources.gold.max(0) as u32, [255,255,255,255], s);
    x = num_x_gold + reserved_w + gap;

    // верх: больше не рисуем кнопки строительства — они в нижней панели

    // Инфо справа: одна строка, правое выравнивание: "FPS <n>  SPEED <m>" или "FPS <n>  PAUSE"
    let px = 2 * s; let gap_info = 2 * px; let y_info = 8 * s; let pad_right = 8 * s;
    let fps_n = fps.round() as u32;
    let fps_w = text_w(b"FPS", s) + gap_info + number_w(fps_n, s);
    let speed_w = if paused { text_w(b"PAUSE", s) } else { let sp = (speed * 10.0).round() as u32; text_w(b"SPEED", s) + gap_info + number_w(sp, s) };
    let total_w = fps_w + gap_info + speed_w;
    let mut ix = fw - pad_right - total_w;
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

    // Панель дополнительных ресурсов (камень, глина, кирпич, пшеница, мука, хлеб, рыба)
    let mut rx = x + 40 * s;
    let ry = pad;
    let entry = |frame: &mut [u8], x: &mut i32, label_color: [u8;4], val: i32, s: i32| {
        let px = 2 * s; // масштаб шрифта
        let reserve_digits = 4; // резерв под 4 цифры
        let reserved_w = reserve_digits * 4 * px; // ширина цифр с шагом 4*px
        let gap = 6 * s; // межколонный отступ
        // иконка
        fill_rect(frame, fw, fh, *x, ry, icon_size, icon_size, label_color);
        // число
        let num_x = *x + icon_size + 4;
        draw_number(frame, fw, fh, num_x, ry, val.max(0) as u32, [255,255,255,255], s);
        // сдвиг курсора строго на иконку + резерв под 4 цифры + зазор
        *x = num_x + reserved_w + gap;
    };
    let mut xcur = rx;
    // Дерево (из складов или видимое общее)
    entry(frame, &mut xcur, [110,70,30,255], total_wood, s);
    entry(frame, &mut xcur, [120,120,120,255], resources.stone, s);
    entry(frame, &mut xcur, [150,90,70,255], resources.clay, s);
    entry(frame, &mut xcur, [180,120,90,255], resources.bricks, s);
    entry(frame, &mut xcur, [200,180,80,255], resources.wheat, s);
    entry(frame, &mut xcur, [210,210,180,255], resources.flour, s);
    entry(frame, &mut xcur, [200,160,120,255], resources.bread, s);
    entry(frame, &mut xcur, [100,140,200,255], resources.fish, s);

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
        draw_button(frame, fw, fh, cx, by0 + padb, bw, btn_h, active, label, [200,200,200,255], s);
        cx += bw + 6 * s;
    }
    // Здания по выбранной категории (нижняя строка)
    let mut bx = padb;
    let by = by0 + padb + btn_h + 6 * s;
    let buildings_for_cat: &[BuildingKind] = match category {
        UICategory::Housing => &[BuildingKind::House],
        UICategory::Storage => &[BuildingKind::Warehouse],
        UICategory::Forestry => &[BuildingKind::Lumberjack, BuildingKind::Forester],
        UICategory::Mining => &[BuildingKind::StoneQuarry, BuildingKind::ClayPit, BuildingKind::Kiln],
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
            BuildingKind::WheatField => b"Wheat Field".as_ref(),
            BuildingKind::Mill => b"Mill".as_ref(),
            BuildingKind::Bakery => b"Bakery".as_ref(),
            BuildingKind::Fishery => b"Fishery".as_ref(),
        };
        let active = selected == bk;
        let bw = button_w_for(label, s);
        if bx + bw > fw - padb { break; }
        draw_button(frame, fw, fh, bx, by, bw, btn_h, active, label, [220,220,220,255], s);
        bx += bw + 6 * s;
    }
}

pub fn point_in_rect(px: i32, py: i32, x: i32, y: i32, w: i32, h: i32) -> bool { px >= x && py >= y && px < x + w && py < y + h }

fn draw_button(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, w: i32, h: i32, active: bool, label: &[u8], col: [u8;4], s: i32) {
    let bg = if active { [70, 120, 220, 200] } else { [50, 50, 50, 160] };
    fill_rect(frame, fw, fh, x, y, w, h, bg);
    draw_text_mini(frame, fw, fh, x + 6 * s, y + 4 * s, label, col, s);
}

pub fn draw_tooltip(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, text: &[u8], s: i32) {
    // вычислим примитивную ширину по числу символов (4*s на символ)
    let width = (text.len() as i32) * 4 * s + 8 * s;
    let height = 12 * s;
    fill_rect(frame, fw, fh, x, y, width, height, [0, 0, 0, 200]);
    draw_text_mini(frame, fw, fh, x + 4 * s, y + 4 * s, text, [230,230,230,255], s);
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


