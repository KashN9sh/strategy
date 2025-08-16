use glam::IVec2;

use crate::atlas::{TileAtlas, RoadAtlas, building_sprite_index};
use noise::NoiseFn;
use crate::render::tiles;
use crate::types::{Building, BuildingKind};
use crate::palette::building_color;
use crate::world::World;
use crate::types::BiomeKind;

pub fn draw_terrain_and_overlays(
    frame: &mut [u8], width: i32, height: i32,
    atlas: &TileAtlas,
    world: &mut World,
    min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32,
    screen_center: IVec2, cam_snap: glam::Vec2,
    water_frame: usize,
    show_grid: bool,
    show_forest_overlay: bool,
    draw_trees_in_this_pass: bool,
    tree_atlas: &Option<crate::atlas::TreeAtlas>,
    road_atlas: &RoadAtlas,
) {
    for my in min_ty..=max_ty { for mx in min_tx..=max_tx {
        let kind = world.get_tile(mx, my);
        let screen_pos = world_to_screen(atlas, screen_center, cam_snap, mx, my);
        // Масштабируем исходный PNG-тайл под ширину ромба; высота может быть больше ромба,
        // поэтому сдвигаем верх так, чтобы низ PNG совпал с низом ромба (как было в main.rs)
        let tile_w_px = atlas.half_w * 2 + 1;
        let draw_w = tile_w_px;
        let scale = tile_w_px as f32 / atlas.base_w.max(1) as f32;
        let draw_h = (atlas.base_h as f32 * scale).round() as i32;
        let top_left_x = screen_pos.x - draw_w / 2;
        let top_left_y = screen_pos.y + atlas.half_h - draw_h; // нижний край PNG совпадает с нижней вершиной ромба
        if atlas.base_loaded {
            match kind {
                crate::types::TileKind::Grass => {
                    let idx = ((mx as i64 * 73856093 ^ my as i64 * 19349663) & 0x7fffffff) as usize;
                    if !atlas.grass_variants.is_empty() {
                        let spr = &atlas.grass_variants[idx % atlas.grass_variants.len()];
                        tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, spr, atlas.base_w, atlas.base_h, draw_w, draw_h);
                    } else {
                        tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, &atlas.base_grass, atlas.base_w, atlas.base_h, draw_w, draw_h);
                    }
                    // оттенок по биому (усилим, чтобы было явнее видно)
                    let biome = world.biome(IVec2::new(mx, my));
                    match biome {
                        BiomeKind::Swamp => tiles::blit_sprite_alpha_scaled_color_tint(frame, width, height, top_left_x, top_left_y, &atlas.base_grass, atlas.base_w, atlas.base_h, draw_w, draw_h, [50, 110, 70], 90, 255),
                        BiomeKind::Rocky => tiles::blit_sprite_alpha_scaled_color_tint(frame, width, height, top_left_x, top_left_y, &atlas.base_grass, atlas.base_w, atlas.base_h, draw_w, draw_h, [150, 150, 150], 90, 255),
                        _ => {}
                    }
                }
                crate::types::TileKind::Forest => {
                    tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, &atlas.base_forest, atlas.base_w, atlas.base_h, draw_w, draw_h);
                    // оттенок по биому и для леса, чтобы зоны читались
                    let biome = world.biome(IVec2::new(mx, my));
                    match biome {
                        BiomeKind::Swamp => tiles::blit_sprite_alpha_scaled_color_tint(frame, width, height, top_left_x, top_left_y, &atlas.base_forest, atlas.base_w, atlas.base_h, draw_w, draw_h, [50, 110, 70], 80, 255),
                        BiomeKind::Rocky => tiles::blit_sprite_alpha_scaled_color_tint(frame, width, height, top_left_x, top_left_y, &atlas.base_forest, atlas.base_w, atlas.base_h, draw_w, draw_h, [150, 150, 150], 80, 255),
                        _ => {}
                    }
                }
                crate::types::TileKind::Water => {
                    let land_n = world.get_tile(mx, my-1) != crate::types::TileKind::Water;
                    let land_e = world.get_tile(mx+1, my) != crate::types::TileKind::Water;
                    let land_s = world.get_tile(mx, my+1) != crate::types::TileKind::Water;
                    let land_w = world.get_tile(mx-1, my) != crate::types::TileKind::Water;
                    let land_ne = world.get_tile(mx+1, my-1) != crate::types::TileKind::Water;
                    let land_nw = world.get_tile(mx-1, my-1) != crate::types::TileKind::Water;
                    let land_se = world.get_tile(mx+1, my+1) != crate::types::TileKind::Water;
                    let land_sw = world.get_tile(mx-1, my+1) != crate::types::TileKind::Water;
                    if !atlas.water_edges.is_empty() && (land_n || land_e || land_s || land_w) {
                        let idx_opt: Option<usize> = if land_n && land_e && !land_ne { Some(7) }
                            else if land_n && land_w && !land_nw { Some(4) }
                            else if land_s && land_e && !land_se { Some(5) }
                            else if land_s && land_w && !land_sw { Some(6) }
                            else if land_n { Some(0) } else if land_w { Some(1) } else if land_e { Some(2) } else if land_s { Some(3) } else { None };
                        if let Some(mut idx) = idx_opt { if idx >= atlas.water_edges.len() { idx = atlas.water_edges.len()-1; }
                            let spr = &atlas.water_edges[idx]; tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, spr, atlas.base_w, atlas.base_h, draw_w, draw_h);
                        } else { tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, &atlas.base_water, atlas.base_w, atlas.base_h, draw_w, draw_h); }
                    } else { tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, &atlas.base_water, atlas.base_w, atlas.base_h, draw_w, draw_h); }
                }
                // Река поверх травы/леса — узкая голубая лента с небольшой прозрачностью
                _ => {}
            }
        } else { atlas.blit(frame, width, height, screen_pos.x, screen_pos.y, kind, water_frame); }
        if show_grid {
            // Если включён режим сетки через консоль для биомов — раскрасим сетку по биому
            let color = match world.biome(IVec2::new(mx, my)) {
                BiomeKind::Meadow => [30, 180, 30, 255],
                BiomeKind::Swamp  => [40, 120, 80, 255],
                BiomeKind::Rocky  => [160, 160, 160, 255],
            };
            tiles::draw_iso_outline(frame, width, height, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, color);
        }
        // дороги — рисуем отдельным проходом после цикла по тайлам (см. ниже)
        if show_forest_overlay { let n = world.fbm.get([mx as f64, my as f64]) as f32; let v = ((n + 1.0) * 0.5 * 255.0) as u8; tiles::draw_iso_outline(frame, width, height, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [v,50,50,255]); }

        // Оверлеи месторождений (поверх базового тайла)
        let tp = IVec2::new(mx, my);
        if kind != crate::types::TileKind::Water {
            if atlas.base_loaded {
                // Используем те же draw_w/draw_h/top_left_x/y, что и для базового PNG тайла
                if world.has_clay_deposit(tp) {
                    if !atlas.clay_variants.is_empty() {
                        let idx = ((mx as i64 * 83492791 ^ my as i64 * 29765723) & 0x7fffffff) as usize;
                        let spr = &atlas.clay_variants[idx % atlas.clay_variants.len()];
                        tiles::blit_sprite_alpha_scaled_color_tint(frame, width, height, top_left_x, top_left_y, spr, atlas.base_w, atlas.base_h, draw_w, draw_h, [170, 100, 80], 120, 230);
                    } else {
                        tiles::blit_sprite_alpha_scaled_color_tint(frame, width, height, top_left_x, top_left_y, &atlas.base_clay, atlas.base_w, atlas.base_h, draw_w, draw_h, [170, 100, 80], 120, 230);
                    }
                }
                if world.has_stone_deposit(tp) {
                    tiles::blit_sprite_alpha_scaled_tinted(frame, width, height, top_left_x, top_left_y, &atlas.base_stone, atlas.base_w, atlas.base_h, draw_w, draw_h, 220);
                }
                if world.has_iron_deposit(tp) {
                    tiles::blit_sprite_alpha_scaled_color_tint(frame, width, height, top_left_x, top_left_y, &atlas.base_iron, atlas.base_w, atlas.base_h, draw_w, draw_h, [200, 205, 220], 140, 240);
                }
            } else {
                if world.has_clay_deposit(tp) { tiles::draw_iso_tile_tinted(frame, width, height, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [200,120,80,120]); }
                if world.has_stone_deposit(tp) { tiles::draw_iso_tile_tinted(frame, width, height, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [200,200,200,120]); }
                if world.has_iron_deposit(tp) { tiles::draw_iso_tile_tinted(frame, width, height, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, [150,140,220,140]); }
            }
        }
    }}

    // Второй проход: дороги поверх базовых тайлов и оверлеев
    for my in min_ty..=max_ty { for mx in min_tx..=max_tx {
        if !world.is_road(IVec2::new(mx, my)) { continue; }
        let screen_pos = world_to_screen(atlas, screen_center, cam_snap, mx, my);
        let tile_w_px = atlas.half_w * 2 + 1;
        let draw_w = tile_w_px;
        let scale = tile_w_px as f32 / atlas.base_w.max(1) as f32;
        let draw_h = (atlas.base_h as f32 * scale).round() as i32;
        let hover_off = ((atlas.half_h as f32) * 0.5).round() as i32;
        if atlas.base_loaded && !atlas.base_road.is_empty() {
            let top_left_x = screen_pos.x - draw_w / 2;
            let top_left_y = screen_pos.y + atlas.half_h - draw_h + hover_off;
            tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, &atlas.base_road, atlas.base_w, atlas.base_h, draw_w, draw_h);
        } else {
            let nb = [ (0,-1,0b0001), (1,0,0b0010), (0,1,0b0100), (-1,0,0b1000) ];
            let mut mask: u8 = 0;
            for (dx,dy,bit) in nb { if world.is_road(IVec2::new(mx+dx, my+dy)) { mask |= bit; } }
            let spr = &road_atlas.sprites[mask as usize];
            let w = road_atlas.w; let h = road_atlas.h;
            let top_left_x = screen_pos.x - w / 2;
            let top_left_y = screen_pos.y + atlas.half_h - h + hover_off;
            tiles::blit_sprite_alpha_noscale_tinted(frame, width, height, top_left_x, top_left_y, spr, w, h, 255);
        }
    }}

    // Третий проход (опционально): деревья поверх дорог
    if draw_trees_in_this_pass {
        for my in min_ty..=max_ty { for mx in min_tx..=max_tx {
            if !world.has_tree(IVec2::new(mx, my)) { continue; }
            let screen_pos = world_to_screen(atlas, screen_center, cam_snap, mx, my);
            let stage = world.tree_stage(IVec2::new(mx, my)).unwrap_or(2) as usize;
            if let Some(ta) = tree_atlas { if !ta.sprites.is_empty() {
                let idx = stage.min(ta.sprites.len()-1);
                let tile_w_px = atlas.half_w * 2 + 1; let scale = tile_w_px as f32 / ta.w as f32; let draw_w = (ta.w as f32 * scale).round() as i32; let draw_h = (ta.h as f32 * scale).round() as i32;
                let top_left_x = screen_pos.x - draw_w / 2; let top_left_y = screen_pos.y - atlas.half_h - draw_h;
                tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, &ta.sprites[idx], ta.w, ta.h, draw_w, draw_h);
            } else {
                tiles::draw_tree(frame, width, height, screen_pos.x, screen_pos.y - atlas.half_h, atlas.half_w, atlas.half_h, stage as u8);
            }} else {
                tiles::draw_tree(frame, width, height, screen_pos.x, screen_pos.y - atlas.half_h, atlas.half_w, atlas.half_h, stage as u8);
            }
        }}
    }
}

