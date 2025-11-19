// Рендерер для UI разметки - преобразует UI дерево в команды для GpuRenderer

use crate::ui_markup::ast::*;
use crate::ui_markup::layout::{LayoutEngine, LayoutRect};
use crate::ui_markup::context::RenderContext;
use crate::ui_markup::components::*;
use crate::gpu_renderer::GpuRenderer;

/// Рендерер UI разметки
pub struct UIMarkupRenderer {
    layout_engine: LayoutEngine,
    scale: f32,
}

impl UIMarkupRenderer {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        UIMarkupRenderer {
            layout_engine: LayoutEngine::new(viewport_width, viewport_height),
            scale: 1.0,
        }
    }
    
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.layout_engine.set_viewport(width, height);
    }
    
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
    
    /// Отрендерить UI дерево через GpuRenderer
    pub fn render(
        &mut self,
        tree: &UITree,
        context: &RenderContext,
        gpu: &mut GpuRenderer,
    ) {
        // Вычисляем layout
        self.layout_engine.compute_layout(tree);
        
        // Очищаем UI буфер
        gpu.clear_ui();
        
        // Рендерим дерево рекурсивно
        self.render_node(tree, context, gpu, tree.root);
    }
    
    /// Рекурсивно отрендерить узел и его детей
    fn render_node(
        &self,
        tree: &UITree,
        context: &RenderContext,
        gpu: &mut GpuRenderer,
        node_id: NodeId,
    ) {
        let Some(node) = tree.get_node(node_id) else { return };
        let Some(layout) = self.layout_engine.get_layout(node_id) else { return };
        
        // Пропускаем невидимые элементы
        if !layout.visible {
            return;
        }
        
        // Рендерим в зависимости от типа компонента
        match &node.component_type {
            ComponentType::Panel => self.render_panel(node, layout.rect, gpu),
            ComponentType::Button => self.render_button(node, context, layout.rect, gpu),
            ComponentType::Text => self.render_text(node, context, layout.rect, gpu),
            ComponentType::Number => self.render_number(node, context, layout.rect, gpu),
            ComponentType::Icon => self.render_icon(node, layout.rect, gpu),
            ComponentType::ProgressBar => self.render_progress_bar(node, context, layout.rect, gpu),
            ComponentType::Conditional => {
                // Условный рендеринг
                if self.should_render_conditional(node, context) {
                    // Рендерим детей если условие true
                    for &child_id in &node.children {
                        self.render_node(tree, context, gpu, child_id);
                    }
                }
                return; // Не рендерим сам conditional элемент
            }
            _ => {
                // Для остальных просто рендерим детей
            }
        }
        
        // Рендерим дочерние элементы
        for &child_id in &node.children {
            self.render_node(tree, context, gpu, child_id);
        }
    }
    
    fn render_panel(&self, node: &UINode, rect: LayoutRect, gpu: &mut GpuRenderer) {
        let panel = Panel::from_node(node);
        
        // Рисуем фон панели
        if let Some(bg_color) = panel.background_color {
            gpu.add_ui_rect(rect.x, rect.y, rect.width, rect.height, bg_color);
        }
        
        // Рисуем рамку
        if panel.border_width > 0.0 {
            if let Some(border_color) = panel.border_color {
                // Верхняя линия
                gpu.add_ui_rect(rect.x, rect.y, rect.width, panel.border_width, border_color);
                // Нижняя линия
                gpu.add_ui_rect(rect.x, rect.y + rect.height - panel.border_width, rect.width, panel.border_width, border_color);
                // Левая линия
                gpu.add_ui_rect(rect.x, rect.y, panel.border_width, rect.height, border_color);
                // Правая линия
                gpu.add_ui_rect(rect.x + rect.width - panel.border_width, rect.y, panel.border_width, rect.height, border_color);
            }
        }
    }
    
    fn render_button(&self, node: &UINode, _context: &RenderContext, rect: LayoutRect, gpu: &mut GpuRenderer) {
        let button = Button::from_node(node);
        
        // Цвета кнопки в зависимости от состояния
        let bg_color = if button.active {
            [0.3, 0.5, 0.7, 1.0]
        } else {
            [0.2, 0.3, 0.4, 1.0]
        };
        
        // Используем встроенный метод для рисования кнопки
        gpu.draw_button(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            button.text.as_bytes(),
            button.active,
            self.scale,
        );
    }
    
    fn render_text(&self, node: &UINode, context: &RenderContext, rect: LayoutRect, gpu: &mut GpuRenderer) {
        let text = Text::from_node(node);
        
        let content = match &text.content {
            TextContent::Static(s) => s.clone(),
            TextContent::Binding(path) => context.resolve_binding(path),
        };
        
        gpu.draw_text(
            rect.x,
            rect.y,
            content.as_bytes(),
            text.color,
            text.scale * self.scale,
        );
    }
    
    fn render_number(&self, node: &UINode, context: &RenderContext, rect: LayoutRect, gpu: &mut GpuRenderer) {
        // Number - специализированный Text для чисел
        let binding = node.get_string_attr("bind").unwrap_or("0");
        let value = context.get(binding)
            .and_then(|v| v.as_int())
            .unwrap_or(0) as u32;
        
        let color = node.get_color_attr("color")
            .map(|c| [c.r, c.g, c.b, c.a])
            .unwrap_or([1.0, 1.0, 1.0, 1.0]);
        
        let scale = node.get_number_attr("scale").unwrap_or(1.0) * self.scale;
        
        gpu.draw_number(rect.x, rect.y, value, color, scale);
    }
    
    fn render_icon(&self, node: &UINode, rect: LayoutRect, gpu: &mut GpuRenderer) {
        let icon = Icon::from_node(node);
        
        // Парсим sprite (формат "props:0")
        if let Some(colon_pos) = icon.sprite.find(':') {
            let _atlas = &icon.sprite[..colon_pos];
            let index_str = &icon.sprite[colon_pos + 1..];
            
            if let Ok(index) = index_str.parse::<u32>() {
                let size = icon.size * self.scale;
                gpu.draw_ui_props_icon(rect.x, rect.y, size, index);
            }
        }
    }
    
    fn render_progress_bar(&self, node: &UINode, context: &RenderContext, rect: LayoutRect, gpu: &mut GpuRenderer) {
        let bar = ProgressBar::from_node(node);
        
        let progress = match &bar.progress {
            ProgressValue::Static(p) => *p,
            ProgressValue::Binding(path) => {
                context.get(path)
                    .and_then(|v| v.as_float())
                    .unwrap_or(0.0)
            }
        };
        
        // Фон
        gpu.add_ui_rect(rect.x, rect.y, rect.width, rect.height, bar.background_color);
        
        // Заполнение
        let filled_width = rect.width * progress.clamp(0.0, 1.0);
        if filled_width > 0.0 {
            gpu.add_ui_rect(rect.x, rect.y, filled_width, rect.height, bar.color);
        }
    }
    
    fn should_render_conditional(&self, node: &UINode, context: &RenderContext) -> bool {
        if let Some(condition) = node.get_string_attr("if") {
            context.evaluate_condition(condition)
        } else {
            true // Нет условия - рендерим всегда
        }
    }
    
    /// Получить layout engine (для hit-testing)
    pub fn layout_engine(&self) -> &LayoutEngine {
        &self.layout_engine
    }
    
    /// Hit-test - найти узел под курсором
    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeId> {
        self.layout_engine.hit_test(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_renderer_creation() {
        let renderer = UIMarkupRenderer::new(800.0, 600.0);
        assert_eq!(renderer.scale, 1.0);
    }
}
