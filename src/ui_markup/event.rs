// Система событий и обработчиков

use crate::ui_markup::ast::{UITree, NodeId};
use crate::ui_markup::layout::LayoutEngine;
use std::collections::HashMap;

/// Типы UI событий
#[derive(Debug, Clone, PartialEq)]
pub enum UIEvent {
    Click { x: f32, y: f32 },
    Hover { x: f32, y: f32 },
    MouseMove { x: f32, y: f32 },
    KeyPress { key: String },
    Scroll { delta: f32 },
}

/// Обработчик события
pub type EventHandler = Box<dyn Fn(&UIEvent) -> EventResult>;

/// Результат обработки события
#[derive(Debug, Clone, PartialEq)]
pub enum EventResult {
    Handled,      // Событие обработано, прекратить распространение
    Propagate,    // Событие должно распространяться дальше
    Command(String), // Выполнить игровую команду
}

/// Регистрация обработчиков событий для узлов
pub struct EventSystem {
    handlers: HashMap<NodeId, Vec<(EventType, String)>>, // node_id -> (event_type, command)
    hovered_node: Option<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Click,
    Hover,
    MouseMove,
    KeyPress,
    Scroll,
}

impl EventSystem {
    pub fn new() -> Self {
        EventSystem {
            handlers: HashMap::new(),
            hovered_node: None,
        }
    }
    
    /// Регистрировать обработчик события для узла
    pub fn register_handler(&mut self, node_id: NodeId, event_type: EventType, command: String) {
        self.handlers.entry(node_id)
            .or_insert_with(Vec::new)
            .push((event_type, command));
    }
    
    /// Обработать событие и вернуть команду (если есть)
    pub fn handle_event(
        &mut self,
        event: &UIEvent,
        tree: &UITree,
        layout: &LayoutEngine,
    ) -> Option<String> {
        match event {
            UIEvent::Click { x, y } => self.handle_click(*x, *y, layout),
            UIEvent::Hover { x, y } => self.handle_hover(*x, *y, layout),
            UIEvent::MouseMove { x, y } => self.handle_mouse_move(*x, *y, layout),
            _ => None,
        }
    }
    
    fn handle_click(&mut self, x: f32, y: f32, layout: &LayoutEngine) -> Option<String> {
        // Найти узел под курсором
        let node_id = layout.hit_test(x, y)?;
        
        // Найти обработчик click для этого узла
        if let Some(handlers) = self.handlers.get(&node_id) {
            for (event_type, command) in handlers {
                if *event_type == EventType::Click {
                    return Some(command.clone());
                }
            }
        }
        
        None
    }
    
    fn handle_hover(&mut self, x: f32, y: f32, layout: &LayoutEngine) -> Option<String> {
        let node_id = layout.hit_test(x, y);
        
        // Обновляем состояние hover
        if self.hovered_node != node_id {
            self.hovered_node = node_id;
            
            // Возвращаем команду hover если есть
            if let Some(id) = node_id {
                if let Some(handlers) = self.handlers.get(&id) {
                    for (event_type, command) in handlers {
                        if *event_type == EventType::Hover {
                            return Some(command.clone());
                        }
                    }
                }
            }
        }
        
        None
    }
    
    fn handle_mouse_move(&mut self, x: f32, y: f32, layout: &LayoutEngine) -> Option<String> {
        // Обновляем hover состояние при движении мыши
        self.handle_hover(x, y, layout)
    }
    
    /// Получить текущий наведенный узел
    pub fn get_hovered_node(&self) -> Option<NodeId> {
        self.hovered_node
    }
    
    /// Очистить все обработчики
    pub fn clear(&mut self) {
        self.handlers.clear();
        self.hovered_node = None;
    }
}

impl Default for EventSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Парсинг команд из строковых атрибутов
pub fn parse_event_command(command_str: &str) -> Option<(String, Vec<String>)> {
    // Формат: "command_name:arg1,arg2,arg3"
    if let Some(colon_pos) = command_str.find(':') {
        let command = command_str[..colon_pos].to_string();
        let args = command_str[colon_pos + 1..]
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        Some((command, args))
    } else {
        // Просто имя команды без аргументов
        Some((command_str.to_string(), Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_system() {
        let mut system = EventSystem::new();
        system.register_handler(1, EventType::Click, "select_building:house".to_string());
        
        assert!(system.handlers.contains_key(&1));
    }
    
    #[test]
    fn test_parse_command() {
        let (cmd, args) = parse_event_command("select_building:house").unwrap();
        assert_eq!(cmd, "select_building");
        assert_eq!(args, vec!["house"]);
        
        let (cmd2, args2) = parse_event_command("pause").unwrap();
        assert_eq!(cmd2, "pause");
        assert!(args2.is_empty());
    }
}