pub fn draw_buildings(
    frame: &mut [u8], width: i32, height: i32,
    atlas: &TileAtlas,
    buildings: &Vec<Building>,
    building_atlas: &Option<crate::atlas::BuildingAtlas>,
    screen_center: IVec2, cam_snap: glam::Vec2,
    min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32,
) {
    for b in buildings.iter() {
        let mx = b.pos.x; let my = b.pos.y; if mx < min_tx || my < min_ty || mx > max_tx || my > max_ty { continue; }
        let screen_pos = world_to_screen(atlas, screen_center, cam_snap, mx, my);
        if let Some(ba) = building_atlas {
            if let Some(idx) = building_sprite_index(b.kind) {
                if idx < ba.sprites.len() {
                    let tile_w_px = atlas.half_w * 2 + 1; let scale = tile_w_px as f32 / ba.w as f32; let draw_w = (ba.w as f32 * scale).round() as i32; let draw_h = (ba.h as f32 * scale).round() as i32;
                    // Привязка здания: нижний центр спрайта должен совпадать с нижней вершиной ромба
                    // Доп. смещение вниз синхронно с подсветкой, чтобы визуальные базисы совпадали при разных масштабах
                    let building_off = ((atlas.half_h as f32) * 0.7).round() as i32; // ~30px при max zoom
                    let top_left_x = screen_pos.x - draw_w / 2; let top_left_y = screen_pos.y + atlas.half_h - draw_h + building_off;
                    tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, &ba.sprites[idx], ba.w, ba.h, draw_w, draw_h);
                    continue;
                }
            }
        }
        let color = building_color(b.kind);
        tiles::draw_building(frame, width, height, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, color);
    }
}

