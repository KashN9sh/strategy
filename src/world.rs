use std::{collections::{HashMap, HashSet}, sync::mpsc::{Sender, Receiver, channel}, thread};
use glam::IVec2;
use noise::{Fbm, NoiseFn, Seedable, MultiFractal};

use crate::types::TileKind;

pub const CHUNK_W: i32 = 32;
pub const CHUNK_H: i32 = 32;

#[derive(Clone)]
pub struct Chunk { pub tiles: Vec<TileKind> }

pub struct World {
    pub seed: u64,
    pub fbm: Fbm<noise::OpenSimplex>,
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub occupied: HashSet<(i32, i32)>,
    pub roads: HashSet<(i32, i32)>,
    pub tx: Sender<(i32, i32)>,
    pub rx: Receiver<ChunkResult>,
    pub pending: HashSet<(i32, i32)>,
    pub max_chunks: usize,
}

impl World {
    pub fn new(seed: u64) -> Self {
        let mut fbm = Fbm::<noise::OpenSimplex>::new(0);
        fbm = fbm.set_seed(seed as u32).set_octaves(5).set_frequency(0.03).set_lacunarity(2.0).set_persistence(0.5);
        let (tx, rx) = spawn_chunk_worker(seed);
        Self { seed, fbm, chunks: HashMap::new(), occupied: HashSet::new(), roads: HashSet::new(), tx, rx, pending: HashSet::new(), max_chunks: 512 }
    }

    pub fn reset_noise(&mut self, seed: u64) {
        self.seed = seed;
        let mut fbm = Fbm::<noise::OpenSimplex>::new(0);
        fbm = fbm.set_seed(seed as u32).set_octaves(5).set_frequency(0.03).set_lacunarity(2.0).set_persistence(0.5);
        self.fbm = fbm;
        self.chunks.clear();
        self.occupied.clear();
        self.roads.clear();
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
        }
        if self.chunks.len() > self.max_chunks {
            while self.chunks.len() > self.max_chunks {
                if let Some((&key, _)) = self.chunks.iter().next() { self.chunks.remove(&key); } else { break; }
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
        if n < -0.2 { TileKind::Water } else if n < 0.2 { TileKind::Grass } else { TileKind::Forest }
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


