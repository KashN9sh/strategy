// Система биндингов для реактивного обновления UI

use crate::ui_markup::context::{RenderContext, ContextValue};
use crate::ui_markup::ast::{UITree, NodeId};
use std::collections::HashMap;

/// Отслеживание зависимостей между UI элементами и данными
pub struct BindingSystem {
    /// Маппинг: путь данных -> список узлов, которые зависят от этих данных
    dependencies: HashMap<String, Vec<NodeId>>,
    
    /// Кеш последних значений для обнаружения изменений
    last_values: HashMap<String, ContextValue>,
    
    /// Флаг "грязности" для каждого узла (нужно перерисовать)
    dirty_nodes: Vec<NodeId>,
}

impl BindingSystem {
    pub fn new() -> Self {
        BindingSystem {
            dependencies: HashMap::new(),
            last_values: HashMap::new(),
            dirty_nodes: Vec::new(),
        }
    }
    
    /// Зарегистрировать зависимость узла от данных
    pub fn register_dependency(&mut self, node_id: NodeId, data_path: &str) {
        self.dependencies
            .entry(data_path.to_string())
            .or_insert_with(Vec::new)
            .push(node_id);
    }
    
    /// Обновить значения из контекста и найти изменившиеся узлы
    pub fn update(&mut self, context: &RenderContext) -> Vec<NodeId> {
        self.dirty_nodes.clear();
        
        for (path, nodes) in &self.dependencies {
            if let Some(new_value) = context.get(path) {
                // Проверяем, изменилось ли значение
                let changed = if let Some(old_value) = self.last_values.get(path) {
                    !values_equal(old_value, new_value)
                } else {
                    true // Новое значение всегда считается изменением
                };
                
                if changed {
                    // Отмечаем все зависимые узлы как "грязные"
                    self.dirty_nodes.extend(nodes.iter().copied());
                    
                    // Обновляем кеш
                    self.last_values.insert(path.clone(), new_value.clone());
                }
            }
        }
        
        self.dirty_nodes.clone()
    }
    
    /// Очистить все зависимости
    pub fn clear(&mut self) {
        self.dependencies.clear();
        self.last_values.clear();
        self.dirty_nodes.clear();
    }
    
    /// Собрать все биндинги из дерева UI
    pub fn collect_bindings(&mut self, tree: &UITree) {
        self.clear();
        
        // Обходим все узлы и собираем биндинги
        tree.traverse(tree.root, &mut |node| {
            // Проверяем атрибуты на наличие биндингов
            for (attr_name, attr_value) in &node.attributes {
                if let Some(binding_path) = attr_value.as_binding() {
                    self.register_dependency(node.id, binding_path);
                } else if attr_name == "bind" {
                    // Атрибут bind напрямую указывает путь
                    if let Some(path) = attr_value.as_string() {
                        self.register_dependency(node.id, path);
                    }
                }
            }
            
            // Специальная обработка для conditional элементов
            if node.component_type == crate::ui_markup::ast::ComponentType::Conditional {
                if let Some(condition) = node.get_string_attr("if") {
                    // Парсим условие и извлекаем пути данных
                    extract_paths_from_condition(condition)
                        .into_iter()
                        .for_each(|path| self.register_dependency(node.id, &path));
                }
            }
        });
    }
}

impl Default for BindingSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Сравнение значений для обнаружения изменений
fn values_equal(a: &ContextValue, b: &ContextValue) -> bool {
    match (a, b) {
        (ContextValue::Int(a), ContextValue::Int(b)) => a == b,
        (ContextValue::Float(a), ContextValue::Float(b)) => (a - b).abs() < f32::EPSILON,
        (ContextValue::String(a), ContextValue::String(b)) => a == b,
        (ContextValue::Bool(a), ContextValue::Bool(b)) => a == b,
        _ => false,
    }
}

/// Извлечь пути данных из условного выражения
/// Например, из "population > 0 && resources.gold >= 100" извлекаем ["population", "resources.gold"]
fn extract_paths_from_condition(condition: &str) -> Vec<String> {
    let mut paths = Vec::new();
    
    // Разбиваем по операторам и пробелам
    let tokens: Vec<&str> = condition
        .split(|c: char| c.is_whitespace() || "()&|<>=!".contains(c))
        .filter(|s| !s.is_empty())
        .collect();
    
    for token in tokens {
        // Проверяем, что это не число и не ключевое слово
        if token.parse::<i32>().is_err() 
            && token.parse::<f32>().is_err()
            && token != "true"
            && token != "false"
            && token != "and"
            && token != "or"
            && token != "not"
        {
            paths.push(token.to_string());
        }
    }
    
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_binding_system() {
        let mut system = BindingSystem::new();
        system.register_dependency(1, "resources.gold");
        system.register_dependency(2, "resources.gold");
        system.register_dependency(3, "population");
        
        assert_eq!(system.dependencies.len(), 2);
        assert_eq!(system.dependencies.get("resources.gold").unwrap().len(), 2);
    }
    
    #[test]
    fn test_extract_paths() {
        let paths = extract_paths_from_condition("population > 0 && resources.gold >= 100");
        assert!(paths.contains(&"population".to_string()));
        assert!(paths.contains(&"resources.gold".to_string()));
        assert_eq!(paths.len(), 2);
    }
}
