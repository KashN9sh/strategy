use crate::types::{Resources, BuildingKind};

pub fn ui_scale(fh: i32, k: f32) -> i32 { (((fh as f32) / 720.0) * k).clamp(1.0, 5.0) as i32 }
pub fn ui_bar_height(fh: i32, s: i32) -> i32 { ((fh as f32 * 0.06).max(24.0) as i32) * s }

pub fn draw_ui(frame: &mut [u8], fw: i32, fh: i32, resources: &Resources, depot_wood: i32, population: i32, selected: BuildingKind, fps: f32, speed: f32, paused: bool, base_scale_k: f32) {
    let s = ui_scale(fh, base_scale_k);
    let bar_h = ui_bar_height(fh, s);
    fill_rect(frame, fw, fh, 0, 0, fw, bar_h, [0, 0, 0, 160]);

    let pad = 8 * s;
    let icon_size = 10 * s;
    let mut x = pad;
    fill_rect(frame, fw, fh, x, pad, icon_size, icon_size, [110, 70, 30, 255]);
    x += icon_size + 4;
    draw_number(frame, fw, fh, x, pad, resources.wood as u32, [255, 255, 255, 255], s);
    x += 50;
    fill_rect(frame, fw, fh, x, pad, icon_size, icon_size, [220, 180, 60, 255]);
    x += icon_size + 4;
    draw_number(frame, fw, fh, x, pad, resources.gold as u32, [255, 255, 255, 255], s);

    let btn_w = 90 * s; let btn_h = 18 * s; let by = pad + icon_size + 8 * s;
    draw_button(frame, fw, fh, pad, by, btn_w, btn_h, selected == BuildingKind::Lumberjack, b"Lumberjack [Z]", [200,200,200,255], s);
    draw_button(frame, fw, fh, pad + btn_w + 6 * s, by, btn_w, btn_h, selected == BuildingKind::House, b"House [X]", [200,200,200,255], s);
    draw_button(frame, fw, fh, pad + (btn_w + 6 * s) * 2, by, btn_w, btn_h, selected == BuildingKind::Warehouse, b"Warehouse", [200,200,200,255], s);

    let info_x = fw - 160 * s; let info_y = 8 * s;
    draw_text_mini(frame, fw, fh, info_x, info_y, b"FPS:", [200,200,200,255], s);
    draw_number(frame, fw, fh, info_x + 20 * s, info_y, fps.round() as u32, [255,255,255,255], s);
    draw_text_mini(frame, fw, fh, info_x, info_y + 10 * s, if paused { b"PAUSE" } else { b"SPEED" }, [200,200,200,255], s);
    if !paused { let sp = (speed * 10.0).round() as u32; draw_number(frame, fw, fh, info_x + 36 * s, info_y + 10 * s, sp, [255,255,255,255], s); }

    // Склад (итого дерево в складах)
    let dep_x = pad + 120 * s;
    fill_rect(frame, fw, fh, dep_x, pad, icon_size, icon_size, [90, 90, 120, 255]);
    draw_number(frame, fw, fh, dep_x + icon_size + 4, pad, depot_wood as u32, [255,255,255,255], s);

    // Население
    let pop_x = dep_x + 80 * s; // немного правее склада
    let pop_y = pad;
    fill_rect(frame, fw, fh, pop_x, pop_y, icon_size, icon_size, [180, 60, 60, 255]);
    draw_number(frame, fw, fh, pop_x + icon_size + 4, pop_y, population as u32, [255,255,255,255], s);
}

pub fn point_in_rect(px: i32, py: i32, x: i32, y: i32, w: i32, h: i32) -> bool { px >= x && py >= y && px < x + w && py < y + h }

fn draw_button(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, w: i32, h: i32, active: bool, label: &[u8], col: [u8;4], s: i32) {
    let bg = if active { [70, 120, 220, 200] } else { [50, 50, 50, 160] };
    fill_rect(frame, fw, fh, x, y, w, h, bg);
    draw_text_mini(frame, fw, fh, x + 6 * s, y + 4 * s, label, col, s);
}

fn draw_text_mini(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, text: &[u8], color: [u8;4], s: i32) {
    let mut cx = x; let cy = y;
    for &ch in text {
        if ch == b' ' { cx += 4; continue; }
        if ch == b'[' || ch == b']' || ch == b'/' || ch == b'\\' || (ch >= b'0' && ch <= b'9') || (ch >= b'A' && ch <= b'Z') || (ch >= b'a' && ch <= b'z') {
            draw_glyph_3x5(frame, fw, fh, cx, cy, ch, color, s);
            cx += 4 * s;
        } else { cx += 4 * s; }
    }
}

fn draw_glyph_3x5(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, ch: u8, color: [u8;4], s: i32) {
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
        b'H' => [1,0,1, 1,0,1, 1,1,1, 1,0,1, 1,0,1],
        b'L' => [1,0,0, 1,0,0, 1,0,0, 1,0,0, 1,1,1],
        b'R' => [1,1,0, 1,0,1, 1,1,0, 1,0,1, 1,0,1],
        b'U' => [1,0,1, 1,0,1, 1,0,1, 1,0,1, 1,1,1],
        b'Z' => [1,1,1, 0,0,1, 0,1,0, 1,0,0, 1,1,1],
        b'X' => [1,0,1, 1,0,1, 0,1,0, 1,0,1, 1,0,1],
        b'[' => [1,1,0, 1,0,0, 1,0,0, 1,0,0, 1,1,0],
        b']' => [0,1,1, 0,0,1, 0,0,1, 0,0,1, 0,1,1],
        b'/' => [0,0,1, 0,1,0, 0,1,0, 1,0,0, 1,0,0],
        b'\\' => [1,0,0, 1,0,0, 0,1,0, 0,1,0, 0,0,1],
        _ => [1,1,1, 1,1,1, 1,1,1, 1,1,1, 1,1,1],
    };
    for row in 0..5 { for cx_i in 0..3 { if pattern[row*3 + cx_i] == 1 { fill_rect(frame, fw, fh, x + cx_i as i32 * s, y + row as i32 * s, 1 * s, 1 * s, color); } } }
}

fn draw_number(frame: &mut [u8], fw: i32, fh: i32, mut x: i32, y: i32, mut n: u32, col: [u8;4], s: i32) {
    let mut digits: [u8; 12] = [0; 12]; let mut len = 0; if n == 0 { digits[0] = b'0'; len = 1; }
    while n > 0 && len < digits.len() { let d = (n % 10) as u8; n /= 10; digits[len] = b'0' + d; len += 1; }
    for i in (0..len).rev() { draw_glyph_3x5(frame, fw, fh, x, y, digits[i], col, s); x += 4 * s; }
}

fn fill_rect(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
    let x0 = x.max(0); let y0 = y.max(0); let x1 = (x + w).min(fw); let y1 = (y + h).min(fh);
    if x0 >= x1 || y0 >= y1 { return; }
    for yy in y0..y1 { let row = (yy as usize) * (fw as usize) * 4; for xx in x0..x1 {
        let idx = row + (xx as usize) * 4; let a = color[3] as u32; let na = 255 - a; let dr = frame[idx] as u32; let dg = frame[idx+1] as u32; let db = frame[idx+2] as u32;
        frame[idx] = ((a * color[0] as u32 + na * dr) / 255) as u8; frame[idx+1] = ((a * color[1] as u32 + na * dg) / 255) as u8; frame[idx+2] = ((a * color[2] as u32 + na * db) / 255) as u8; frame[idx+3] = 255;
    }}
}