// индекс спрайта здания — см. atlas::building_sprite_index

/// Нарисовать одно дерево (спрайт или примитив) в координатах тайла
pub fn draw_tree_at(
    frame: &mut [u8], width: i32, height: i32,
    atlas: &TileAtlas,
    tree_atlas: &Option<crate::atlas::TreeAtlas>,
    screen_center: IVec2, cam_snap: glam::Vec2,
    mx: i32, my: i32, stage: usize,
) {
    let screen_pos = world_to_screen(atlas, screen_center, cam_snap, mx, my);
    if let Some(ta) = tree_atlas { if !ta.sprites.is_empty() {
        let idx = stage.min(ta.sprites.len()-1);
        let tile_w_px = atlas.half_w * 2 + 1; let scale = tile_w_px as f32 / ta.w as f32; let draw_w = (ta.w as f32 * scale).round() as i32; let draw_h = (ta.h as f32 * scale).round() as i32;
        let top_left_x = screen_pos.x - draw_w / 2; let top_left_y = screen_pos.y - atlas.half_h - draw_h;
        tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, &ta.sprites[idx], ta.w, ta.h, draw_w, draw_h);
    } else {
        tiles::draw_tree(frame, width, height, screen_pos.x, screen_pos.y - atlas.half_h, atlas.half_w, atlas.half_h, stage as u8);
    }} else {
        tiles::draw_tree(frame, width, height, screen_pos.x, screen_pos.y - atlas.half_h, atlas.half_w, atlas.half_h, stage as u8);
    }
}

