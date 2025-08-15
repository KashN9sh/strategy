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

pub fn draw_emote(frame: &mut [u8], width: i32, height: i32, x: i32, y: i32, kind: u8, s: i32) {
    // kind: 0 sad, 1 neutral, 2 happy
    let px = s.max(1);
    match kind {
        0 => { // sad face
            // eyes
            set_px(frame, width, height, x - 2*px, y, [255,255,255,255]);
            set_px(frame, width, height, x + 2*px, y, [255,255,255,255]);
            // mouth frown
            for dx in -2*px..=2*px { set_px(frame, width, height, x + dx, y + 3*px, [255,200,200,255]); }
            set_px(frame, width, height, x - 3*px, y + 2*px, [255,200,200,255]);
            set_px(frame, width, height, x + 3*px, y + 2*px, [255,200,200,255]);
        }
        2 => { // happy
            set_px(frame, width, height, x - 2*px, y, [255,255,255,255]);
            set_px(frame, width, height, x + 2*px, y, [255,255,255,255]);
            for dx in -2*px..=2*px { set_px(frame, width, height, x + dx, y + 2*px, [255,255,0,255]); }
            set_px(frame, width, height, x - 3*px, y + 1*px, [255,255,0,255]);
            set_px(frame, width, height, x + 3*px, y + 1*px, [255,255,0,255]);
        }
        _ => { // neutral
            set_px(frame, width, height, x - 2*px, y, [255,255,255,255]);
            set_px(frame, width, height, x + 2*px, y, [255,255,255,255]);
            for dx in -2*px..=2*px { set_px(frame, width, height, x + dx, y + 2*px, [220,220,220,255]); }
        }
    }
}

pub fn draw_emote_on_marker(frame: &mut [u8], width: i32, height: i32, cx: i32, cy: i32, r: i32, kind: u8) {
    let t = (r / 4).max(1);
    let eye_dx = (r / 2).max(2);
    let eye_y = cy - t;
    // draw 2x2 (or t x t) eyes
    for dy in 0..t { for dx in 0..t {
        set_px(frame, width, height, cx - eye_dx + dx, eye_y + dy, [255,255,255,255]);
        set_px(frame, width, height, cx + eye_dx - dx, eye_y + dy, [255,255,255,255]);
    }}
    // mouth
    let mouth_w = (r as f32 * 1.2).round() as i32; // slightly wider
    let mx0 = cx - mouth_w/2; let mx1 = cx + mouth_w/2;
    let my = cy + t; // baseline inside circle
    let (col, up) = match kind { 2 => ([255,255,0,255], 1), 0 => ([255,160,160,255], -1), _ => ([220,220,220,255], 0) };
    for yoff in 0..t { for x in mx0..=mx1 { set_px(frame, width, height, x, my + yoff, col); }}
    // curve hint at corners
    if up != 0 {
        for k in 0..=t { set_px(frame, width, height, mx0, my - up * (t - k), col); set_px(frame, width, height, mx1, my - up * (t - k), col); }
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

// удалено: draw_road_connections (не используется)

pub use crate::render::utils::{
    blit_sprite_alpha_scaled,
    blit_sprite_alpha_scaled_tinted,
    blit_sprite_alpha_scaled_color_tint,
    blit_sprite_alpha_noscale_tinted,
};

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

