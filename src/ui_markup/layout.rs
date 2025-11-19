// Система лейаута (flexbox-подобная)

use crate::ui_markup::ast::{UITree, UINode, NodeId, ComponentType};
use crate::ui_markup::components::*;
use std::collections::HashMap;

/// Вычисленные размеры и позиция узла
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        LayoutRect { x, y, width, height }
    }
    
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width &&
        py >= self.y && py < self.y + self.height
    }
}

/// Узел с вычисленным layout
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub node_id: NodeId,
    pub rect: LayoutRect,
    pub visible: bool,
}

/// Layout Engine - вычисляет позиции и размеры всех элементов
pub struct LayoutEngine {
    pub layouts: HashMap<NodeId, LayoutNode>,
    viewport_width: f32,
    viewport_height: f32,
}

impl LayoutEngine {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        LayoutEngine {
            layouts: HashMap::new(),
            viewport_width,
            viewport_height,
        }
    }
    
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }
    
    /// Вычислить layout для всего дерева
    pub fn compute_layout(&mut self, tree: &UITree) {
        self.layouts.clear();
        
        // Начинаем с корневого элемента
        let root_rect = LayoutRect::new(0.0, 0.0, self.viewport_width, self.viewport_height);
        self.compute_node_layout(tree, tree.root, root_rect);
    }
    
    /// Рекурсивно вычислить layout для узла и его детей
    fn compute_node_layout(&mut self, tree: &UITree, node_id: NodeId, parent_rect: LayoutRect) {
        let Some(node) = tree.get_node(node_id) else { return };
        
        let style = ComponentStyle::from_node(node);
        
        // Пропускаем невидимые элементы
        if !style.visible {
            self.layouts.insert(node_id, LayoutNode {
                node_id,
                rect: LayoutRect::default(),
                visible: false,
            });
            return;
        }
        
        // Вычисляем размеры узла
        let width = self.resolve_size(style.width, parent_rect.width);
        let height = self.resolve_size(style.height, parent_rect.height);
        
        // Применяем min/max constraints
        let width = width
            .max(style.min_width.unwrap_or(0.0))
            .min(style.max_width.unwrap_or(f32::MAX));
        let height = height
            .max(style.min_height.unwrap_or(0.0))
            .min(style.max_height.unwrap_or(f32::MAX));
        
        // Вычисляем позицию
        let (x, y) = match style.position {
            Position::Absolute | Position::Fixed => {
                // Абсолютное позиционирование (относительно parent)
                let x = node.get_number_attr("x").unwrap_or(parent_rect.x);
                let y = node.get_number_attr("y").unwrap_or(parent_rect.y);
                (x, y)
            }
            Position::Relative => {
                // Относительное позиционирование (будет определено контейнером)
                (parent_rect.x + style.padding.left, parent_rect.y + style.padding.top)
            }
        };
        
        let rect = LayoutRect::new(x, y, width, height);
        
        self.layouts.insert(node_id, LayoutNode {
            node_id,
            rect,
            visible: true,
        });
        
        // Вычисляем layout для детей в зависимости от типа контейнера
        match node.component_type {
            ComponentType::HBox => self.layout_hbox(tree, node, rect),
            ComponentType::VBox => self.layout_vbox(tree, node, rect),
            ComponentType::Panel => self.layout_children_stacked(tree, node, rect),
            _ => self.layout_children_stacked(tree, node, rect),
        }
    }
    
    /// Layout для HBox (горизонтальное размещение)
    fn layout_hbox(&mut self, tree: &UITree, node: &UINode, rect: LayoutRect) {
        let hbox = HBox::from_node(node);
        let padding = hbox.style.padding;
        
        let content_width = rect.width - padding.left - padding.right;
        let content_height = rect.height - padding.top - padding.bottom;
        
        let children: Vec<NodeId> = node.children.clone();
        if children.is_empty() {
            return;
        }
        
        // Вычисляем размеры детей
        let total_gap = hbox.gap * (children.len() as f32 - 1.0);
        let available_width = content_width - total_gap;
        
        // Простое распределение: равномерно делим ширину
        let child_width = available_width / children.len() as f32;
        
        let mut current_x = rect.x + padding.left;
        let start_y = rect.y + padding.top;
        
        for &child_id in &children {
            let child_rect = LayoutRect::new(current_x, start_y, child_width, content_height);
            self.compute_node_layout(tree, child_id, child_rect);
            current_x += child_width + hbox.gap;
        }
    }
    
    /// Layout для VBox (вертикальное размещение)
    fn layout_vbox(&mut self, tree: &UITree, node: &UINode, rect: LayoutRect) {
        let vbox = VBox::from_node(node);
        let padding = vbox.style.padding;
        
        let content_width = rect.width - padding.left - padding.right;
        let content_height = rect.height - padding.top - padding.bottom;
        
        let children: Vec<NodeId> = node.children.clone();
        if children.is_empty() {
            return;
        }
        
        // Вычисляем размеры детей
        let total_gap = vbox.gap * (children.len() as f32 - 1.0);
        let available_height = content_height - total_gap;
        
        // Простое распределение: равномерно делим высоту
        let child_height = available_height / children.len() as f32;
        
        let start_x = rect.x + padding.left;
        let mut current_y = rect.y + padding.top;
        
        for &child_id in &children {
            let child_rect = LayoutRect::new(start_x, current_y, content_width, child_height);
            self.compute_node_layout(tree, child_id, child_rect);
            current_y += child_height + vbox.gap;
        }
    }
    
    /// Layout для обычных контейнеров (дети накладываются друг на друга)
    fn layout_children_stacked(&mut self, tree: &UITree, node: &UINode, rect: LayoutRect) {
        let style = ComponentStyle::from_node(node);
        let padding = style.padding;
        
        let content_rect = LayoutRect::new(
            rect.x + padding.left,
            rect.y + padding.top,
            rect.width - padding.left - padding.right,
            rect.height - padding.top - padding.bottom,
        );
        
        for &child_id in &node.children {
            self.compute_node_layout(tree, child_id, content_rect);
        }
    }
    
    /// Разрешить размер (auto, pixels, percent)
    fn resolve_size(&self, size: SizeValue, parent_size: f32) -> f32 {
        match size.unit {
            SizeUnit::Pixels => size.value,
            SizeUnit::Percent => parent_size * (size.value / 100.0),
            SizeUnit::Auto => parent_size, // По умолчанию заполняем весь parent
        }
    }
    
    /// Найти узел в указанной точке (для hit-testing)
    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeId> {
        // Ищем самый верхний (последний в порядке отрисовки) элемент под курсором
        self.layouts.iter()
            .filter(|(_, layout)| layout.visible && layout.rect.contains_point(x, y))
            .max_by_key(|(id, _)| *id) // Элементы с большим ID рисуются позже
            .map(|(id, _)| *id)
    }
    
    /// Получить layout узла
    pub fn get_layout(&self, node_id: NodeId) -> Option<&LayoutNode> {
        self.layouts.get(&node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_markup::ast::*;
    
    #[test]
    fn test_layout_engine() {
        let mut tree = UITree::new();
        let mut engine = LayoutEngine::new(800.0, 600.0);
        
        engine.compute_layout(&tree);
        
        let root_layout = engine.get_layout(tree.root).unwrap();
        assert_eq!(root_layout.rect.width, 800.0);
        assert_eq!(root_layout.rect.height, 600.0);
    }
    
    #[test]
    fn test_hit_test() {
        let mut tree = UITree::new();
        let mut engine = LayoutEngine::new(800.0, 600.0);
        
        engine.compute_layout(&tree);
        
        // Тест hit-test в корневом элементе
        let hit = engine.hit_test(400.0, 300.0);
        assert!(hit.is_some());
    }
}