/// Нарисовать одно здание
pub fn draw_building_at(
    frame: &mut [u8], width: i32, height: i32,
    atlas: &TileAtlas,
    b: &Building,
    building_atlas: &Option<crate::atlas::BuildingAtlas>,
    screen_center: IVec2, cam_snap: glam::Vec2,
) {
    let screen_pos = world_to_screen(atlas, screen_center, cam_snap, b.pos.x, b.pos.y);
    if let Some(ba) = building_atlas {
        if let Some(idx) = building_sprite_index(b.kind) {
            if idx < ba.sprites.len() {
                let tile_w_px = atlas.half_w * 2 + 1; let scale = tile_w_px as f32 / ba.w as f32; let draw_w = (ba.w as f32 * scale).round() as i32; let draw_h = (ba.h as f32 * scale).round() as i32;
                let building_off = ((atlas.half_h as f32) * 0.7).round() as i32;
                let top_left_x = screen_pos.x - draw_w / 2; let top_left_y = screen_pos.y + atlas.half_h - draw_h + building_off;
                tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, &ba.sprites[idx], ba.w, ba.h, draw_w, draw_h);
                return;
            }
        }
    }
    let color = building_color(b.kind);
    tiles::draw_building(frame, width, height, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, color);
}

/// Объединённый проход: деревья и здания, отсортированные по глубине (x+y)
pub fn draw_structures_sorted(
    frame: &mut [u8], width: i32, height: i32,
    atlas: &TileAtlas,
    world: &World,
    buildings: &Vec<Building>,
    building_atlas: &Option<crate::atlas::BuildingAtlas>,
    tree_atlas: &Option<crate::atlas::TreeAtlas>,
    screen_center: IVec2, cam_snap: glam::Vec2,
    min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32,
) {
    #[derive(Clone, Copy)]
    enum ItemKind { Building(usize), Tree { x: i32, y: i32, stage: usize } }
    let mut items: Vec<(i32, i32, i32, ItemKind)> = Vec::new();
    // здания
    for (i, b) in buildings.iter().enumerate() {
        let mx = b.pos.x; let my = b.pos.y;
        if mx < min_tx || my < min_ty || mx > max_tx || my > max_ty { continue; }
        // Классический порядок изометрии: по диагонали (x+y), затем по x (запад→восток)
        items.push((mx + my, mx, 1, ItemKind::Building(i)));
    }
    // деревья
    for my in min_ty..=max_ty { for mx in min_tx..=max_tx {
        if world.has_tree(IVec2::new(mx, my)) {
            let stage = world.tree_stage(IVec2::new(mx, my)).unwrap_or(2) as usize;
            items.push((mx + my, mx, 0, ItemKind::Tree { x: mx, y: my, stage }));
        }
    }}
    // Сортировка: (x+y) → x → pri (дерево до здания)
    items.sort_by_key(|(z, x, pri, _)| (*z, *x, *pri));
    for (_, _, _, it) in items {
        match it {
            ItemKind::Building(i) => {
                let b = &buildings[i];
                draw_building_at(frame, width, height, atlas, b, building_atlas, screen_center, cam_snap);
            }
            ItemKind::Tree { x, y, stage } => {
                draw_tree_at(frame, width, height, atlas, tree_atlas, screen_center, cam_snap, x, y, stage);
            }
        }
    }
}

