use std::{collections::{HashMap, HashSet}, sync::mpsc::{Sender, Receiver, channel}, thread};
use glam::IVec2;
use noise::{Fbm, NoiseFn, Seedable, MultiFractal};

use crate::types::{TileKind, BiomeKind};

pub const CHUNK_W: i32 = 32;
pub const CHUNK_H: i32 = 32;

#[derive(Clone)]
pub struct Chunk { pub tiles: Vec<TileKind> }

#[derive(Clone, Copy, Debug)]
pub struct Tree { pub stage: u8, pub age_ms: i32 }

pub struct World {
    pub seed: u64,
    pub fbm: Fbm<noise::OpenSimplex>,
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub occupied: HashSet<(i32, i32)>,
    pub roads: HashSet<(i32, i32)>,
    pub trees: HashMap<(i32, i32), Tree>,
    pub tx: Sender<(i32, i32)>,
    pub rx: Receiver<ChunkResult>,
    pub pending: HashSet<(i32, i32)>,
    pub max_chunks: usize,
    // деревья, которые были вырублены (не восстанавливать при догрузке чанков)
    pub removed_trees: HashSet<(i32, i32)>,
    // кэш месторождений по тайлам (уменьшаем вызовы шума в рендер-цикле)
    pub clay_deposits: HashSet<(i32, i32)>,
    pub stone_deposits: HashSet<(i32, i32)>,
    pub iron_deposits: HashSet<(i32, i32)>,
    pub rivers: HashSet<(i32, i32)>,
    // биом по тайлам (кэш)
    pub biomes: HashMap<(i32,i32), BiomeKind>,
    // настройки биомов из конфига (runtime)
    pub biome_swamp_thr: f32,
    pub biome_rocky_thr: f32,
    pub biome_swamp_tree_growth_wmul: f32,
    pub biome_rocky_tree_growth_wmul: f32,
    // область строительства (разблокированные тайлы)
    pub explored_tiles: HashSet<(i32, i32)>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let mut fbm = Fbm::<noise::OpenSimplex>::new(0);
        fbm = fbm.set_seed(seed as u32).set_octaves(5).set_frequency(0.03).set_lacunarity(2.0).set_persistence(0.5);
        let (tx, rx) = spawn_chunk_worker(seed);
        Self { seed, fbm, chunks: HashMap::new(), occupied: HashSet::new(), roads: HashSet::new(), trees: HashMap::new(), tx, rx, pending: HashSet::new(), max_chunks: 512, removed_trees: HashSet::new(), clay_deposits: HashSet::new(), stone_deposits: HashSet::new(), iron_deposits: HashSet::new(), rivers: HashSet::new(), biomes: HashMap::new(), biome_swamp_thr: 0.10, biome_rocky_thr: 0.10, biome_swamp_tree_growth_wmul: 0.85, biome_rocky_tree_growth_wmul: 1.20, explored_tiles: HashSet::new() }
    }

    pub fn reset_noise(&mut self, seed: u64) {
        self.seed = seed;
        let mut fbm = Fbm::<noise::OpenSimplex>::new(0);
        fbm = fbm.set_seed(seed as u32).set_octaves(5).set_frequency(0.03).set_lacunarity(2.0).set_persistence(0.5);
        self.fbm = fbm;
        self.chunks.clear();
        self.occupied.clear();
        self.roads.clear();
        self.trees.clear();
        self.removed_trees.clear();
        self.clay_deposits.clear();
        self.stone_deposits.clear();
        self.iron_deposits.clear();
        self.rivers.clear();
        self.biomes.clear();
        self.explored_tiles.clear();
        let (tx, rx) = spawn_chunk_worker(seed);
        self.tx = tx; self.rx = rx; self.pending.clear();
    }

    pub fn get_tile(&mut self, tx_t: i32, ty_t: i32) -> TileKind {
        let cx = tx_t.div_euclid(CHUNK_W);
        let cy = ty_t.div_euclid(CHUNK_H);
        let lx = tx_t.rem_euclid(CHUNK_W);
        let ly = ty_t.rem_euclid(CHUNK_H);
        if !self.chunks.contains_key(&(cx, cy)) && !self.pending.contains(&(cx, cy)) {
            let _ = self.tx.send((cx, cy));
            self.pending.insert((cx, cy));
        }
        if let Some(chunk) = self.chunks.get(&(cx, cy)) {
            return chunk.tiles[(ly * CHUNK_W + lx) as usize];
        }
        self.tile_by_noise(tx_t, ty_t)
    }

    pub fn is_occupied(&self, t: IVec2) -> bool { self.occupied.contains(&(t.x, t.y)) }
    pub fn occupy(&mut self, t: IVec2) { self.occupied.insert((t.x, t.y)); }
    pub fn set_road(&mut self, t: IVec2, on: bool) { if on { self.roads.insert((t.x, t.y)); } else { self.roads.remove(&(t.x, t.y)); } }
    pub fn is_road(&self, t: IVec2) -> bool { self.roads.contains(&(t.x, t.y)) }

    pub fn integrate_ready_chunks(&mut self) {
        for res in self.rx.try_iter() {
            self.chunks.insert((res.cx, res.cy), Chunk { tiles: res.tiles });
            self.pending.remove(&(res.cx, res.cy));
            // Заполним набор деревьев по лесным тайлам в чанке
            if let Some(chunk) = self.chunks.get_mut(&(res.cx, res.cy)) {
                for ly in 0..CHUNK_H { for lx in 0..CHUNK_W {
                    let idx = (ly * CHUNK_W + lx) as usize;
                    let tx = res.cx * CHUNK_W + lx;
                    let ty = res.cy * CHUNK_H + ly;
                    let p = IVec2::new(tx, ty);
                    // Преобразуем «речные» клетки в воду на этапе интеграции (без доп. заимствования self)
                    // Локальные параметры/шумы
                    let fbm = &self.fbm;
                    let is_river = {
                        // маска 1
                        let v1 = fbm.get([p.x as f64 * 0.035 + 1234.0, p.y as f64 * 0.035 - 987.0]) as f32;
                        let u1 = fbm.get([p.x as f64 * 0.018 - 777.0, p.y as f64 * 0.022 + 444.0]) as f32;
                        let w1 = fbm.get([p.x as f64 * 0.10 - 222.0, p.y as f64 * 0.10 + 333.0]) as f32;
                        let s1 = ((v1 + u1 * 0.5) * 3.14159).sin().abs();
                        let width1 = 0.028 + 0.018 * (w1 * 0.5 + 0.5); // 0.028..0.046
                        // маска 2 (смещённые фазы) — добавляет дополнительных рек, почти не увеличивая ширину
                        let v2 = fbm.get([p.x as f64 * 0.032 - 321.0, p.y as f64 * 0.032 + 654.0]) as f32;
                        let u2 = fbm.get([p.x as f64 * 0.020 + 999.0, p.y as f64 * 0.017 - 888.0]) as f32;
                        let w2 = fbm.get([p.x as f64 * 0.12 + 111.0, p.y as f64 * 0.12 - 222.0]) as f32;
                        let s2 = ((v2 + u2 * 0.5) * 3.14159).sin().abs();
                        let width2 = 0.026 + 0.016 * (w2 * 0.5 + 0.5); // 0.026..0.042
                        (s1 < width1) || (s2 < width2)
                    };
                    if chunk.tiles[idx] != TileKind::Water && is_river {
                        chunk.tiles[idx] = TileKind::Water;
                        self.rivers.insert((tx, ty));
                    }
                    if chunk.tiles[idx] == TileKind::Forest {
                        if !self.removed_trees.contains(&(tx, ty)) {
                            // Детерминированное распределение зрелости по координатам и seed:
                            // ~15% stage0, ~30% stage1, ~55% stage2
                            let mut v = (tx as i64).wrapping_mul(0x9E3779B97F4A7C15u64 as i64)
                                ^ (ty as i64).wrapping_mul(0xC2B2AE3D27D4EB4Fu64 as i64)
                                ^ (self.seed as i64);
                            // xorshift64*
                            v ^= v >> 12; v ^= v << 25; v ^= v >> 27;
                            let u = ((v.wrapping_mul(0x2545F4914F6CDD1D) >> 11) & 0xFFFF_FFFF) as u32;
                            let r = (u as f32) / (u32::MAX as f32); // 0..1
                            let stage = if r < 0.15 { 0 } else if r < 0.45 { 1 } else { 2 };
                            self.trees.insert((tx, ty), Tree { stage, age_ms: 0 });
                        }
                    }
                    // кэш: месторождений и биомы (на неводных)
                    if chunk.tiles[idx] != TileKind::Water {
                        // Локальный расчёт биома и месторождений для избежания конфликта заимствований
                        let fbm = &self.fbm;
                        let swamp_thr = self.biome_swamp_thr;
                        let rocky_thr = self.biome_rocky_thr;
                        let moisture = fbm.get([tx as f64 * 0.18 + 311.0, ty as f64 * 0.18 - 211.0]) as f32;
                        let rocky_v  = fbm.get([tx as f64 * 0.22 - 157.0, ty as f64 * 0.22 +  97.0]) as f32;
                        let bm = if moisture > swamp_thr { BiomeKind::Swamp } else if rocky_v > rocky_thr { BiomeKind::Rocky } else { BiomeKind::Meadow };
                        // базовые вероятности
                        let has_clay_base = {
                            let base = fbm.get([tx as f64 * 0.30 + 31.0, ty as f64 * 0.30 - 77.0]) as f32;
                            let thr = 0.22_f32; let margin = 0.05_f32;
                            if base > thr { true } else if base > thr - margin {
                                const NB: [(i32,i32);8] = [(1,0),(-1,0),(0,1),(0,-1),(1,1),(1,-1),(-1,1),(-1,-1)];
                                let mut hits = 0; for (dx,dy) in NB { let v = fbm.get([(tx+dx) as f64 * 0.30 + 31.0, (ty+dy) as f64 * 0.30 - 77.0]) as f32; if v > thr { hits += 1; } }
                                hits >= 2
                            } else { false }
                        };
                        let has_stone_base = {
                            let base = fbm.get([tx as f64 * 0.38 - 123.0, ty as f64 * 0.38 + 19.0]) as f32;
                            let thr = 0.26_f32; let margin = 0.06_f32;
                            if base > thr { true } else if base > thr - margin {
                                const NB: [(i32,i32);8] = [(1,0),(-1,0),(0,1),(0,-1),(1,1),(1,-1),(-1,1),(-1,-1)];
                                let mut hits = 0; for (dx,dy) in NB { let v = fbm.get([(tx+dx) as f64 * 0.38 - 123.0, (ty+dy) as f64 * 0.38 + 19.0]) as f32; if v > thr { hits += 1; } }
                                hits >= 2
                            } else { false }
                        };
                        let has_iron_base = {
                            let base = fbm.get([tx as f64 * 0.34 + 211.0, ty as f64 * 0.34 + 87.0]) as f32;
                            let thr = 0.30_f32; let margin = 0.05_f32;
                            if base > thr { true } else if base > thr - margin {
                                const NB: [(i32,i32);8] = [(1,0),(-1,0),(0,1),(0,-1),(1,1),(1,-1),(-1,1),(-1,-1)];
                                let mut hits = 0; for (dx,dy) in NB { let v = fbm.get([(tx+dx) as f64 * 0.34 + 211.0, (ty+dy) as f64 * 0.34 + 87.0]) as f32; if v > thr { hits += 1; } }
                                hits >= 2
                            } else { false }
                        };
                        // биомные довески
                        let mut has_clay = has_clay_base;
                        if !has_clay && matches!(bm, BiomeKind::Swamp) {
                            let n = fbm.get([tx as f64 * 0.55 - 13.0, ty as f64 * 0.55 + 21.0]) as f32; if n > 0.60 { has_clay = true; }
                        }
                        let mut has_stone = has_stone_base;
                        if !has_stone && matches!(bm, BiomeKind::Rocky) {
                            let n = fbm.get([tx as f64 * 0.52 + 77.0, ty as f64 * 0.52 - 41.0]) as f32; if n > 0.62 { has_stone = true; }
                        }
                        let mut has_iron = has_iron_base;
                        if !has_iron && matches!(bm, BiomeKind::Rocky) {
                            let n = fbm.get([tx as f64 * 0.59 - 91.0, ty as f64 * 0.59 + 63.0]) as f32; if n > 0.64 { has_iron = true; }
                        }
                        if has_clay { self.clay_deposits.insert((tx, ty)); }
                        if has_stone { self.stone_deposits.insert((tx, ty)); }
                        if has_iron { self.iron_deposits.insert((tx, ty)); }
                        self.biomes.insert((tx, ty), bm);
                    }
                }}
            }
        }
        if self.chunks.len() > self.max_chunks {
            while self.chunks.len() > self.max_chunks {
                if let Some((&key, _)) = self.chunks.iter().next() {
                    // Удалим деревья, принадлежащие этому чанку
                    let (cx, cy) = key;
                    let min_tx = cx * CHUNK_W; let max_tx = min_tx + CHUNK_W - 1;
                    let min_ty = cy * CHUNK_H; let max_ty = min_ty + CHUNK_H - 1;
                    self.trees.retain(|&(tx, ty), _| !(tx >= min_tx && tx <= max_tx && ty >= min_ty && ty <= max_ty));
                    // очистим кэш месторождений в пределах чанка
                    self.clay_deposits.retain(|&(tx, ty)| !(tx >= min_tx && tx <= max_tx && ty >= min_ty && ty <= max_ty));
                    self.stone_deposits.retain(|&(tx, ty)| !(tx >= min_tx && tx <= max_tx && ty >= min_ty && ty <= max_ty));
                    self.iron_deposits.retain(|&(tx, ty)| !(tx >= min_tx && tx <= max_tx && ty >= min_ty && ty <= max_ty));
                    self.rivers.retain(|&(tx, ty)| !(tx >= min_tx && tx <= max_tx && ty >= min_ty && ty <= max_ty));
                    self.chunks.remove(&key);
                } else { break; }
            }
        }
    }

    pub fn schedule_ring(&mut self, min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32) {
        let cmin_x = (min_tx.div_euclid(CHUNK_W)) - 1;
        let cmin_y = (min_ty.div_euclid(CHUNK_H)) - 1;
        let cmax_x = (max_tx.div_euclid(CHUNK_W)) + 1;
        let cmax_y = (max_ty.div_euclid(CHUNK_H)) + 1;
        for cy in cmin_y..=cmax_y { for cx in cmin_x..=cmax_x {
            if !self.chunks.contains_key(&(cx, cy)) && !self.pending.contains(&(cx, cy)) {
                let _ = self.tx.send((cx, cy)); self.pending.insert((cx, cy));
            }
        }}
    }

    fn tile_by_noise(&self, tx: i32, ty: i32) -> TileKind {
        let n = self.fbm.get([tx as f64, ty as f64]) as f32;
        // Дополнительные вариации ресурсов через смещённые октавы шума
        let n2 = self.fbm.get([tx as f64 * 0.7 + 100.0, ty as f64 * 0.7 - 50.0]) as f32;
        let _n3 = self.fbm.get([tx as f64 * 0.9 - 200.0, ty as f64 * 0.9 + 80.0]) as f32;
        if n < -0.2 { TileKind::Water }
        else if n > 0.6 && n2 > 0.5 { TileKind::Forest }
        else { TileKind::Grass }
    }

    pub fn has_clay_deposit(&self, p: IVec2) -> bool { self.clay_deposits.contains(&(p.x, p.y)) }

    pub fn has_stone_deposit(&self, p: IVec2) -> bool { self.stone_deposits.contains(&(p.x, p.y)) }

    pub fn has_iron_deposit(&self, p: IVec2) -> bool { self.iron_deposits.contains(&(p.x, p.y)) }

    pub fn biome(&self, p: IVec2) -> BiomeKind {
        self.biomes.get(&(p.x, p.y)).cloned().unwrap_or_else(|| self.compute_biome(p))
    }

    fn compute_biome(&self, p: IVec2) -> BiomeKind {
        // базовый шум для «влажности/болотистости» и «каменистости»
        let moisture = self.fbm.get([p.x as f64 * 0.18 + 311.0, p.y as f64 * 0.18 - 211.0]) as f32; // -1..1
        let rocky    = self.fbm.get([p.x as f64 * 0.22 - 157.0, p.y as f64 * 0.22 +  97.0]) as f32; // -1..1
        let swamp_thr = self.biome_swamp_thr;
        let rocky_thr = self.biome_rocky_thr;
        // простое разбиение: высокий moisture -> болото; высокий rocky -> скалы; иначе — луг
        if moisture > swamp_thr { BiomeKind::Swamp }
        else if rocky > rocky_thr { BiomeKind::Rocky }
        else { BiomeKind::Meadow }
    }

    pub fn apply_biome_config(&mut self, cfg: &crate::input::Config) {
        self.biome_swamp_thr = cfg.biome_swamp_thr;
        self.biome_rocky_thr = cfg.biome_rocky_thr;
        self.biome_swamp_tree_growth_wmul = cfg.biome_swamp_tree_growth_wmul;
        self.biome_rocky_tree_growth_wmul = cfg.biome_rocky_tree_growth_wmul;
    }

    // --- Деревья ---
    pub fn has_tree(&self, p: IVec2) -> bool { self.trees.contains_key(&(p.x, p.y)) }
    pub fn tree_stage(&self, p: IVec2) -> Option<u8> { self.trees.get(&(p.x, p.y)).map(|t| t.stage) }
    pub fn plant_tree(&mut self, p: IVec2) { self.trees.insert((p.x, p.y), Tree { stage: 0, age_ms: 0 }); self.removed_trees.remove(&(p.x, p.y)); }
    pub fn remove_tree(&mut self, p: IVec2) { self.trees.remove(&(p.x, p.y)); self.removed_trees.insert((p.x, p.y)); }

    pub fn grow_trees(&mut self, dt_ms: i32) {
        // простая модель роста: 0->1 за 20с, 1->2 за ещё 40с, скорректированные биомом
        // Swamp — быстрее деревья; Rocky — медленнее
        let swamp_thr = self.biome_swamp_thr;
        let rocky_thr = self.biome_rocky_thr;
        let fbm = &self.fbm;
        let swamp_tree_wmul = self.biome_swamp_tree_growth_wmul.max(0.01);
        let rocky_tree_wmul = self.biome_rocky_tree_growth_wmul.max(0.01);
        for (&(tx, ty), tr) in self.trees.iter_mut() {
            // локально вычислим биом по шумам без повторного заимствования self
            let moisture = fbm.get([tx as f64 * 0.18 + 311.0, ty as f64 * 0.18 - 211.0]) as f32;
            let rocky = fbm.get([tx as f64 * 0.22 - 157.0, ty as f64 * 0.22 +  97.0]) as f32;
            let wmul = if moisture > swamp_thr { swamp_tree_wmul } else if rocky > rocky_thr { rocky_tree_wmul } else { 1.0 };
            tr.age_ms += ((dt_ms as f32) / wmul) as i32;
            if tr.stage == 0 && tr.age_ms >= 20000 { tr.stage = 1; tr.age_ms = 0; }
            else if tr.stage == 1 && tr.age_ms >= 40000 { tr.stage = 2; tr.age_ms = 0; }
        }
    }

    // --- Область строительства (fog of war) ---
    /// Проверить, разблокирован ли тайл для строительства
    pub fn is_explored(&self, p: IVec2) -> bool {
        self.explored_tiles.contains(&(p.x, p.y))
    }

    /// Разблокировать тайл
    pub fn explore_tile(&mut self, p: IVec2) {
        self.explored_tiles.insert((p.x, p.y));
    }

    /// Разблокировать область вокруг точки (круг с радиусом)
    pub fn explore_area(&mut self, center: IVec2, radius: i32) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= radius * radius {
                    self.explored_tiles.insert((center.x + dx, center.y + dy));
                }
            }
        }
    }

    /// Обновить область строительства на основе населения
    /// Радиус увеличивается с ростом населения
    pub fn update_exploration_by_population(&mut self, buildings: &[crate::types::Building], population: i32) {
        // Начальный радиус: 8 тайлов
        // Каждые 5 жителей добавляют +1 к радиусу
        let base_radius = 8;
        let radius = base_radius + (population / 5).min(20); // максимум радиус 28
        
        // Разблокируем область вокруг каждого дома
        for building in buildings.iter() {
            if matches!(building.kind, crate::types::BuildingKind::House) {
                self.explore_area(building.pos, radius);
            }
        }
    }
}

