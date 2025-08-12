use crate::types::TileKind;

pub const TILE_W: i32 = 32;
pub const TILE_H: i32 = 16;

pub struct TileAtlas {
    pub zoom_px: i32,
    pub half_w: i32,
    pub half_h: i32,
    pub grass: Vec<u8>,
    pub forest: Vec<u8>,
    pub water_frames: Vec<Vec<u8>>,
    // вариативные PNG-спрайты
    pub grass_variants: Vec<Vec<u8>>, // для целикового блита
    pub clay_variants: Vec<Vec<u8>>,  // варианты глины из спрайтшита
    pub water_edges: Vec<Vec<u8>>,    // кромки воды (последняя строка, ячейки 2..8)
    // предмасштабированные и замаскированные наложения месторождений
    pub clay: Vec<u8>,
    pub stone: Vec<u8>,
    pub iron: Vec<u8>,
    pub base_loaded: bool,
    pub base_w: i32,
    pub base_h: i32,
    pub base_grass: Vec<u8>,
    pub base_forest: Vec<u8>,
    pub base_water: Vec<u8>,
    pub base_clay: Vec<u8>,
    pub base_stone: Vec<u8>,
    pub base_iron: Vec<u8>,
}

impl TileAtlas {
    pub fn new() -> Self {
        Self { zoom_px: -1, half_w: 0, half_h: 0, grass: Vec::new(), forest: Vec::new(), water_frames: Vec::new(), grass_variants: Vec::new(), clay_variants: Vec::new(), water_edges: Vec::new(), clay: Vec::new(), stone: Vec::new(), iron: Vec::new(), base_loaded: false, base_w: 0, base_h: 0, base_grass: Vec::new(), base_forest: Vec::new(), base_water: Vec::new(), base_clay: Vec::new(), base_stone: Vec::new(), base_iron: Vec::new() }
    }

    pub fn ensure_zoom(&mut self, zoom: f32) {
        let half_w = ((TILE_W as f32 / 2.0) * zoom).round() as i32;
        let half_h = ((TILE_H as f32 / 2.0) * zoom).round() as i32;
        let zoom_px = half_w.max(1) * 2 + 1;
        if zoom_px == self.zoom_px && self.half_w == half_w && self.half_h == half_h { return; }
        self.zoom_px = zoom_px;
        self.half_w = half_w.max(1);
        self.half_h = half_h.max(1);
        if self.base_loaded {
            // безопасные версии: если источник пустой — вернём пустой буфер нужного размера
            self.grass = Self::scale_and_mask_or_empty(&self.base_grass, self.base_w, self.base_h, self.half_w, self.half_h);
            self.forest = Self::scale_and_mask_or_empty(&self.base_forest, self.base_w, self.base_h, self.half_w, self.half_h);
            self.water_frames.clear();
            self.water_frames.push(Self::scale_and_mask_or_empty(&self.base_water, self.base_w, self.base_h, self.half_w, self.half_h));
            // подготовим наложения deposit-тайлов под текущий zoom с ромбической маской
            self.clay = Self::scale_and_mask_or_empty(&self.base_clay, self.base_w, self.base_h, self.half_w, self.half_h);
            self.stone = Self::scale_and_mask_or_empty(&self.base_stone, self.base_w, self.base_h, self.half_w, self.half_h);
            self.iron = Self::scale_and_mask_or_empty(&self.base_iron, self.base_w, self.base_h, self.half_w, self.half_h);
        } else {
            self.grass = Self::build_tile(self.half_w, self.half_h, [40, 120, 80, 255]);
            self.forest = Self::build_tile(self.half_w, self.half_h, [26, 100, 60, 255]);
            self.water_frames.clear();
            let frames = 8;
            for phase in 0..frames { self.water_frames.push(Self::build_water_tile(self.half_w, self.half_h, phase, frames)); }
        }
    }

