use std::{collections::{HashMap, HashSet}, sync::mpsc::{Sender, Receiver, channel}, thread};
use glam::IVec2;
use noise::{Fbm, NoiseFn, Seedable, MultiFractal};

use crate::types::TileKind;

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
}

impl World {
    pub fn new(seed: u64) -> Self {
        let mut fbm = Fbm::<noise::OpenSimplex>::new(0);
        fbm = fbm.set_seed(seed as u32).set_octaves(5).set_frequency(0.03).set_lacunarity(2.0).set_persistence(0.5);
        let (tx, rx) = spawn_chunk_worker(seed);
        Self { seed, fbm, chunks: HashMap::new(), occupied: HashSet::new(), roads: HashSet::new(), trees: HashMap::new(), tx, rx, pending: HashSet::new(), max_chunks: 512, removed_trees: HashSet::new(), clay_deposits: HashSet::new(), stone_deposits: HashSet::new(), iron_deposits: HashSet::new() }
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
    pub fn set_road(&mut self, t: IVec2, on: bool) { if on { self.roads.insert((t.x+1, t.y+1)); } else { self.roads.remove(&(t.x+1, t.y+1)); } }
    pub fn is_road(&self, t: IVec2) -> bool { self.roads.contains(&(t.x, t.y)) }

    pub fn integrate_ready_chunks(&mut self) {
        for res in self.rx.try_iter() {
            self.chunks.insert((res.cx, res.cy), Chunk { tiles: res.tiles });
            self.pending.remove(&(res.cx, res.cy));
            // Заполним набор деревьев по лесным тайлам в чанке
            if let Some(chunk) = self.chunks.get(&(res.cx, res.cy)) {
                for ly in 0..CHUNK_H { for lx in 0..CHUNK_W {
                    let idx = (ly * CHUNK_W + lx) as usize;
                    if chunk.tiles[idx] == TileKind::Forest {
                        let tx = res.cx * CHUNK_W + lx;
                        let ty = res.cy * CHUNK_H + ly;
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
                    // заполним кэш месторождений (только для неводных клеток)
                    if chunk.tiles[idx] != TileKind::Water {
                        let tx = res.cx * CHUNK_W + lx;
                        let ty = res.cy * CHUNK_H + ly;
                        let p = IVec2::new(tx, ty);
                        if self.compute_has_clay_deposit(p) { self.clay_deposits.insert((tx, ty)); }
                        if self.compute_has_stone_deposit(p) { self.stone_deposits.insert((tx, ty)); }
                        if self.compute_has_iron_deposit(p) { self.iron_deposits.insert((tx, ty)); }
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

    // --- Приватные методы расчёта месторождений (для заполнения кэша при интеграции чанков) ---
    fn compute_has_clay_deposit(&self, p: IVec2) -> bool {
        let base = self.fbm.get([p.x as f64 * 0.30 + 31.0, p.y as f64 * 0.30 - 77.0]) as f32;
        let thr = 0.22_f32; let margin = 0.05_f32;
        if base > thr { return true; }
        if base > thr - margin {
            const NB: [(i32,i32);8] = [(1,0),(-1,0),(0,1),(0,-1),(1,1),(1,-1),(-1,1),(-1,-1)];
            let mut hits = 0;
            for (dx,dy) in NB { let v = self.fbm.get([(p.x+dx) as f64 * 0.30 + 31.0, (p.y+dy) as f64 * 0.30 - 77.0]) as f32; if v > thr { hits += 1; } }
            return hits >= 2;
        }
        false
    }

    fn compute_has_stone_deposit(&self, p: IVec2) -> bool {
        let base = self.fbm.get([p.x as f64 * 0.38 - 123.0, p.y as f64 * 0.38 + 19.0]) as f32;
        let thr = 0.26_f32; let margin = 0.06_f32;
        if base > thr { return true; }
        if base > thr - margin {
            const NB: [(i32,i32);8] = [(1,0),(-1,0),(0,1),(0,-1),(1,1),(1,-1),(-1,1),(-1,-1)];
            let mut hits = 0;
            for (dx,dy) in NB { let v = self.fbm.get([(p.x+dx) as f64 * 0.38 - 123.0, (p.y+dy) as f64 * 0.38 + 19.0]) as f32; if v > thr { hits += 1; } }
            return hits >= 2;
        }
        false
    }

    fn compute_has_iron_deposit(&self, p: IVec2) -> bool {
        let base = self.fbm.get([p.x as f64 * 0.34 + 211.0, p.y as f64 * 0.34 + 87.0]) as f32;
        let thr = 0.30_f32; let margin = 0.05_f32;
        if base > thr { return true; }
        if base > thr - margin {
            const NB: [(i32,i32);8] = [(1,0),(-1,0),(0,1),(0,-1),(1,1),(1,-1),(-1,1),(-1,-1)];
            let mut hits = 0;
            for (dx,dy) in NB { let v = self.fbm.get([(p.x+dx) as f64 * 0.34 + 211.0, (p.y+dy) as f64 * 0.34 + 87.0]) as f32; if v > thr { hits += 1; } }
            return hits >= 2;
        }
        false
    }

    // --- Деревья ---
    pub fn has_tree(&self, p: IVec2) -> bool { self.trees.contains_key(&(p.x, p.y)) }
    pub fn tree_stage(&self, p: IVec2) -> Option<u8> { self.trees.get(&(p.x, p.y)).map(|t| t.stage) }
    pub fn plant_tree(&mut self, p: IVec2) { self.trees.insert((p.x, p.y), Tree { stage: 0, age_ms: 0 }); self.removed_trees.remove(&(p.x, p.y)); }
    pub fn remove_tree(&mut self, p: IVec2) { self.trees.remove(&(p.x, p.y)); self.removed_trees.insert((p.x, p.y)); }

    pub fn grow_trees(&mut self, dt_ms: i32) {
        // простая модель роста: 0->1 за 20с, 1->2 за ещё 40с
        for (_pos, tr) in self.trees.iter_mut() {
            tr.age_ms += dt_ms;
            if tr.stage == 0 && tr.age_ms >= 20000 { tr.stage = 1; tr.age_ms = 0; }
            else if tr.stage == 1 && tr.age_ms >= 40000 { tr.stage = 2; tr.age_ms = 0; }
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


