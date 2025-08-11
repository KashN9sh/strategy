pub fn draw_iso_tile_tinted(frame: &mut [u8], width: i32, height: i32, cx: i32, cy: i32, half_w: i32, half_h: i32, color: [u8; 4]) {
    for dy in -half_h..=half_h {
        let t = dy.abs() as f32 / half_h.max(1) as f32;
        let row_half = ((1.0 - t) * half_w as f32).round() as i32;
        let y = cy + dy; if y < 0 || y >= height { continue; }
        let x0 = (cx - row_half).clamp(0, width - 1);
        let x1 = (cx + row_half).clamp(0, width - 1);
        for x in x0..=x1 {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let a = color[3] as u32; let na = 255 - a;
            let dr = frame[idx] as u32; let dg = frame[idx+1] as u32; let db = frame[idx+2] as u32;
            frame[idx] = ((a * color[0] as u32 + na * dr) / 255) as u8;
            frame[idx+1] = ((a * color[1] as u32 + na * dg) / 255) as u8;
            frame[idx+2] = ((a * color[2] as u32 + na * db) / 255) as u8;
            frame[idx+3] = 255;
        }
    }
}

pub fn draw_iso_outline(frame: &mut [u8], width: i32, height: i32, cx: i32, cy: i32, half_w: i32, half_h: i32, color: [u8; 4]) {
    let top = (cx, cy - half_h);
    let right = (cx + half_w, cy);
    let bottom = (cx, cy + half_h);
    let left = (cx - half_w, cy);
    draw_line(frame, width, height, top.0, top.1, right.0, right.1, color);
    draw_line(frame, width, height, right.0, right.1, bottom.0, bottom.1, color);
    draw_line(frame, width, height, bottom.0, bottom.1, left.0, left.1, color);
    draw_line(frame, width, height, left.0, left.1, top.0, top.1, color);
}

pub fn draw_line(frame: &mut [u8], width: i32, height: i32, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
    let dx = (x1 - x0).abs(); let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs(); let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x0 >= 0 && y0 >= 0 && x0 < width && y0 < height {
            let idx = ((y0 as usize) * (width as usize) + (x0 as usize)) * 4;
            frame[idx..idx + 4].copy_from_slice(&color);
        }
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err; if e2 >= dy { err += dy; x0 += sx; } if e2 <= dx { err += dx; y0 += sy; }
    }
}

pub fn draw_building(frame: &mut [u8], width: i32, height: i32, cx: i32, cy: i32, half_w: i32, half_h: i32, color: [u8; 4]) {
    let bw = (half_w as f32 * 1.2) as i32; let bh = (half_h as f32 * 1.8) as i32;
    let x0 = (cx - bw / 2).clamp(0, width - 1); let x1 = (cx + bw / 2).clamp(0, width - 1);
    let y0 = (cy - bh).clamp(0, height - 1); let y1 = (cy - bh / 2).clamp(0, height - 1);
    for y in y0..=y1 { for x in x0..=x1 { let idx = ((y as usize) * (width as usize) + (x as usize)) * 4; frame[idx..idx + 4].copy_from_slice(&color); } }
}

pub fn draw_citizen_marker(frame: &mut [u8], width: i32, height: i32, cx: i32, cy: i32, radius: i32, color: [u8;4]) {
    let r = radius.max(1).min(12);
    let r2 = r * r;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx*dx + dy*dy <= r2 {
                let x = cx + dx; let y = cy + dy;
                if x >= 0 && y >= 0 && x < width && y < height {
                    let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
                    frame[idx..idx+4].copy_from_slice(&color);
                }
            }
        }
    }
}

pub fn draw_log(frame: &mut [u8], width: i32, height: i32, cx: i32, cy: i32, half_w: i32, half_h: i32) {
    // маленький прямоугольник как полено
    let w = (half_w as f32 * 0.4) as i32; let h = (half_h as f32 * 0.3) as i32;
    let x0 = (cx - w/2).clamp(0, width-1);
    let y0 = (cy - h/2).clamp(0, height-1);
    let x1 = (cx + w/2).clamp(0, width-1);
    let y1 = (cy + h/2).clamp(0, height-1);
    for y in y0..=y1 { for x in x0..=x1 { let idx=((y as usize)*(width as usize)+(x as usize))*4; frame[idx..idx+4].copy_from_slice(&[120,80,40,255]); }}
}

