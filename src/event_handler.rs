use winit::event::{ElementState, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::event_loop::EventLoopWindowTarget;
use winit::dpi::PhysicalSize;
use glam::IVec2;
use crate::camera::Camera;
use crate::game_state::GameState;
use crate::input::ResolvedInput;
use crate::save;
use crate::gpu_renderer::GpuRenderer;
use crate::ui_interaction;
use crate::controls;

/// Обработать событие клавиатуры
pub fn handle_keyboard_input(
    key: PhysicalKey,
    state: &ElementState,
    elwt: &EventLoopWindowTarget<()>,
    game_state: &mut GameState,
    camera: &mut Camera,
    input: &ResolvedInput,
    config: &crate::input::Config,
) -> bool {
    // true = событие обработано, не нужно дальше обрабатывать
    
    if *state != ElementState::Pressed {
        return false;
    }
    
    // Обработка консоли
    if game_state.console.handle_key(key) {
        // Если Enter нажат и консоль открыта - выполнить команду
        if let PhysicalKey::Code(KeyCode::Enter) = key {
            if game_state.console.open && !game_state.console.input.is_empty() {
                let cmd = game_state.console.input.clone();
                game_state.console.log.push(format!("> {}", cmd));
                game_state.console.execute_command(
                    &cmd,
                    &mut game_state.resources,
                    &mut game_state.weather_system,
                    &mut game_state.world_clock_ms,
                    &mut game_state.world,
                    &mut game_state.biome_overlay_debug,
                    &mut game_state.biome_debug_mode,
                    &mut game_state.show_deposits,
                    &mut game_state.rng,
                );
                game_state.console.input.clear();
            }
        }
        return true;
    }
    
    if key == PhysicalKey::Code(KeyCode::Escape) {
        elwt.exit();
        return true;
    }

    // Движение камеры
    if key == PhysicalKey::Code(input.move_up) {
        camera.move_by(0.0, -80.0);
        return false;
    }
    if key == PhysicalKey::Code(input.move_down) {
        camera.move_by(0.0, 80.0);
        return false;
    }
    if key == PhysicalKey::Code(input.move_left) {
        camera.move_by(-80.0, 0.0);
        return false;
    }
    if key == PhysicalKey::Code(input.move_right) {
        camera.move_by(80.0, 0.0);
        return false;
    }
    
    // Зум
    if key == PhysicalKey::Code(input.zoom_out) {
        camera.zoom_by_factor(0.9, 0.5, 8.0);
        return false;
    }
    if key == PhysicalKey::Code(input.zoom_in) {
        camera.zoom_by_factor(1.1, 0.5, 8.0);
        return false;
    }
    
    // Пауза
    if key == PhysicalKey::Code(input.toggle_pause) {
        game_state.paused = !game_state.paused;
        return false;
    }
    
    // Налоги
    if key == PhysicalKey::Code(input.tax_up) {
        game_state.tax_rate = (game_state.tax_rate + config.tax_step).min(config.tax_max);
        return false;
    }
    if key == PhysicalKey::Code(input.tax_down) {
        game_state.tax_rate = (game_state.tax_rate - config.tax_step).max(config.tax_min);
        return false;
    }
    
    // Сохранение/загрузка
    if key == PhysicalKey::Code(input.save_game) {
        let _ = save::save_game(&save::SaveData::from_runtime(
            game_state.seed,
            &game_state.resources,
            &game_state.buildings,
            camera.pos,
            camera.zoom,
            &game_state.world,
        ));
        return false;
    }
    if key == PhysicalKey::Code(input.load_game) {
        if let Ok(save) = save::load_game() {
            game_state.seed = save.seed;
            game_state.world.reset_noise(game_state.seed);
            game_state.buildings = save.to_buildings();
            game_state.buildings_dirty = true;
            game_state.citizens.clear();
            game_state.population = 0; // пока не сохраняем жителей
            game_state.resources = save.resources;
            camera.pos = glam::Vec2::new(save.cam_x, save.cam_y);
            camera.zoom = save.zoom;
            // восстановим отметку occupied
            game_state.world.occupied.clear();
            for b in &game_state.buildings {
                game_state.world.occupy(b.pos);
            }
            // восстановим деревья
            game_state.world.trees.clear();
            game_state.world.removed_trees.clear();
            for t in &save.trees {
                game_state.world.trees.insert((t.x, t.y), crate::world::Tree {
                    stage: t.stage,
                    age_ms: t.age_ms,
                });
            }
        }
        return false;
    }
    
    // Остальные клавиши через controls
    controls::handle_key_press(
        key,
        input,
        &mut game_state.rng,
        &mut game_state.world,
        &mut game_state.buildings,
        &mut game_state.buildings_dirty,
        &mut game_state.citizens,
        &mut game_state.population,
        &mut game_state.resources,
        &mut game_state.selected_building,
        &mut game_state.show_grid,
        &mut game_state.show_forest_overlay,
        &mut game_state.show_tree_stage_overlay,
        &mut game_state.show_ui,
        &mut game_state.road_mode,
        &mut game_state.path_debug_mode,
        &mut game_state.path_sel_a,
        &mut game_state.path_sel_b,
        &mut game_state.last_path,
        &mut game_state.speed_mult,
        &mut game_state.seed,
    );
    
    false
}

/// Обработать движение курсора
pub fn handle_cursor_moved(
    position: winit::dpi::PhysicalPosition<f64>,
    game_state: &mut GameState,
    camera: &Camera,
) {
    let mx = position.x as i32;
    let my = position.y as i32;
    game_state.cursor_xy = IVec2::new(mx, my);
    game_state.hovered_tile = camera.screen_to_tile(mx, my, game_state.width_i32, game_state.height_i32, &game_state.atlas);
    
    if game_state.left_mouse_down && game_state.road_mode {
        // считаем только предпросмотр, без применения
        if game_state.drag_anchor_tile.is_none() {
            if let Some(curr) = game_state.hovered_tile {
                if game_state.drag_road_state.is_none() {
                    game_state.drag_road_state = Some(!game_state.world.is_road(curr));
                }
                game_state.drag_anchor_tile = Some(curr);
            }
        }
        if let (Some(anchor), Some(curr)) = (game_state.drag_anchor_tile, game_state.hovered_tile) {
            game_state.preview_road_path.clear();
            let mut x = anchor.x;
            let mut y = anchor.y;
            let sx = (curr.x - x).signum();
            let sy = (curr.y - y).signum();
            game_state.preview_road_path.push(IVec2::new(x, y));
            while x != curr.x {
                x += sx;
                game_state.preview_road_path.push(IVec2::new(x, y));
            }
            while y != curr.y {
                y += sy;
                game_state.preview_road_path.push(IVec2::new(x, y));
            }
        }
    }
}

/// Обработать нажатие мыши
pub fn handle_mouse_input(
    button: winit::event::MouseButton,
    state: ElementState,
    game_state: &mut GameState,
    config: &crate::input::Config,
    gpu_renderer: &mut GpuRenderer,
) -> bool {
    // true = событие обработано
    
    if button == winit::event::MouseButton::Left {
        if state == ElementState::Pressed {
            game_state.left_mouse_down = true;
            
            if game_state.road_mode {
                if let Some(tp) = game_state.hovered_tile {
                    let on = !game_state.world.is_road(tp);
                    game_state.drag_road_state = Some(on);
                    game_state.drag_anchor_tile = Some(tp);
                    game_state.preview_road_path.clear();
                    game_state.preview_road_path.push(tp);
                    return true;
                }
            }
            
            if game_state.show_ui {
                if ui_interaction::handle_left_click(
                    game_state.cursor_xy,
                    game_state.width_i32,
                    game_state.height_i32,
                    config,
                    &game_state.atlas,
                    game_state.hovered_tile,
                    &mut game_state.ui_category,
                    &mut game_state.ui_tab,
                    &mut game_state.tax_rate,
                    &mut game_state.food_policy,
                    &mut game_state.selected_building,
                    &mut game_state.active_building_panel,
                    &mut game_state.world,
                    &mut game_state.buildings,
                    &mut game_state.buildings_dirty,
                    &mut game_state.citizens,
                    &mut game_state.population,
                    &mut game_state.warehouses,
                    &mut game_state.resources,
                    &mut game_state.road_mode,
                    &mut game_state.path_debug_mode,
                    &mut game_state.path_sel_a,
                    &mut game_state.path_sel_b,
                    &mut game_state.last_path,
                    &mut game_state.show_deposits,
                ) {
                    return true;
                }
            }
            
            // остальная часть обработки ЛКМ остаётся прежней (клика по миру вне UI)
            if let Some(tp) = game_state.hovered_tile {
                if let Some(bh) = game_state.buildings.iter().find(|bb| bb.pos == tp) {
                    game_state.active_building_panel = match game_state.active_building_panel {
                        Some(cur) if cur == bh.pos => None,
                        _ => Some(bh.pos),
                    };
                    return true;
                }
                
                if game_state.path_debug_mode {
                    match (game_state.path_sel_a, game_state.path_sel_b) {
                        (None, _) => {
                            game_state.path_sel_a = Some(tp);
                            game_state.last_path = None;
                        }
                        (Some(_), None) => {
                            game_state.path_sel_b = Some(tp);
                        }
                        (Some(_), Some(_)) => {
                            game_state.path_sel_a = Some(tp);
                            game_state.path_sel_b = None;
                            game_state.last_path = None;
                        }
                    }
                    if let (Some(a), Some(b)) = (game_state.path_sel_a, game_state.path_sel_b) {
                        game_state.last_path = crate::path::astar(&game_state.world, a, b, 20_000);
                    }
                    return true;
                }
            }
        } else if state == ElementState::Released {
            game_state.left_mouse_down = false;
            
            if game_state.road_mode {
                if let Some(on) = game_state.drag_road_state {
                    for p in game_state.preview_road_path.iter() {
                        game_state.world.set_road(*p, on);
                    }
                    // Очищаем предпросмотр дорог после применения
                    gpu_renderer.clear_road_preview();
                }
            }
            
            game_state.drag_road_state = None;
            game_state.drag_anchor_tile = None;
            game_state.preview_road_path.clear();
        }
    }
    
    false
}

/// Обработать прокрутку мыши
pub fn handle_mouse_wheel(
    delta: MouseScrollDelta,
    camera: &mut Camera,
) {
    let factor = match delta {
        MouseScrollDelta::LineDelta(_, y) => if y > 0.0 { 1.1 } else { 0.9 },
        MouseScrollDelta::PixelDelta(p) => if p.y > 0.0 { 1.1 } else { 0.9 },
    };
    camera.zoom_by_factor(factor, 0.5, 8.0);
}

/// Обработать изменение размера окна
pub fn handle_resize(
    new_size: PhysicalSize<u32>,
    game_state: &mut GameState,
    gpu_renderer: &mut GpuRenderer,
) {
    game_state.width_i32 = new_size.width as i32;
    game_state.height_i32 = new_size.height as i32;
    gpu_renderer.resize(new_size);
}