    fn build_tile(half_w: i32, half_h: i32, color: [u8; 4]) -> Vec<u8> {
        let w = half_w * 2 + 1; let h = half_h * 2 + 1; let mut buf = vec![0u8; (w * h * 4) as usize];
        for dy in -half_h..=half_h {
            let t = dy.abs() as f32 / half_h.max(1) as f32;
            let row_half = ((1.0 - t) * half_w as f32).round() as i32;
            let y = dy + half_h; let x0 = half_w - row_half; let x1 = half_w + row_half; let base = (y as usize) * (w as usize) * 4;
            for x in x0..=x1 { let idx = base + (x as usize) * 4; buf[idx..idx + 4].copy_from_slice(&color); }
        }
        buf
    }

    fn build_water_tile(half_w: i32, half_h: i32, phase: i32, frames: i32) -> Vec<u8> {
        let w = half_w * 2 + 1; let h = half_h * 2 + 1; let mut buf = vec![0u8; (w * h * 4) as usize];
        for dy in -half_h..=half_h {
            let t = dy.abs() as f32 / half_h.max(1) as f32; let row_half = ((1.0 - t) * half_w as f32).round() as i32;
            let y = dy + half_h; let x0 = half_w - row_half; let x1 = half_w + row_half; let base = (y as usize) * (w as usize) * 4;
            for x in x0..=x1 {
                let fx = (x - half_w) as f32 / half_w.max(1) as f32; let ph = (phase as f32) / frames as f32;
                let wave = (fx * std::f32::consts::PI + ph * 2.0 * std::f32::consts::PI).sin();
                let mut r = 28.0 + wave * 4.0; let mut g = 64.0 + wave * 10.0; let mut b = 120.0 + wave * 16.0;
                r = r.clamp(0.0, 255.0); g = g.clamp(0.0, 255.0); b = b.clamp(0.0, 255.0);
                let idx = base + (x as usize) * 4; buf[idx] = r as u8; buf[idx + 1] = g as u8; buf[idx + 2] = b as u8; buf[idx + 3] = 255;
            }
        }
        buf
    }

    fn scale_and_mask(base: &Vec<u8>, bw: i32, bh: i32, half_w: i32, half_h: i32) -> Vec<u8> {
        let w = half_w * 2 + 1; let h = half_h * 2 + 1; let mut buf = vec![0u8; (w * h * 4) as usize];
        for dy in -half_h..=half_h {
            let t = dy.abs() as f32 / half_h.max(1) as f32; let row_half = ((1.0 - t) * half_w as f32).round() as i32;
            let y = dy + half_h; let x0 = half_w - row_half; let x1 = half_w + row_half;
            for x in x0..=x1 {
                let gx = x; let gy = y;
                let src_x = ((gx as f32) * (bw as f32 - 1.0) / (w as f32 - 1.0)).round() as i32;
                let src_y = ((gy as f32) * (bh as f32 - 1.0) / (h as f32 - 1.0)).round() as i32;
                let sidx = ((src_y as usize) * (bw as usize) + (src_x as usize)) * 4;
                let didx = ((gy as usize) * (w as usize) + (gx as usize)) * 4;
                buf[didx..didx + 4].copy_from_slice(&base[sidx..sidx + 4]);
            }
        }
        buf
    }

    fn scale_and_mask_or_empty(base: &Vec<u8>, bw: i32, bh: i32, half_w: i32, half_h: i32) -> Vec<u8> {
        if base.is_empty() || bw <= 0 || bh <= 0 { return vec![0u8; ((half_w * 2 + 1) * (half_h * 2 + 1) * 4) as usize]; }
        Self::scale_and_mask(base, bw, bh, half_w, half_h)
    }

