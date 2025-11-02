
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
    // Предтюненные варианты для производительности
    pub grass_swamp: Vec<u8>,
    pub grass_rocky: Vec<u8>,
    pub forest_swamp: Vec<u8>,
    pub forest_rocky: Vec<u8>,
    pub clay_tinted: Vec<u8>,
    pub stone_tinted: Vec<u8>,
    pub iron_tinted: Vec<u8>,
    pub clay_variants_tinted: Vec<Vec<u8>>,
}

impl TileAtlas {
    pub fn new() -> Self {
        Self { zoom_px: -1, half_w: 0, half_h: 0, grass: Vec::new(), forest: Vec::new(), water_frames: Vec::new(), grass_variants: Vec::new(), clay_variants: Vec::new(), water_edges: Vec::new(), clay: Vec::new(), stone: Vec::new(), iron: Vec::new(), base_loaded: false, base_w: 0, base_h: 0, base_grass: Vec::new(), base_forest: Vec::new(), base_water: Vec::new(), base_clay: Vec::new(), base_stone: Vec::new(), base_iron: Vec::new(), grass_swamp: Vec::new(), grass_rocky: Vec::new(), forest_swamp: Vec::new(), forest_rocky: Vec::new(), clay_tinted: Vec::new(), stone_tinted: Vec::new(), iron_tinted: Vec::new(), clay_variants_tinted: Vec::new() }
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

            // Предтюненные биомные тайлы
            self.grass_swamp = Self::tint_buffer_color(&self.grass, [50,110,70], 90, 255);
            self.grass_rocky = Self::tint_buffer_color(&self.grass, [150,150,150], 90, 255);
            self.forest_swamp = Self::tint_buffer_color(&self.forest, [50,110,70], 80, 255);
            self.forest_rocky = Self::tint_buffer_color(&self.forest, [150,150,150], 80, 255);
            // Предтюненные оверлеи месторождений
            self.clay_tinted = Self::tint_buffer_color(&self.clay, [170,100,80], 120, 230);
            self.stone_tinted = Self::tint_buffer_alpha(&self.stone, 220);
            self.iron_tinted = Self::tint_buffer_color(&self.iron, [200,205,220], 140, 240);
            // Варианты глины — тоже предтюнить, если загружены
            self.clay_variants_tinted.clear();
            if !self.clay_variants.is_empty() {
                self.clay_variants_tinted.reserve(self.clay_variants.len());
                for spr in &self.clay_variants { self.clay_variants_tinted.push(Self::tint_buffer_color(spr, [170,100,80], 120, 230)); }
            }
        } else {
            self.grass = Self::build_tile(self.half_w, self.half_h, [40, 120, 80, 255]);
            self.forest = Self::build_tile(self.half_w, self.half_h, [26, 100, 60, 255]);
            self.water_frames.clear();
            let frames = 8;
            for phase in 0..frames { self.water_frames.push(Self::build_water_tile(self.half_w, self.half_h, phase, frames)); }
            // на процедурном пути тюны не нужны
            self.grass_swamp = self.grass.clone(); self.grass_rocky = self.grass.clone();
            self.forest_swamp = self.forest.clone(); self.forest_rocky = self.forest.clone();
            self.clay_tinted = self.clay.clone(); self.stone_tinted = self.stone.clone(); self.iron_tinted = self.iron.clone();
            self.clay_variants_tinted.clear();
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

    fn tint_buffer_color(src: &Vec<u8>, tint_rgb: [u8;3], strength: u8, global_alpha: u8) -> Vec<u8> {
        let mut out = src.clone();
        let k = strength as i32; let invk = 255 - k; let ga = global_alpha as u32;
        let len = out.len(); let mut i = 0;
        while i + 3 < len {
            let sr0 = out[i] as i32; let sg0 = out[i+1] as i32; let sb0 = out[i+2] as i32; let sa0 = out[i+3] as u32;
            if sa0 == 0 { i += 4; continue; }
            let sr = ((sr0 * invk + tint_rgb[0] as i32 * k) / 255) as u32;
            let sg = ((sg0 * invk + tint_rgb[1] as i32 * k) / 255) as u32;
            let sb = ((sb0 * invk + tint_rgb[2] as i32 * k) / 255) as u32;
            let sa = (sa0 * ga) / 255;
            out[i] = sr as u8; out[i+1] = sg as u8; out[i+2] = sb as u8; out[i+3] = sa as u8;
            i += 4;
        }
        out
    }

    fn tint_buffer_alpha(src: &Vec<u8>, global_alpha: u8) -> Vec<u8> {
        let mut out = src.clone();
        let ga = global_alpha as u32; let len = out.len(); let mut i = 0;
        while i + 3 < len { let sa0 = out[i+3] as u32; let sa = (sa0 * ga) / 255; out[i+3] = sa as u8; i += 4; }
        out
    }

}

pub struct BuildingAtlas { pub w: i32, pub h: i32 }

// Отдельный атлас деревьев (кадры по горизонтали: стадии роста)
pub struct TreeAtlas { pub w: i32, pub h: i32 }


