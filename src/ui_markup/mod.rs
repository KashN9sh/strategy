// Публичный API системы разметки UI

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod components;
pub mod layout;
pub mod event;
pub mod context;
pub mod renderer;
pub mod bindings;
pub mod theme;
pub mod binary;
pub mod integration;

pub use ast::{UINode, UITree, NodeId};
pub use parser::{Parser, ParseError};
pub use components::*;
pub use layout::{LayoutEngine, LayoutNode};
pub use event::{EventSystem, UIEvent};
pub use context::RenderContext;
pub use theme::Theme;
pub use integration::{UIMarkupManager, execute_ui_command};

/// Загрузить UI дерево из текстового файла (.ui)
pub fn load_ui_from_file(path: &str) -> Result<UITree, ParseError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ParseError::IOError(format!("Failed to read file {}: {}", path, e)))?;
    
    let tokens = lexer::tokenize(&content)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Загрузить UI дерево из бинарного файла (.uib)
pub fn load_ui_from_binary(path: &str) -> Result<UITree, String> {
    binary::load_binary(path)
}

/// Сохранить UI дерево в бинарный файл (.uib)
pub fn save_ui_to_binary(tree: &UITree, path: &str) -> Result<(), String> {
    binary::save_binary(tree, path)
}