pub struct ChunkResult { pub cx: i32, pub cy: i32, pub tiles: Vec<TileKind> }

pub fn spawn_chunk_worker(seed: u64) -> (Sender<(i32, i32)>, Receiver<ChunkResult>) {
    let (tx, rx_req) = channel::<(i32, i32)>();
    let (tx_res, rx_res) = channel::<ChunkResult>();
    thread::spawn(move || {
        let mut fbm = Fbm::<noise::OpenSimplex>::new(0);
        fbm = fbm.set_seed(seed as u32).set_octaves(5).set_frequency(0.03).set_lacunarity(2.0).set_persistence(0.5);
        while let Ok((cx, cy)) = rx_req.recv() {
            let mut tiles = vec![TileKind::Water; (CHUNK_W * CHUNK_H) as usize];
            for ly in 0..CHUNK_H { for lx in 0..CHUNK_W {
                let tx = cx * CHUNK_W + lx; let ty = cy * CHUNK_H + ly;
                let n = fbm.get([tx as f64, ty as f64]) as f32;
                tiles[(ly * CHUNK_W + lx) as usize] = if n < -0.2 { TileKind::Water } else if n < 0.2 { TileKind::Grass } else { TileKind::Forest };
            }}
            let _ = tx_res.send(ChunkResult { cx, cy, tiles });
        }
    });
    (tx, rx_res)
}


