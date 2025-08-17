use std::sync::atomic::{AtomicI32, Ordering};

// Глобальный масштаб клетки миникарты (px на клетку)
pub static MINIMAP_CELL_PX: AtomicI32 = AtomicI32::new(0);

pub fn get_minimap_cell_px() -> i32 {
    MINIMAP_CELL_PX.load(Ordering::Relaxed)
}

pub fn set_minimap_cell_px(value: i32) {
    MINIMAP_CELL_PX.store(value, Ordering::Relaxed);
}