    pub fn blit(&self, frame: &mut [u8], fw: i32, fh: i32, cx: i32, cy: i32, kind: TileKind, water_frame: usize) {
        let src = match kind {
            TileKind::Grass => &self.grass,
            TileKind::Forest => &self.forest,
            TileKind::Water => &self.water_frames[water_frame % self.water_frames.len().max(1)],
        };
        let w = self.half_w * 2 + 1; let h = self.half_h * 2 + 1; let x0 = cx - self.half_w; let y0 = cy - self.half_h;
        let dst_y_start = y0.max(0); let dst_y_end = (y0 + h).min(fh); if dst_y_start >= dst_y_end { return; }
        for dy in dst_y_start..dst_y_end {
            let sy = dy - y0; let from_center = sy - self.half_h;
            let t = (from_center.unsigned_abs() as f32) / (self.half_h.max(1) as f32);
            let row_half = ((1.0 - t) * self.half_w as f32).round() as i32;
            let src_x0 = self.half_w - row_half; let src_x1 = self.half_w + row_half + 1;
            let row_dst_x0 = x0 + src_x0; let row_dst_x1 = x0 + src_x1;
            let dst_x0 = row_dst_x0.max(0); let dst_x1 = row_dst_x1.min(fw); if dst_x0 >= dst_x1 { continue; }
            let cut_left = dst_x0 - row_dst_x0; let src_copy_x0 = src_x0 + cut_left; let src_row = (sy as usize) * (w as usize) * 4;
            let src_slice = &src[(src_row + (src_copy_x0 as usize) * 4)..(src_row + ((src_copy_x0 + (dst_x1 - dst_x0)) as usize) * 4)];
            let dst_row = (dy as usize) * (fw as usize) * 4; let dst_slice = &mut frame[(dst_row + (dst_x0 as usize) * 4)..(dst_row + (dst_x1 as usize) * 4)];
            dst_slice.copy_from_slice(src_slice);
        }
    }
}

pub struct BuildingAtlas { pub sprites: Vec<Vec<u8>>, pub w: i32, pub h: i32 }

// Отдельный атлас деревьев (кадры по горизонтали: стадии роста)
pub struct TreeAtlas { pub sprites: Vec<Vec<u8>>, pub w: i32, pub h: i32 }

// Временный процедурный атлас дорог для автосоединений (16 масок N/E/S/W)
pub struct RoadAtlas {
    pub sprites: Vec<Vec<u8>>, // 16 элементов
    pub w: i32,
    pub h: i32,
    zoom_px: i32,
    half_w: i32,
    half_h: i32,
}

// Единое соответствие вида здания индексу в `BuildingAtlas`
pub fn building_sprite_index(kind: crate::types::BuildingKind) -> Option<usize> {
    use crate::types::BuildingKind::*;
    match kind {
        Lumberjack => Some(0),
        House => Some(1),
        Warehouse => Some(2),
        Forester => Some(3),
        StoneQuarry => Some(4),
        ClayPit => Some(5),
        Kiln => Some(6),
        WheatField => Some(7),
        Mill => Some(8),
        Bakery => Some(9),
        Fishery => Some(10),
        IronMine => Some(11),
        Smelter => Some(12),
    }
}

impl RoadAtlas {
    pub fn new() -> Self { Self { sprites: Vec::new(), w: 0, h: 0, zoom_px: -1, half_w: 0, half_h: 0 } }

    pub fn ensure_zoom(&mut self, half_w: i32, half_h: i32) {
        let zoom_px = half_w.max(1) * 2 + 1;
        if zoom_px == self.zoom_px && self.half_w == half_w && self.half_h == half_h { return; }
        self.zoom_px = zoom_px; self.half_w = half_w; self.half_h = half_h; self.w = zoom_px; self.h = half_h * 2 + 1;
        self.sprites.clear(); self.sprites.resize(16, vec![0u8; (self.w * self.h * 4) as usize]);
        for mask in 0..16 { self.build_sprite(mask as u8); }
    }

