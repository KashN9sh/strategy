use glam::IVec2;
use crate::game_state::GameState;
use crate::camera::Camera;
use crate::gpu_renderer::GpuRenderer;

/// Подготовить все данные для рендеринга
/// Возвращает границы видимых тайлов (min_tx, min_ty, max_tx, max_ty)
pub fn prepare_rendering_data(
    game_state: &mut GameState,
    camera: &Camera,
    gpu_renderer: &mut GpuRenderer,
) -> (i32, i32, i32, i32) {
    // Обновим атлас для текущего зума
    game_state.atlas.ensure_zoom(camera.zoom);

    // Границы видимых тайлов через инверсию проекции
    let (min_tx, min_ty, max_tx, max_ty) = camera.visible_tile_bounds(
        game_state.width_i32,
        game_state.height_i32,
        &game_state.atlas,
    );
    
    // Закажем генерацию колец чанков
    game_state.world.schedule_ring(min_tx, min_ty, max_tx, max_ty);
    // Интегрируем готовые чанки (non-blocking)
    game_state.world.integrate_ready_chunks();

    // Обновляем камеру GPU рендерера
    gpu_renderer.update_camera(camera.pos.x, camera.pos.y, camera.zoom);

    // Центр экрана нужен для рендеринга поленьев
    let screen_center = IVec2::new(game_state.width_i32 / 2, game_state.height_i32 / 2);
    
    // Подготавливаем тайлы для GPU рендеринга (с подсветкой при наведении)
    gpu_renderer.prepare_tiles(
        &mut game_state.world,
        &game_state.atlas,
        min_tx,
        min_ty,
        max_tx,
        max_ty,
        game_state.hovered_tile,
        game_state.show_deposits,
    );
    
    // Подготавливаем поленья как простые коричневые прямоугольники
    gpu_renderer.prepare_logs(
        &game_state.logs_on_ground,
        &game_state.atlas,
        camera.pos,
        screen_center,
    );
    
    // Подготавливаем структуры (здания и деревья) для GPU рендеринга с правильной сортировкой
    if game_state.buildings_dirty {
        game_state.buildings.sort_by_key(|b| b.pos.x + b.pos.y);
        game_state.buildings_dirty = false;
    }
    gpu_renderer.prepare_structures(
        &mut game_state.world,
        &game_state.buildings,
        &game_state.building_atlas,
        &game_state.tree_atlas,
        &game_state.atlas,
        min_tx,
        min_ty,
        max_tx,
        max_ty,
        game_state.hovered_tile,
    );
    gpu_renderer.prepare_citizens(&game_state.citizens, &game_state.buildings, &game_state.atlas);
    
    // Подготавливаем ночное освещение (окна домов, факелы, светлячки)
    let fireflies_data: Vec<(glam::Vec2, f32)> = game_state
        .fireflies
        .iter()
        .map(|f| (f.pos, f.phase))
        .collect();
    gpu_renderer.prepare_night_lights(
        &game_state.world,
        &game_state.buildings,
        &fireflies_data,
        &game_state.atlas,
        min_tx,
        min_ty,
        max_tx,
        max_ty,
        game_state.world_clock_ms,
        game_state.world_clock_ms / 1000.0,
        game_state.width_i32 as f32,
        game_state.height_i32 as f32,
        camera.pos.x,
        camera.pos.y,
        camera.zoom,
    );
    
    // Предпросмотр дорог при перетаскивании
    if game_state.left_mouse_down
        && game_state.road_mode
        && !game_state.preview_road_path.is_empty()
    {
        let is_building = game_state.drag_road_state.unwrap_or(true);
        gpu_renderer.prepare_road_preview(
            &game_state.preview_road_path,
            is_building,
            &game_state.atlas,
        );
    }
    
    // Предпросмотр зданий при наведении (если не в режиме дорог)
    if !game_state.road_mode {
        if let Some(tile_pos) = game_state.hovered_tile {
            let is_allowed = crate::ui_interaction::building_allowed_at(
                &mut game_state.world,
                game_state.selected_building,
                tile_pos,
            );
            gpu_renderer.prepare_building_preview(
                game_state.selected_building,
                tile_pos,
                is_allowed,
                &game_state.building_atlas,
                &game_state.atlas,
            );
        } else {
            gpu_renderer.clear_building_preview();
        }
    } else {
        gpu_renderer.clear_building_preview();
    }
    
    (min_tx, min_ty, max_tx, max_ty)
}