// Автосоединение дорог: рисует соединения к соседям N/E/S/W поверх базовой заливки дороги
pub fn draw_road_connections(
    frame: &mut [u8], width: i32, height: i32,
    cx: i32, cy: i32, half_w: i32, half_h: i32,
    mask: u8, color: [u8; 4]
) {
    // Центр ромба
    let center = (cx, cy);
    // Цели: середины рёбер ромба
    let top = (cx, cy - half_h);
    let right = (cx + half_w, cy);
    let bottom = (cx, cy + half_h);
    let left = (cx - half_w, cy);
    // Рисуем толстые линии (3px) к активным сторонам
    let mut draw_thick = |x0: i32, y0: i32, x1: i32, y1: i32| {
        // основная линия
        draw_line(frame, width, height, x0, y0, x1, y1, color);
        // смещения по перпендикуляру для толщины
        let dx = x1 - x0; let dy = y1 - y0;
        // нормаль (dy, -dx); нормируем грубо по знаку
        let nx = if dy == 0 { 0 } else { dy.signum() };
        let ny = if dx == 0 { 0 } else { -dx.signum() };
        draw_line(frame, width, height, x0 + nx, y0 + ny, x1 + nx, y1 + ny, color);
        draw_line(frame, width, height, x0 - nx, y0 - ny, x1 - nx, y1 - ny, color);
    };
    if mask & 0b0001 != 0 { draw_thick(center.0, center.1, top.0, top.1); }
    if mask & 0b0010 != 0 { draw_thick(center.0, center.1, right.0, right.1); }
    if mask & 0b0100 != 0 { draw_thick(center.0, center.1, bottom.0, bottom.1); }
    if mask & 0b1000 != 0 { draw_thick(center.0, center.1, left.0, left.1); }
}

pub fn blit_sprite_alpha_scaled(frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32, src: &Vec<u8>, sw: i32, sh: i32, dw: i32, dh: i32) {
    let dst_x0 = x.max(0); let dst_y0 = y.max(0); let dst_x1 = (x + dw).min(fw); let dst_y1 = (y + dh).min(fh);
    if dst_x0 >= dst_x1 || dst_y0 >= dst_y1 { return; }
    for dy in dst_y0..dst_y1 {
        let sy = ((dy - y) as f32 * (sh as f32 - 1.0) / (dh as f32 - 1.0)).round() as i32; let src_row = (sy as usize) * (sw as usize) * 4; let dst_row = (dy as usize) * (fw as usize) * 4;
        for dx in dst_x0..dst_x1 {
            let sx = ((dx - x) as f32 * (sw as f32 - 1.0) / (dw as f32 - 1.0)).round() as i32;
            let sidx = src_row + (sx as usize) * 4; let didx = dst_row + (dx as usize) * 4; let sa = src[sidx + 3] as u32; if sa == 0 { continue; }
            let sr = src[sidx] as u32; let sg = src[sidx + 1] as u32; let sb = src[sidx + 2] as u32; let dr = frame[didx] as u32; let dg = frame[didx + 1] as u32; let db = frame[didx + 2] as u32;
            let a = sa; let na = 255 - a; frame[didx] = ((a * sr + na * dr) / 255) as u8; frame[didx + 1] = ((a * sg + na * dg) / 255) as u8; frame[didx + 2] = ((a * sb + na * db) / 255) as u8; frame[didx + 3] = 255;
        }
    }
}

pub fn blit_sprite_alpha_scaled_tinted(
    frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32,
    src: &Vec<u8>, sw: i32, sh: i32, dw: i32, dh: i32,
    global_alpha: u8,
) {
    let dst_x0 = x.max(0); let dst_y0 = y.max(0); let dst_x1 = (x + dw).min(fw); let dst_y1 = (y + dh).min(fh);
    if dst_x0 >= dst_x1 || dst_y0 >= dst_y1 { return; }
    let ga = global_alpha as u32;
    for dy in dst_y0..dst_y1 {
        let sy = ((dy - y) as f32 * (sh as f32 - 1.0) / (dh as f32 - 1.0)).round() as i32; let src_row = (sy as usize) * (sw as usize) * 4; let dst_row = (dy as usize) * (fw as usize) * 4;
        for dx in dst_x0..dst_x1 {
            let sx = ((dx - x) as f32 * (sw as f32 - 1.0) / (dw as f32 - 1.0)).round() as i32;
            let sidx = src_row + (sx as usize) * 4; let didx = dst_row + (dx as usize) * 4;
            let sa0 = src[sidx + 3] as u32; if sa0 == 0 { continue; }
            let sa = (sa0 * ga) / 255; if sa == 0 { continue; }
            let sr = src[sidx] as u32; let sg = src[sidx + 1] as u32; let sb = src[sidx + 2] as u32; let dr = frame[didx] as u32; let dg = frame[didx + 1] as u32; let db = frame[didx + 2] as u32;
            let na = 255 - sa; frame[didx] = ((sa * sr + na * dr) / 255) as u8; frame[didx + 1] = ((sa * sg + na * dg) / 255) as u8; frame[didx + 2] = ((sa * sb + na * db) / 255) as u8; frame[didx + 3] = 255;
        }
    }
}

