use glam::IVec2;
use rand::rngs::StdRng;
use rand::Rng;
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::input::ResolvedInput;
use crate::types::{Building, BuildingKind, Citizen, Resources};
use crate::world::World;

pub fn handle_key_press(
    key: PhysicalKey,
    input: &ResolvedInput,
    rng: &mut StdRng,
    world: &mut World,
    buildings: &mut Vec<Building>,
    buildings_dirty: &mut bool,
    citizens: &mut Vec<Citizen>,
    population: &mut i32,
    resources: &mut Resources,
    selected_building: &mut Option<BuildingKind>,
    show_grid: &mut bool,
    show_forest_overlay: &mut bool,
    show_tree_stage_overlay: &mut bool,
    show_ui: &mut bool,
    road_mode: &mut bool,
    path_debug_mode: &mut bool,
    path_sel_a: &mut Option<IVec2>,
    path_sel_b: &mut Option<IVec2>,
    last_path: &mut Option<Vec<IVec2>>,
    speed_mult: &mut f32,
    seed: &mut u64,
) {
    let PhysicalKey::Code(code) = key else { return; };
    if code == input.speed_0_5x { *speed_mult = 0.5; }
    if code == input.speed_1x { *speed_mult = 1.0; }
    if code == input.speed_2x { *speed_mult = 2.0; }
    if code == input.speed_3x { *speed_mult = 3.0; }
    if code == KeyCode::KeyG { *show_grid = !*show_grid; }
    if code == KeyCode::KeyH { *show_forest_overlay = !*show_forest_overlay; }
    if code == KeyCode::KeyJ { *show_tree_stage_overlay = !*show_tree_stage_overlay; }
    if code == KeyCode::KeyU { *show_ui = !*show_ui; }
    if code == input.toggle_road_mode { *road_mode = !*road_mode; }
    if code == KeyCode::KeyP { *path_debug_mode = !*path_debug_mode; *path_sel_a=None; *path_sel_b=None; *last_path=None; }
    if code == input.build_lumberjack { *selected_building = Some(BuildingKind::Lumberjack); }
    if code == input.build_house { *selected_building = Some(BuildingKind::House); }
}


