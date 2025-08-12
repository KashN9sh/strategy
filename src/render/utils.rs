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

pub fn blit_sprite_alpha_scaled_color_tint(
    frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32,
    src: &Vec<u8>, sw: i32, sh: i32, dw: i32, dh: i32,
    tint_rgb: [u8;3], strength: u8, global_alpha: u8,
) {
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
            let sr = ((sr0 * invk + tint_rgb[0] as i32 * k) / 255) as u32;
            let sg = ((sg0 * invk + tint_rgb[1] as i32 * k) / 255) as u32;
            let sb = ((sb0 * invk + tint_rgb[2] as i32 * k) / 255) as u32;
            let dr = frame[didx] as u32; let dg = frame[didx + 1] as u32; let db = frame[didx + 2] as u32;
            let na = 255 - sa; frame[didx] = ((sa * sr + na * dr) / 255) as u8; frame[didx + 1] = ((sa * sg + na * dg) / 255) as u8; frame[didx + 2] = ((sa * sb + na * db) / 255) as u8; frame[didx + 3] = 255;
        }
    }
}

pub fn blit_sprite_alpha_noscale_tinted(
    frame: &mut [u8], fw: i32, fh: i32, x: i32, y: i32,
    src: &Vec<u8>, w: i32, h: i32, global_alpha: u8,
) {
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