// Цветовая тонировка спрайта: смешивает исходный цвет с tint с заданной силой (0..255)
pub fn blit_sprite_alpha_scaled_color_tint(
    frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32,
    src: &Vec<u8>, sw: i32, sh: i32, dw: i32, dh: i32,
    tint_rgb: [u8;3], strength: u8, // 0 — без тона, 255 — полный тон
    global_alpha: u8,
)
{
    let dst_x0 = x.max(0); let dst_y0 = y.max(0); let dst_x1 = (x + dw).min(fw); let dst_y1 = (y + dh).min(fh);
    if dst_x0 >= dst_x1 || dst_y0 >= dst_y1 { return; }
    let ga = global_alpha as u32; let k = strength as i32; let invk = 255 - k;
    for dy in dst_y0..dst_y1 {
        let sy = ((dy - y) as f32 * (sh as f32 - 1.0) / (dh as f32 - 1.0)).round() as i32; let src_row = (sy as usize) * (sw as usize) * 4; let dst_row = (dy as usize) * (fw as usize) * 4;
        for dx in dst_x0..dst_x1 {
            let sx = ((dx - x) as f32 * (sw as f32 - 1.0) / (dw as f32 - 1.0)).round() as i32;
            let sidx = src_row + (sx as usize) * 4; let didx = dst_row + (dx as usize) * 4;
            let sa0 = src[sidx + 3] as u32; if sa0 == 0 { continue; }
            let sa = (sa0 * ga) / 255; if sa == 0 { continue; }
            let sr0 = src[sidx] as i32; let sg0 = src[sidx + 1] as i32; let sb0 = src[sidx + 2] as i32;
            // смешаем с tint
            let sr = ((sr0 * invk + tint_rgb[0] as i32 * k) / 255) as u32;
            let sg = ((sg0 * invk + tint_rgb[1] as i32 * k) / 255) as u32;
            let sb = ((sb0 * invk + tint_rgb[2] as i32 * k) / 255) as u32;
            let dr = frame[didx] as u32; let dg = frame[didx + 1] as u32; let db = frame[didx + 2] as u32;
            let na = 255 - sa;
            frame[didx] = ((sa * sr + na * dr) / 255) as u8;
            frame[didx + 1] = ((sa * sg + na * dg) / 255) as u8;
            frame[didx + 2] = ((sa * sb + na * db) / 255) as u8;
            frame[didx + 3] = 255;
        }
    }
}

// Быстрый вариант: альфа-блит без масштабирования (src и dst одного размера)
pub fn blit_sprite_alpha_noscale_tinted(
    frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32,
    src: &Vec<u8>, w: i32, h: i32, global_alpha: u8,
)
{
    let dst_x0 = x.max(0); let dst_y0 = y.max(0); let dst_x1 = (x + w).min(fw); let dst_y1 = (y + h).min(fh);
    if dst_x0 >= dst_x1 || dst_y0 >= dst_y1 { return; }
    let ga = global_alpha as u32;
    for dy in dst_y0..dst_y1 {
        let sy = dy - y; let src_row = (sy as usize) * (w as usize) * 4; let dst_row = (dy as usize) * (fw as usize) * 4;
        for dx in dst_x0..dst_x1 {
            let sx = dx - x;
            let sidx = src_row + (sx as usize) * 4; let didx = dst_row + (dx as usize) * 4;
            let sa0 = src[sidx + 3] as u32; if sa0 == 0 { continue; }
            let sa = (sa0 * ga) / 255; if sa == 0 { continue; }
            let sr = src[sidx] as u32; let sg = src[sidx + 1] as u32; let sb = src[sidx + 2] as u32;
            let dr = frame[didx] as u32; let dg = frame[didx + 1] as u32; let db = frame[didx + 2] as u32;
            let na = 255 - sa;
            frame[didx] = ((sa * sr + na * dr) / 255) as u8;
            frame[didx + 1] = ((sa * sg + na * dg) / 255) as u8;
            frame[didx + 2] = ((sa * sb + na * db) / 255) as u8;
            frame[didx + 3] = 255;
        }
    }
}

pub fn draw_tree(frame: &mut [u8], width: i32, height: i32, cx: i32, cy: i32, half_w: i32, half_h: i32, stage: u8) {
    let trunk_color = [90, 60, 40, 255];
    let leaf_color = [40, 120, 60, 255];
    let scale = match stage { 0 => 0.5, 1 => 0.8, _ => 1.0 } as f32;
    let trunk_h = ((half_h as f32 * 0.5) * scale).max(2.0) as i32;
    for y in -trunk_h..=0 { set_px(frame, width, height, cx, cy + y, trunk_color); }
    let rw = ((half_w as f32 * 0.4) * scale).max(2.0) as i32;
    let rh = ((half_h as f32 * 0.6) * scale).max(3.0) as i32;
    for yy in -rh..=rh {
        let span = rw - (rw * yy.abs() / rh);
        for xx in -span..=span { set_px(frame, width, height, cx + xx, cy + yy - rh, leaf_color); }
    }
}

fn set_px(frame: &mut [u8], w: i32, h: i32, x: i32, y: i32, rgba: [u8; 4]) {
    if x < 0 || y < 0 || x >= w || y >= h { return; }
    let i = ((y * w + x) * 4) as usize;
    frame[i..i+4].copy_from_slice(&rgba);
}

