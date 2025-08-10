use glam::IVec2;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::types::TileKind;
use noise::NoiseFn;
use crate::world::World;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node { cost: i32, pos: IVec2 }

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // обратная для min-heap через BinaryHeap
        other.cost.cmp(&self.cost).then_with(|| self.pos.x.cmp(&other.pos.x))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

fn heuristic(a: IVec2, b: IVec2) -> i32 { (a.x - b.x).abs() + (a.y - b.y).abs() }

fn tile_cost(world: &World, p: IVec2) -> Option<i32> {
    let _t = world.fbm.get([p.x as f64, p.y as f64]) as f32; // touch NoiseFn trait
    // чтение без запроса новых чанков: если нет — считаем границу непроходимой
    let kind = world.chunks.get(&(p.x.div_euclid(crate::world::CHUNK_W), p.y.div_euclid(crate::world::CHUNK_H)))
        .map(|ch| {
            let lx = p.x.rem_euclid(crate::world::CHUNK_W);
            let ly = p.y.rem_euclid(crate::world::CHUNK_H);
            ch.tiles[(ly * crate::world::CHUNK_W + lx) as usize]
        })
        .unwrap_or(TileKind::Water);
    match kind {
        TileKind::Water => None,
        _ => {
            // высокая разница стоимостей: дорога=1, трава=4, лес=7
            let base = match kind { TileKind::Grass => 4, TileKind::Forest => 7, TileKind::Water => 999 };
            let cost = if world.is_road(p) { 1 } else { base };
            Some(cost)
        }
    }
}

pub fn astar(world: &World, start: IVec2, goal: IVec2, max_expansions: usize) -> Option<Vec<IVec2>> {
    if start == goal { return Some(vec![start]); }
    let mut open = BinaryHeap::new();
    open.push(Node { cost: 0, pos: start });
    let mut came_from: HashMap<(i32,i32), IVec2> = HashMap::new();
    let mut gscore: HashMap<(i32,i32), i32> = HashMap::new();
    gscore.insert((start.x,start.y), 0);
    let mut closed: HashSet<(i32,i32)> = HashSet::new();
    let mut expansions = 0;
    while let Some(Node { cost: _, pos }) = open.pop() {
        if pos == goal { break; }
        if !closed.insert((pos.x,pos.y)) { continue; }
        expansions += 1; if expansions > max_expansions { break; }
        const NB: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
        for (dx,dy) in NB {
            let np = IVec2::new(pos.x + dx, pos.y + dy);
            if let Some(step_cost) = tile_cost(world, np) {
                let tentative = gscore.get(&(pos.x,pos.y)).copied().unwrap_or(i32::MAX/4) + step_cost;
                if tentative < gscore.get(&(np.x,np.y)).copied().unwrap_or(i32::MAX/4) {
                    came_from.insert((np.x,np.y), pos);
                    gscore.insert((np.x,np.y), tentative);
                    let f = tentative + heuristic(np, goal);
                    open.push(Node { cost: f, pos: np });
                }
            }
        }
    }
    if !came_from.contains_key(&(goal.x, goal.y)) { return None; }
    // восстановление пути
    let mut path = vec![goal];
    let mut cur = goal;
    while cur != start {
        if let Some(&prev) = came_from.get(&(cur.x, cur.y)) { path.push(prev); cur = prev; } else { break; }
    }
    path.reverse();
    Some(path)
}