    fn build_sprite(&mut self, mask: u8) {
        let mut buf = vec![0u8; (self.w * self.h * 4) as usize];
        // базовая ромбическая заливка дороги
        Self::draw_diamond(&mut buf, self.w, self.half_w, self.half_h, [120, 110, 90, 220]);
        // соединительные «полосы» — заранее заметные
        let col = [95, 85, 70, 255];
        let center = (self.half_w, self.half_h);
        let top = (self.half_w, 0);
        let right = (self.w - 1, self.half_h);
        let bottom = (self.half_w, self.h - 1);
        let left = (0, self.half_h);
        let mut thick = |x0: i32, y0: i32, x1: i32, y1: i32| {
            Self::draw_line_rgba(&mut buf, self.w, self.h, x0, y0, x1, y1, col);
            // утолщение 2 пикселя по нормали
            let dx = x1 - x0; let dy = y1 - y0; let nx = if dy == 0 { 0 } else { dy.signum() }; let ny = if dx == 0 { 0 } else { -dx.signum() };
            Self::draw_line_rgba(&mut buf, self.w, self.h, x0 + nx, y0 + ny, x1 + nx, y1 + ny, col);
            Self::draw_line_rgba(&mut buf, self.w, self.h, x0 - nx, y0 - ny, x1 - nx, y1 - ny, col);
        };
        if mask & 0b0001 != 0 { thick(center.0, center.1, top.0, top.1); }
        if mask & 0b0010 != 0 { thick(center.0, center.1, right.0, right.1); }
        if mask & 0b0100 != 0 { thick(center.0, center.1, bottom.0, bottom.1); }
        if mask & 0b1000 != 0 { thick(center.0, center.1, left.0, left.1); }
        // лёгкий светлый гребень по центру для читаемости
        Self::draw_line_rgba(&mut buf, self.w, self.h, left.0, left.1, right.0, right.1, [140,130,110,255]);
        Self::draw_line_rgba(&mut buf, self.w, self.h, top.0, top.1, bottom.0, bottom.1, [140,130,110,255]);
        self.sprites[mask as usize] = buf;
    }

    fn draw_diamond(buf: &mut [u8], w: i32, half_w: i32, half_h: i32, color: [u8;4]) {
        let tile_w = half_w * 2 + 1; let tile_h = half_h * 2 + 1;
        for dy in -half_h..=half_h {
            let t = dy.abs() as f32 / half_h.max(1) as f32;
            let row_half = ((1.0 - t) * half_w as f32).round() as i32;
            let y = dy + half_h; let x0 = half_w - row_half; let x1 = half_w + row_half;
            let base = (y as usize) * (w as usize) * 4;
            for x in x0..=x1 { let idx = base + (x as usize) * 4; buf[idx..idx + 4].copy_from_slice(&color); }
        }
        // остальное остаётся прозрачным
        let _ = tile_w; let _ = tile_h;
    }

    fn draw_line_rgba(buf: &mut [u8], w: i32, h: i32, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: [u8;4]) {
        let dx = (x1 - x0).abs(); let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs(); let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            if x0 >= 0 && y0 >= 0 && x0 < w && y0 < h {
                let idx = ((y0 as usize) * (w as usize) + (x0 as usize)) * 4;
                // альфа-композит поверх
                let a = color[3] as u32; let na = 255 - a;
                let dr = buf[idx] as u32; let dg = buf[idx+1] as u32; let db = buf[idx+2] as u32;
                buf[idx] = ((a * color[0] as u32 + na * dr) / 255) as u8;
                buf[idx+1] = ((a * color[1] as u32 + na * dg) / 255) as u8;
                buf[idx+2] = ((a * color[2] as u32 + na * db) / 255) as u8;
                buf[idx+3] = 255;
            }
            if x0 == x1 && y0 == y1 { break; }
            let e2 = 2 * err; if e2 >= dy { err += dy; x0 += sx; } if e2 <= dx { err += dx; y0 += sy; }
        }
    }
}