/// Последовательный проход по тайлам (my→mx): на каждом тайле рисуем дерево и/или здание.
/// Это даёт корректный painter's order для высоких спрайтов без сложной сортировки.
pub fn draw_structures_scanned(
    frame: &mut [u8], width: i32, height: i32,
    atlas: &TileAtlas,
    world: &World,
    buildings: &Vec<Building>,
    building_atlas: &Option<crate::atlas::BuildingAtlas>,
    tree_atlas: &Option<crate::atlas::TreeAtlas>,
    screen_center: IVec2, cam_snap: glam::Vec2,
    min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32,
) {
    use std::collections::HashMap;
    let mut by_pos: HashMap<(i32,i32), usize> = HashMap::new();
    for (i, b) in buildings.iter().enumerate() {
        by_pos.insert((b.pos.x, b.pos.y), i);
    }
    for my in min_ty..=max_ty { for mx in min_tx..=max_tx {
        // дерево на этой клетке
        if world.has_tree(IVec2::new(mx, my)) {
            let stage = world.tree_stage(IVec2::new(mx, my)).unwrap_or(2) as usize;
            draw_tree_at(frame, width, height, atlas, tree_atlas, screen_center, cam_snap, mx, my, stage);
        }
        // здание на этой клетке
        if let Some(&bi) = by_pos.get(&(mx, my)) {
            let b = &buildings[bi];
            draw_building_at(frame, width, height, atlas, b, building_atlas, screen_center, cam_snap);
        }
    }}
}

/// Диагональный проход (по сумме координат x+y): корректный порядок для изометрии.
pub fn draw_structures_diagonal_scan(
    frame: &mut [u8], width: i32, height: i32,
    atlas: &TileAtlas,
    world: &World,
    buildings: &Vec<Building>,
    building_atlas: &Option<crate::atlas::BuildingAtlas>,
    tree_atlas: &Option<crate::atlas::TreeAtlas>,
    screen_center: IVec2, cam_snap: glam::Vec2,
    min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32,
) {
    use std::collections::HashMap;
    let mut by_pos: HashMap<(i32,i32), usize> = HashMap::new();
    for (i, b) in buildings.iter().enumerate() {
        by_pos.insert((b.pos.x, b.pos.y), i);
    }
    let min_s = min_tx + min_ty;
    let max_s = max_tx + max_ty;
    for s in min_s..=max_s {
        for mx in min_tx..=max_tx {
            let my = s - mx;
            if my < min_ty || my > max_ty { continue; }
            // дерево: рисуем раньше, чтобы здание при необходимости его перекрывало на этой же диагонали
            if world.has_tree(IVec2::new(mx, my)) {
                let stage = world.tree_stage(IVec2::new(mx, my)).unwrap_or(2) as usize;
                draw_tree_at(frame, width, height, atlas, tree_atlas, screen_center, cam_snap, mx, my, stage);
            }
            if let Some(&bi) = by_pos.get(&(mx, my)) {
                let b = &buildings[bi];
                draw_building_at(frame, width, height, atlas, b, building_atlas, screen_center, cam_snap);
            }
        }
    }
}

