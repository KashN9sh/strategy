use winit::event::{ElementState, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::event_loop::EventLoopWindowTarget;
use winit::dpi::PhysicalSize;
use glam::IVec2;
use crate::camera::Camera;
use crate::game_state::GameState;
use crate::input::ResolvedInput;
use crate::gpu_renderer::GpuRenderer;
use crate::ui_interaction;
use crate::controls;
use crate::commands::{Command, CommandManager, ExitCommand};

/// Обработать событие клавиатуры
pub fn handle_keyboard_input(
    key: PhysicalKey,
    state: &ElementState,
    elwt: &EventLoopWindowTarget<()>,
    game_state: &mut GameState,
    camera: &mut Camera,
    input: &ResolvedInput,
    config: &crate::input::Config,
    gpu_renderer: &mut GpuRenderer,
) -> bool {
    // true = событие обработано, не нужно дальше обрабатывать
    
    // Не обрабатываем события клавиатуры, если игра на паузе (меню паузы обрабатывает их отдельно)
    if game_state.app_state == crate::game_state::AppState::Paused {
        return false;
    }
    
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
    
    // Используем Command Pattern для обработки основных команд
    if let PhysicalKey::Code(key_code) = key {
        // Escape: отменить выбор здания, закрыть окно исследований или открыть меню паузы
        if key_code == KeyCode::Escape {
            // Закрыть окно исследований, если оно открыто
            if game_state.show_research_tree {
                game_state.show_research_tree = false;
                return true;
            }
            // Если консоль закрыта и выбрано здание - отменяем выбор
            if !game_state.console.open && game_state.selected_building.is_some() {
                game_state.selected_building = None;
                return true;
            }
            // Иначе - открыть меню паузы (если мы в игре)
            if game_state.app_state == crate::game_state::AppState::Playing {
                game_state.app_state = crate::game_state::AppState::Paused;
                return true;
            }
            // Если не в игре - выход из игры
            let exit_cmd = ExitCommand;
            return exit_cmd.execute(game_state, camera, elwt, input, config, gpu_renderer);
        }
        
        // T: открыть/закрыть окно исследований (только если есть лаборатория)
        if key_code == KeyCode::KeyT && !game_state.console.open {
            if game_state.research_system.has_research_lab {
                game_state.show_research_tree = !game_state.show_research_tree;
                // Сбрасываем скролл при открытии
                if game_state.show_research_tree {
                    game_state.research_tree_scroll = 0.0;
                }
                return true;
            }
        }
        
        // Space: продолжить туториал (если туториал активен и ожидает нажатия)
        if key_code == KeyCode::Space && !game_state.console.open && game_state.tutorial_system.active {
            game_state.tutorial_system.handle_space();
            return true;
        }
        
        // Tab: пропустить туториал
        if key_code == KeyCode::Tab && !game_state.console.open && game_state.tutorial_system.active {
            game_state.tutorial_system.skip();
            return true;
        }
        
        // Создаем менеджер команд с предустановленными командами
        let command_manager = CommandManager::create_default(input);
        
        // Выполняем команду, если она зарегистрирована
        if let Some(handled) = command_manager.execute(key_code, game_state, camera, elwt, input, config, gpu_renderer) {
            return handled;
        }
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
            
            // Обработка кликов в окне исследований
            if game_state.show_research_tree {
                // Обрабатываем клики в дереве исследований
                // Функция возвращает true если нужно закрыть окно (клик на крестик)
                let should_close = crate::ui_interaction::handle_research_tree_click(
                    game_state.cursor_xy,
                    game_state.width_i32,
                    game_state.height_i32,
                    config.ui_scale_base,
                    &mut game_state.research_system,
                    &mut game_state.warehouses,
                    &mut game_state.resources,
                    game_state.research_tree_scroll,
                );
                
                if should_close {
                    game_state.show_research_tree = false;
                }
                
                return true; // Поглощаем клик, если он в окне исследований
            }
            
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
                    &mut game_state.research_system,
                    &mut game_state.show_research_tree,
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
    game_state: &mut GameState,
    base_scale_k: f32,
) {
    // Если открыто окно исследований, скроллим его
    if game_state.show_research_tree {
        let scroll_amount = match delta {
            MouseScrollDelta::LineDelta(_, y) => y * 30.0,
            MouseScrollDelta::PixelDelta(p) => p.y as f32,
        };
        
        // Вычисляем максимальный скролл (это должно соответствовать логике в draw_research_tree_gpu)
        let window_w = (game_state.width_i32 as f32 * 0.9).max(900.0);
        let window_h = (game_state.height_i32 as f32 * 0.85).max(650.0);
        let window_x = (game_state.width_i32 as f32 - window_w) / 2.0;
        let window_y = (game_state.height_i32 as f32 - window_h) / 2.0;
        let s = crate::ui::ui_scale(game_state.height_i32, base_scale_k);
        let pad = (16 * s) as f32;
        let node_h = (80 * s) as f32;
        let gap_y = (45 * s) as f32;
        
        // Вычисляем максимальный ряд динамически (как в draw_research_tree_gpu)
        use crate::research::ResearchKind;
        let mut max_row = 0;
        for &research_kind in ResearchKind::all() {
            let (_, row) = research_kind.tree_position();
            if row > max_row {
                max_row = row;
            }
        }
        // Высота дерева: позиция верхней границы последнего узла + высота узла
        // Позиция последнего узла: max_row * (node_h + gap_y)
        // Нижняя граница последнего узла: max_row * (node_h + gap_y) + node_h
        let total_tree_height = max_row as f32 * (node_h + gap_y) + node_h;
        
        // Вычисляем высоту области дерева (точно как в draw_research_tree_gpu)
        let title_height = (28 * s) as f32;
        let info_y = window_y + pad + title_height + pad + 12.0;
        let tree_start_y = info_y + (40 * s) as f32;
        let tree_area_top = tree_start_y;
        let tree_area_bottom = window_y + window_h - pad * 2.0;
        let tree_area_height = tree_area_bottom - tree_area_top;
        
        let max_scroll = (total_tree_height - tree_area_height).max(0.0);
        
        // Обновляем скролл с ограничениями
        game_state.research_tree_scroll = (game_state.research_tree_scroll - scroll_amount)
            .max(0.0)
            .min(max_scroll);
    } else {
        // Иначе зумируем камеру
        let factor = match delta {
            MouseScrollDelta::LineDelta(_, y) => if y > 0.0 { 1.1 } else { 0.9 },
            MouseScrollDelta::PixelDelta(p) => if p.y > 0.0 { 1.1 } else { 0.9 },
        };
        camera.zoom_by_factor(factor, 0.5, 8.0);
    }
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

