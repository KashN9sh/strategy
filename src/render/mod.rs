// GPU Rendering Module
// Все функции рендеринга теперь выполняются на GPU через wgpu
//
// Этот модуль предоставляет высокоуровневый API для рендеринга,
// который внутри использует GpuRenderer

pub mod gpu;
pub mod map {
    pub use super::gpu::draw_minimap;
}

// Re-export основных функций
pub use gpu::*;

