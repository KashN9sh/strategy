// GPU Rendering Functions
// Высокоуровневые функции рендеринга, использующие GpuRenderer

use crate::gpu_renderer::GpuRenderer;
use crate::world::World;
use crate::atlas::{TileAtlas, BuildingAtlas, TreeAtlas};
use crate::types::{Building, Citizen};

/// Рендеринг всего мира (тайлы + здания + граждане)
pub fn render_world(
    gpu: &mut GpuRenderer,
    world: &mut World,
    buildings: &Vec<Building>,
    building_atlas: &Option<BuildingAtlas>,
    tree_atlas: &Option<TreeAtlas>,
    tile_atlas: &TileAtlas,
    citizens: &Vec<Citizen>,
    cam_x: f32,
    cam_y: f32,
    zoom: f32,
    hovered_tile: Option<glam::IVec2>,
) -> anyhow::Result<()> {
    // Определяем видимые тайлы
    let (min_tx, min_ty, max_tx, max_ty) = calculate_visible_tiles(cam_x, cam_y, zoom, tile_atlas);
    
    // Обновляем камеру
    gpu.update_camera(cam_x, cam_y, zoom);
    
    // Подготавливаем тайлы (с подсветкой при наведении)
    gpu.prepare_tiles(world, tile_atlas, min_tx, min_ty, max_tx, max_ty, hovered_tile, false);
    
    // Подготавливаем здания и деревья
    gpu.prepare_structures(world, buildings, building_atlas, tree_atlas, tile_atlas, min_tx, min_ty, max_tx, max_ty, hovered_tile);
    
    // Подготавливаем граждан
    gpu.prepare_citizens(citizens, buildings, tile_atlas);
    
    Ok(())
}

/// Вычисляет границы видимых тайлов
fn calculate_visible_tiles(
    cam_x: f32,
    cam_y: f32,
    zoom: f32,
    atlas: &TileAtlas,
) -> (i32, i32, i32, i32) {
    // Примерная оценка видимой области
    let visible_radius = (800.0 / zoom).max(20.0) as i32;
    let cam_tile_x = (cam_x / (atlas.half_w as f32 * 2.0)) as i32;
    let cam_tile_y = (cam_y / (atlas.half_h as f32 * 2.0)) as i32;
    
    let min_tx = cam_tile_x - visible_radius;
    let min_ty = cam_tile_y - visible_radius;
    let max_tx = cam_tile_x + visible_radius;
    let max_ty = cam_tile_y + visible_radius;
    
    (min_tx, min_ty, max_tx, max_ty)
}

/// Рендеринг UI
pub fn render_ui(
    gpu: &mut GpuRenderer,
    resources: &crate::types::Resources,
    citizens: &[Citizen],
    buildings: &[Building],
    day_progress: f32,
    happiness: f32,
    tax_rate: f32,
    weather: crate::WeatherKind,
    food_policy: crate::types::FoodPolicy,
    ui_enabled: bool,
    ui_tab: crate::ui::UITab,
    ui_category: crate::ui::UICategory,
    selected_building: crate::types::BuildingKind,
    screen_width: f32,
    screen_height: f32,
    ui_scale: f32,
) {
    if !ui_enabled {
        return;
    }
    
    gpu.clear_ui();
    
    // Верхняя панель с ресурсами
    let panel_height = 85.0;
    gpu.draw_ui_panel(0.0, 0.0, screen_width, panel_height);
    
    // ... (рендеринг ресурсов, населения, и т.д.)
    // Этот код уже реализован в main.rs и должен быть перенесен сюда
}

/// Применяет эффекты окружения (погода, ночь)
pub fn apply_environment_effects(
    gpu: &mut GpuRenderer,
    weather: crate::WeatherKind,
    day_progress: f32,
    time: f32,
) {
    use crate::WeatherKind;
    
    // Ночное освещение (обновляется через update_night_overlay в main.rs)
    
    // Обновляем погодные эффекты в GPU
    let intensity = match weather {
        WeatherKind::Clear => 0.0,
        WeatherKind::Rain => 0.8,
        WeatherKind::Fog => 0.6,
        WeatherKind::Snow => 0.7,
    };
    
    gpu.update_weather(crate::types::WeatherKind::from(weather), time, intensity);
}

// === Вспомогательные функции для обратной совместимости ===

/// Рисует миникарту (CPU версия для совместимости, TODO: переписать на GPU)
#[allow(dead_code)]
pub fn draw_minimap(
    _frame: &mut [u8], _fw: i32, _fh: i32,
    _world: &mut crate::world::World,
    _buildings: &Vec<crate::types::Building>,
    _mm_min_tx: i32, _mm_min_ty: i32, _mm_max_tx: i32, _mm_max_ty: i32,
    _x: i32, _y: i32, _cell: i32,
    _vis_min_tx: i32, _vis_min_ty: i32, _vis_max_tx: i32, _vis_max_ty: i32,
) {
    // TODO: Реализовать минимапу на GPU
    // Пока заглушка для совместимости
}

