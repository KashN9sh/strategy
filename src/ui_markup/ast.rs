// AST (Abstract Syntax Tree) для представления UI дерева
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Уникальный идентификатор узла в дереве
pub type NodeId = usize;

/// Атрибуты узла (key-value пары)
pub type Attributes = HashMap<String, AttributeValue>;

/// Значение атрибута
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Number(f32),
    Bool(bool),
    Color(Color),
    Binding(String), // для биндингов к игровому состоянию (например, "resources.gold")
}

impl AttributeValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            AttributeValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_number(&self) -> Option<f32> {
        match self {
            AttributeValue::Number(n) => Some(*n),
            _ => None,
        }
    }
    
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AttributeValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    pub fn as_color(&self) -> Option<&Color> {
        match self {
            AttributeValue::Color(c) => Some(c),
            _ => None,
        }
    }
    
    pub fn as_binding(&self) -> Option<&str> {
        match self {
            AttributeValue::Binding(s) => Some(s),
            _ => None,
        }
    }
}

/// Цвет в формате RGBA (0.0-1.0)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_hex(hex: &str) -> Option<Color> {
        // Парсинг цвета из шестнадцатеричного формата (#RRGGBB или #RRGGBBAA)
        let hex = hex.trim_start_matches('#');
        
        let (r, g, b, a) = if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b, 255)
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            (r, g, b, a)
        } else {
            return None;
        };
        
        Some(Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        })
    }
    
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
    
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b, a: 1.0 }
    }
}

/// Тип компонента UI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    // Root
    UI,
    
    // Layout containers
    Panel,
    HBox,
    VBox,
    Row,
    Col,
    Spacer,
    
    // UI elements
    Button,
    Text,
    Number,
    Icon,
    ProgressBar,
    Tooltip,
    
    // Bootstrap-style components
    Navbar,
    Card,
    Modal,
    Badge,
    Alert,
    Tabs,
    TabPane,
    
    // Conditional rendering
    Conditional,
    
    // Custom/Unknown
    Custom(String),
}

impl ComponentType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ui" => ComponentType::UI,
            "panel" => ComponentType::Panel,
            "hbox" => ComponentType::HBox,
            "vbox" => ComponentType::VBox,
            "row" => ComponentType::Row,
            "col" => ComponentType::Col,
            "spacer" => ComponentType::Spacer,
            "button" => ComponentType::Button,
            "text" => ComponentType::Text,
            "number" => ComponentType::Number,
            "icon" => ComponentType::Icon,
            "progressbar" => ComponentType::ProgressBar,
            "tooltip" => ComponentType::Tooltip,
            "navbar" => ComponentType::Navbar,
            "card" => ComponentType::Card,
            "modal" => ComponentType::Modal,
            "badge" => ComponentType::Badge,
            "alert" => ComponentType::Alert,
            "tabs" => ComponentType::Tabs,
            "tabpane" => ComponentType::TabPane,
            "conditional" => ComponentType::Conditional,
            _ => ComponentType::Custom(s.to_string()),
        }
    }
}

/// Узел UI дерева
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UINode {
    pub id: NodeId,
    pub component_type: ComponentType,
    pub element_id: Option<String>, // ID элемента (для #id в разметке)
    pub attributes: Attributes,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
}

impl UINode {
    pub fn new(id: NodeId, component_type: ComponentType) -> Self {
        UINode {
            id,
            component_type,
            element_id: None,
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
        }
    }
    
    pub fn with_id(mut self, element_id: String) -> Self {
        self.element_id = Some(element_id);
        self
    }
    
    pub fn with_attribute(mut self, key: String, value: AttributeValue) -> Self {
        self.attributes.insert(key, value);
        self
    }
    
    pub fn get_attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes.get(key)
    }
    
    pub fn get_string_attr(&self, key: &str) -> Option<&str> {
        self.get_attribute(key).and_then(|v| v.as_string())
    }
    
    pub fn get_number_attr(&self, key: &str) -> Option<f32> {
        self.get_attribute(key).and_then(|v| v.as_number())
    }
    
    pub fn get_bool_attr(&self, key: &str) -> Option<bool> {
        self.get_attribute(key).and_then(|v| v.as_bool())
    }
    
    pub fn get_color_attr(&self, key: &str) -> Option<&Color> {
        self.get_attribute(key).and_then(|v| v.as_color())
    }
}

/// Дерево UI (хранит все узлы)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UITree {
    pub nodes: HashMap<NodeId, UINode>,
    pub root: NodeId,
    next_id: NodeId,
}

impl UITree {
    pub fn new() -> Self {
        let mut tree = UITree {
            nodes: HashMap::new(),
            root: 0,
            next_id: 1,
        };
        
        // Создаем корневой узел
        let root_node = UINode::new(0, ComponentType::UI);
        tree.nodes.insert(0, root_node);
        
        tree
    }
    
    pub fn create_node(&mut self, component_type: ComponentType) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        
        let node = UINode::new(id, component_type);
        self.nodes.insert(id, node);
        
        id
    }
    
    pub fn get_node(&self, id: NodeId) -> Option<&UINode> {
        self.nodes.get(&id)
    }
    
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut UINode> {
        self.nodes.get_mut(&id)
    }
    
    pub fn add_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.children.push(child_id);
        }
        if let Some(child) = self.nodes.get_mut(&child_id) {
            child.parent = Some(parent_id);
        }
    }
    
    pub fn find_by_element_id(&self, element_id: &str) -> Option<NodeId> {
        self.nodes.iter()
            .find(|(_, node)| node.element_id.as_ref().map(|s| s.as_str()) == Some(element_id))
            .map(|(id, _)| *id)
    }
    
    /// Рекурсивный обход дерева (depth-first)
    pub fn traverse<F>(&self, node_id: NodeId, visitor: &mut F) 
    where
        F: FnMut(&UINode),
    {
        if let Some(node) = self.get_node(node_id) {
            visitor(node);
            for &child_id in &node.children {
                self.traverse(child_id, visitor);
            }
        }
    }
}

impl Default for UITree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF0000").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
        
        let color_alpha = Color::from_hex("#FF0000AA").unwrap();
        assert!((color_alpha.a - 0.667).abs() < 0.01);
    }
    
    #[test]
    fn test_ui_tree() {
        let mut tree = UITree::new();
        
        let panel_id = tree.create_node(ComponentType::Panel);
        let button_id = tree.create_node(ComponentType::Button);
        
        tree.add_child(tree.root, panel_id);
        tree.add_child(panel_id, button_id);
        
        assert_eq!(tree.get_node(panel_id).unwrap().parent, Some(0));
        assert_eq!(tree.get_node(button_id).unwrap().parent, Some(panel_id));
    }
}