pub fn draw_citizens(
    frame: &mut [u8], width: i32, height: i32,
    atlas: &TileAtlas,
    citizens: &Vec<crate::types::Citizen>,
    buildings: &Vec<Building>,
    screen_center: IVec2, cam_snap: glam::Vec2,
    citizen_sprites: &Option<(Vec<Vec<u8>>, i32, i32)>,
    face_sprites: &Option<(Vec<Vec<u8>>, i32, i32)>,
) {
    for c in citizens.iter() {
        let (fx, fy) = if c.moving { let dx = (c.target.x - c.pos.x) as f32; let dy = (c.target.y - c.pos.y) as f32; (c.pos.x as f32 + dx * c.progress, c.pos.y as f32 + dy * c.progress) } else { (c.pos.x as f32, c.pos.y as f32) };
        let sx = ((fx - fy) * atlas.half_w as f32).round() as i32; let sy = ((fx + fy) * atlas.half_h as f32).round() as i32;
        let screen_pos = screen_center + IVec2::new(sx - cam_snap.x as i32, sy - cam_snap.y as i32);
        let r = (atlas.half_w as f32 * 0.24).round() as i32; // увеличенный размер маркера
        let mut col = [255, 230, 120, 255];
        if let Some(wp) = c.workplace { if let Some(b) = buildings.iter().find(|b| b.pos == wp) { col = building_color(b.kind); } }
        let base_y = screen_pos.y - atlas.half_h/3;
        let rr = r.max(2);
        if let Some((frames, sw, sh)) = citizen_sprites {
            // Анимация: 0 — стоит, 1.. — 2-4 кадра ходьбы
            let num_frames = frames.len().max(1);
            let anim_idx = if c.moving {
                // простой счётчик по прогрессу и пути
                let stride = ((c.progress * 8.0) as usize) % num_frames.max(1);
                if stride == 0 { 1.min(num_frames-1) } else { stride.min(num_frames-1) }
            } else { 0 };
            let spr = &frames[anim_idx.min(frames.len()-1)];
            let draw_w = rr * 2 + 1; let draw_h = rr * 2 + 1;
            let top_left_x = screen_pos.x - draw_w / 2; let top_left_y = base_y - draw_h / 2;
            tiles::blit_sprite_alpha_scaled_color_tint(
                frame, width, height,
                top_left_x, top_left_y,
                spr, *sw, *sh, draw_w, draw_h,
                [col[0], col[1], col[2]], 180, 255,
            );
        } else {
            tiles::draw_citizen_marker(frame, width, height, screen_pos.x, base_y, rr, col);
        }
        // мини-эмоция поверх маркера: используем спрайт если есть
        // mood: 0 = sad, 1 = neutral, 2 = happy (внутреннее значение)
        // порядок в ассете: neutral (0), happy (1), sad (2)
        let mood = if c.happiness as i32 >= 66 { 2 } else if c.happiness as i32 <= 33 { 0 } else { 1 };
        if let Some((sprites, sw, sh)) = face_sprites {
            // маппинг: internal -> asset index: neutral(1)->0, happy(2)->1, sad(0)->2
            let idx = match mood { 1 => 0, 2 => 1, _ => 2 };
            // выбор ряда по яркости основы: если фон светлый — тёмный ряд (ry=1), иначе светлый (ry=0)
            let ry = {
                // проба пикселя маркера в центре
                let col = [col[0], col[1], col[2]]; // цвет маркера; approx luminance
                let lum = (col[0] as u16 * 77 + col[1] as u16 * 150 + col[2] as u16 * 29) >> 8;
                if *sh > 0 && sprites.len() >= 6 && lum > 140 { 1 } else { 0 }
            } as usize;
            let sprite_idx = ry * 3 + idx;
            if sprite_idx < sprites.len() {
                let face = &sprites[sprite_idx];
                // позиционируем лицо внутри круга; делаем крупнее для читаемости
                let scale = ((rr as f32) * 1.7).round() as i32;
                let draw_w = scale; let draw_h = scale;
                let top_left_x = screen_pos.x - draw_w / 2; let top_left_y = base_y - draw_h / 2;
                tiles::blit_sprite_alpha_scaled(frame, width, height, top_left_x, top_left_y, face, *sw, *sh, draw_w, draw_h);
            }
        } else {
            tiles::draw_emote_on_marker(frame, width, height, screen_pos.x, base_y, rr, mood);
        }
    }
}

