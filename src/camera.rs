use glam::{IVec2, Vec2};
use crate::atlas::TileAtlas;

/// Камера для изометрической проекции
pub struct Camera {
    pub pos: Vec2,
    pub zoom: f32,
}

impl Camera {
    /// Создать новую камеру с заданной позицией и зумом
    pub fn new(pos: Vec2, zoom: f32) -> Self {
        Self { pos, zoom }
    }

    /// Преобразовать экранные координаты в координаты тайла
    pub fn screen_to_tile(&self, mx: i32, my: i32, screen_w: i32, screen_h: i32, atlas: &TileAtlas) -> Option<IVec2> {
        screen_to_tile_px(mx, my, screen_w, screen_h, self.pos, atlas.half_w, atlas.half_h, self.zoom)
    }

    /// Получить границы видимых тайлов на экране
    pub fn visible_tile_bounds(&self, screen_w: i32, screen_h: i32, atlas: &TileAtlas) -> (i32, i32, i32, i32) {
        visible_tile_bounds_px(screen_w, screen_h, self.pos, atlas.half_w, atlas.half_h, self.zoom)
    }

    /// Переместить камеру
    pub fn move_by(&mut self, dx: f32, dy: f32) {
        self.pos.x += dx;
        self.pos.y += dy;
    }

    /// Установить зум с ограничениями
    pub fn set_zoom(&mut self, new_zoom: f32, min: f32, max: f32) {
        self.zoom = new_zoom.clamp(min, max);
    }

    /// Увеличить/уменьшить зум на множитель
    pub fn zoom_by_factor(&mut self, factor: f32, min: f32, max: f32) {
        self.set_zoom(self.zoom * factor, min, max);
    }

}

/// Преобразовать экранные координаты в координаты тайла в мире
pub fn screen_to_tile_px(mx: i32, my: i32, sw: i32, sh: i32, cam_px: Vec2, half_w: i32, half_h: i32, zoom: f32) -> Option<IVec2> {
    // экран -> мир (с учетом zoom и камеры)
    // GPU: world_x = (screen_x - sw/2) / zoom + cam_x
    //      world_y = -(screen_y - sh/2) / zoom - cam_y  (камера с +cam_y, view матрица)
    let wx = (mx - sw / 2) as f32 / zoom + cam_px.x;
    let wy = (my - sh / 2) as f32 / zoom + cam_px.y;
    
    let a = half_w as f32;
    let b = half_h as f32;
    // обратное к изометрической проекции: iso_x = (mx - my)*a, iso_y = (mx + my)*b
    let tx = 0.5 * (wy / b + wx / a) + 1.0;
    let ty = 0.5 * (wy / b - wx / a) + 1.0;
    let ix = tx.floor() as i32;
    let iy = ty.floor() as i32;
    Some(IVec2::new(ix, iy))
}

/// Вычислить границы видимых тайлов на экране
pub fn visible_tile_bounds_px(sw: i32, sh: i32, cam_px: Vec2, half_w: i32, half_h: i32, zoom: f32) -> (i32, i32, i32, i32) {
    // по четырём углам экрана
    let corners = [
        (0, 0),
        (sw, 0),
        (0, sh),
        (sw, sh),
    ];
    let mut min_tx = i32::MAX;
    let mut min_ty = i32::MAX;
    let mut max_tx = i32::MIN;
    let mut max_ty = i32::MIN;
    for (x, y) in corners {
        if let Some(tp) = screen_to_tile_px(x, y, sw, sh, cam_px, half_w, half_h, zoom) {
            min_tx = min_tx.min(tp.x);
            min_ty = min_ty.min(tp.y);
            max_tx = max_tx.max(tp.x);
            max_ty = max_ty.max(tp.y);
        }
    }
    // запас; не ограничиваем картой, чтобы рисовать воду вне карты
    if min_tx == i32::MAX {
        return (-64, -64, 64, 64);
    }
    // немного запаса вокруг экрана
    (min_tx - 2, min_ty - 2, max_tx + 2, max_ty + 2)
}