pub fn draw_debug_path(frame: &mut [u8], width: i32, height: i32, atlas: &TileAtlas, path: &Vec<IVec2>, screen_center: IVec2, cam_snap: glam::Vec2) {
    for p in path.iter() {
        let sp = world_to_screen(atlas, screen_center, cam_snap, p.x, p.y);
        tiles::draw_iso_outline(frame, width, height, sp.x, sp.y, atlas.half_w, atlas.half_h, [50, 200, 240, 255]);
    }
}

/// Нарисовать «призрак» здания на тайле `tp` с цветом в зависимости от допустимости `allowed`.
pub fn draw_building_ghost(
    frame: &mut [u8], width: i32, height: i32,
    atlas: &TileAtlas,
    building_kind: BuildingKind,
    tp: IVec2,
    allowed: bool,
    screen_center: IVec2, cam_snap: glam::Vec2,
    building_atlas: &Option<crate::atlas::BuildingAtlas>,
)
{
    let screen_pos = world_to_screen(atlas, screen_center, cam_snap, tp.x, tp.y);
    // зелёный/красный оттенок
    let fill = if allowed { [120, 200, 120, 80] } else { [220, 100, 100, 80] };
    let line = if allowed { [120, 220, 120, 240] } else { [240, 140, 140, 240] };
    let hover_off = ((atlas.half_h as f32) * 0.5).round() as i32;
    // Подсветим ромб клетки под зданием
    tiles::draw_iso_tile_tinted(frame, width, height, screen_pos.x, screen_pos.y + hover_off, atlas.half_w, atlas.half_h, fill);
    tiles::draw_iso_outline(frame, width, height, screen_pos.x, screen_pos.y + hover_off, atlas.half_w, atlas.half_h, line);
    // Попробуем полупрозрачно показать спрайт здания, если есть
    if let Some(ba) = building_atlas {
        if let Some(idx) = crate::atlas::building_sprite_index(building_kind) {
            if idx < ba.sprites.len() {
                let tile_w_px = atlas.half_w * 2 + 1;
                let scale = tile_w_px as f32 / ba.w as f32;
                let draw_w = (ba.w as f32 * scale).round() as i32;
                let draw_h = (ba.h as f32 * scale).round() as i32;
                let building_off = ((atlas.half_h as f32) * 0.7).round() as i32;
                let top_left_x = screen_pos.x - draw_w / 2;
                let top_left_y = screen_pos.y + atlas.half_h - draw_h + building_off;
                // Тонируем спрайт под allowed/denied
                let tint_rgb = if allowed { [160, 240, 160] } else { [255, 120, 120] };
                let tint_a: u8 = 160; // полупрозрачный
                tiles::blit_sprite_alpha_scaled_color_tint(frame, width, height, top_left_x, top_left_y, &ba.sprites[idx], ba.w, ba.h, draw_w, draw_h, tint_rgb, tint_a, 255);
                return;
            }
        }
    }
    // Фоллбек: примитив здания с тоном
    let mut color = building_color(building_kind);
    if allowed { color = [color[0], (color[1] as u16 + 30).min(255) as u8, color[2], 255]; }
    tiles::draw_building(frame, width, height, screen_pos.x, screen_pos.y, atlas.half_w, atlas.half_h, color);
}

pub fn world_to_screen(atlas: &TileAtlas, screen_center: IVec2, cam_snap: glam::Vec2, mx: i32, my: i32) -> IVec2 {
    // Эквивалент обратной проекции: screen_x = (x - y) * half_w, screen_y = (x + y) * half_h
    let sx = (mx - my) * atlas.half_w - cam_snap.x as i32;
    let sy = (mx + my) * atlas.half_h - cam_snap.y as i32;
    screen_center + IVec2::new(sx, sy)
}

/// Простая мини-карта: даунскейл тайлов, дороги/здания поверх и рамка текущей камеры
pub fn draw_minimap(
    frame: &mut [u8], fw: i32, fh: i32,
    world: &mut World,
    buildings: &Vec<Building>,
    min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32, // область, которую рисуем на мини-карте
    x: i32, y: i32, cell_px: i32,                         // левый верх и размер ячейки в пикселях
    cam_min_tx: i32, cam_min_ty: i32, cam_max_tx: i32, cam_max_ty: i32, // текущие видимые границы для маркера камеры
) {
    let w_tiles = (max_tx - min_tx + 1).max(1);
    let h_tiles = (max_ty - min_ty + 1).max(1);
    let map_w = w_tiles * cell_px;
    let map_h = h_tiles * cell_px;
    // тайлы
    for ty in min_ty..=max_ty { for tx in min_tx..=max_tx {
        let kind = world.get_tile(tx, ty);
        let col = match kind {
            crate::types::TileKind::Grass => [100, 170, 90, 255],
            crate::types::TileKind::Forest => [60, 130, 80, 255],
            crate::types::TileKind::Water => [70, 130, 220, 255],
        };
        let px = x + (tx - min_tx) * cell_px; let py = y + (ty - min_ty) * cell_px;
        tiles::fill_rect(frame, fw, fh, px, py, cell_px, cell_px, col);
        // на миникарте река уже учтена как вода (перезаписана в чанке), поэтому отдельной отрисовки не нужно
        // дорога поверх
        if world.is_road(IVec2::new(tx, ty)) {
            tiles::fill_rect(frame, fw, fh, px, py, cell_px, cell_px, [210, 180, 120, 255]);
        }
    }}
    // здания поверх
    for b in buildings.iter() {
        if b.pos.x < min_tx || b.pos.y < min_ty || b.pos.x > max_tx || b.pos.y > max_ty { continue; }
        let bx = x + (b.pos.x - min_tx) * cell_px; let by = y + (b.pos.y - min_ty) * cell_px;
        let c = building_color(b.kind);
        // чуть ярче на миникарте
        let brighten = |v: u8| -> u8 { (v as u16 + 40).min(255) as u8 };
        tiles::fill_rect(frame, fw, fh, bx, by, cell_px, cell_px, [brighten(c[0]), brighten(c[1]), brighten(c[2]), 255]);
    }
    // рамка текущей видимой области камеры (отрисовываем только пересечение с миникартой)
    if cam_max_tx >= min_tx && cam_min_tx <= max_tx && cam_max_ty >= min_ty && cam_min_ty <= max_ty {
        let sx_t = cam_min_tx.max(min_tx);
        let ex_t = cam_max_tx.min(max_tx);
        let sy_t = cam_min_ty.max(min_ty);
        let ey_t = cam_max_ty.min(max_ty);
        let rx = x + (sx_t - min_tx) * cell_px;
        let ry = y + (sy_t - min_ty) * cell_px;
        let rw = ((ex_t - sx_t + 1).max(1)) * cell_px;
        let rh = ((ey_t - sy_t + 1).max(1)) * cell_px;
        let col = [240, 230, 200, 180];
        // тонкая рамка в 1 пиксель
        tiles::draw_line(frame, fw, fh, rx, ry, rx + rw - 1, ry, col);
        tiles::draw_line(frame, fw, fh, rx + rw - 1, ry, rx + rw - 1, ry + rh - 1, col);
        tiles::draw_line(frame, fw, fh, rx + rw - 1, ry + rh - 1, rx, ry + rh - 1, col);
        tiles::draw_line(frame, fw, fh, rx, ry + rh - 1, rx, ry, col);
    }
}


