// Парсер для разметки UI (indent-based синтаксис)

use crate::ui_markup::lexer::{Token, tokenize};
use crate::ui_markup::ast::*;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken { expected: String, found: Token, line: usize },
    UnexpectedEof,
    InvalidAttribute(String),
    IOError(String),
    LexerError(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found, line } => {
                write!(f, "Line {}: Expected {}, found {}", line, expected, found)
            }
            ParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            ParseError::InvalidAttribute(msg) => write!(f, "Invalid attribute: {}", msg),
            ParseError::IOError(msg) => write!(f, "IO error: {}", msg),
            ParseError::LexerError(msg) => write!(f, "Lexer error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    line: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
            line: 1,
        }
    }
    
    fn current_token(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position]
        } else {
            &Token::Eof
        }
    }
    
    fn peek_token(&self, offset: usize) -> &Token {
        let pos = self.position + offset;
        if pos < self.tokens.len() {
            &self.tokens[pos]
        } else {
            &Token::Eof
        }
    }
    
    fn advance(&mut self) -> Token {
        let token = self.current_token().clone();
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        if matches!(token, Token::Newline) {
            self.line += 1;
        }
        token
    }
    
    fn expect_token(&mut self, expected: &str) -> Result<Token, ParseError> {
        let token = self.advance();
        if matches!(token, Token::Eof) {
            return Err(ParseError::UnexpectedEof);
        }
        Ok(token)
    }
    
    fn skip_newlines(&mut self) {
        while matches!(self.current_token(), Token::Newline) {
            self.advance();
        }
    }
    
    fn parse_attribute_value(&mut self) -> Result<AttributeValue, ParseError> {
        match self.current_token() {
            Token::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(AttributeValue::String(value))
            }
            Token::Number(n) => {
                let value = *n;
                self.advance();
                Ok(AttributeValue::Number(value))
            }
            Token::Identifier(id) => {
                let id_str = id.clone();
                self.advance();
                
                // Проверка на boolean значения
                match id_str.to_lowercase().as_str() {
                    "true" => Ok(AttributeValue::Bool(true)),
                    "false" => Ok(AttributeValue::Bool(false)),
                    _ => {
                        // Проверка на цвет (начинается с #)
                        if id_str.starts_with('#') {
                            if let Some(color) = Color::from_hex(&id_str) {
                                Ok(AttributeValue::Color(color))
                            } else {
                                Err(ParseError::InvalidAttribute(format!("Invalid color format: {}", id_str)))
                            }
                        } else {
                            // Иначе это идентификатор (обычная строка)
                            Ok(AttributeValue::String(id_str))
                        }
                    }
                }
            }
            Token::Hash => {
                // Цвет в формате #RRGGBB
                self.advance();
                if let Token::Identifier(hex) = self.current_token() {
                    let hex_str = format!("#{}", hex);
                    self.advance();
                    
                    if let Some(color) = Color::from_hex(&hex_str) {
                        Ok(AttributeValue::Color(color))
                    } else {
                        Err(ParseError::InvalidAttribute(format!("Invalid color: {}", hex_str)))
                    }
                } else {
                    Err(ParseError::InvalidAttribute("Expected hex color after #".to_string()))
                }
            }
            token => {
                Err(ParseError::UnexpectedToken {
                    expected: "attribute value".to_string(),
                    found: token.clone(),
                    line: self.line,
                })
            }
        }
    }
    
    fn parse_attributes(&mut self) -> Result<Attributes, ParseError> {
        let mut attributes = Attributes::new();
        
        loop {
            // Проверяем, что следующий токен - это идентификатор атрибута
            if !matches!(self.current_token(), Token::Identifier(_)) {
                break;
            }
            
            // Проверяем, что после идентификатора идет =
            if !matches!(self.peek_token(1), Token::Equals) {
                break;
            }
            
            // Парсим атрибут
            if let Token::Identifier(key) = self.advance() {
                self.advance(); // skip '='
                
                let value = self.parse_attribute_value()?;
                attributes.insert(key, value);
            }
        }
        
        Ok(attributes)
    }
    
    fn parse_node(&mut self, tree: &mut UITree, indent_level: usize) -> Result<NodeId, ParseError> {
        // Парсим тип компонента
        let component_type = match self.advance() {
            Token::Identifier(type_str) => ComponentType::from_str(&type_str),
            token => {
                return Err(ParseError::UnexpectedToken {
                    expected: "component type".to_string(),
                    found: token,
                    line: self.line,
                });
            }
        };
        
        // Создаем узел
        let node_id = tree.create_node(component_type);
        
        // Парсим ID элемента (если есть #id)
        if matches!(self.current_token(), Token::Hash) {
            self.advance();
            if let Token::Identifier(id) = self.advance() {
                if let Some(node) = tree.get_node_mut(node_id) {
                    node.element_id = Some(id);
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "element id after #".to_string(),
                    found: self.current_token().clone(),
                    line: self.line,
                });
            }
        }
        
        // Парсим атрибуты
        let attributes = self.parse_attributes()?;
        if let Some(node) = tree.get_node_mut(node_id) {
            node.attributes = attributes;
        }
        
        // Проверяем наличие блока { }
        let has_brace = matches!(self.current_token(), Token::LeftBrace);
        if has_brace {
            self.advance(); // skip '{'
        }
        
        // Пропускаем newline после определения узла
        self.skip_newlines();
        
        // Парсим дочерние элементы
        loop {
            // Проверяем отступ
            let child_indent = if let Token::Indent(n) = self.current_token() {
                let indent = *n;
                self.advance();
                indent
            } else if matches!(self.current_token(), Token::RightBrace) {
                // Конец блока
                if has_brace {
                    self.advance(); // skip '}'
                }
                break;
            } else if matches!(self.current_token(), Token::Eof | Token::Newline) {
                break;
            } else {
                // Нет отступа, значит это элемент на том же уровне
                break;
            };
            
            // Если отступ меньше или равен текущему уровню, выходим
            if child_indent <= indent_level {
                break;
            }
            
            // Парсим дочерний узел
            let child_id = self.parse_node(tree, child_indent)?;
            tree.add_child(node_id, child_id);
            
            self.skip_newlines();
        }
        
        Ok(node_id)
    }
    
    pub fn parse(&mut self) -> Result<UITree, ParseError> {
        let mut tree = UITree::new();
        
        self.skip_newlines();
        
        // Парсим корневой элемент (должен быть "ui")
        if matches!(self.current_token(), Token::Identifier(_)) {
            let root_id = self.parse_node(&mut tree, 0)?;
            
            // Проверяем, что корневой элемент - это UI
            if let Some(root_node) = tree.get_node(root_id) {
                if root_node.component_type != ComponentType::UI {
                    return Err(ParseError::InvalidAttribute(
                        "Root element must be 'ui'".to_string()
                    ));
                }
            }
            
            tree.root = root_id;
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "root element 'ui'".to_string(),
                found: self.current_token().clone(),
                line: self.line,
            });
        }
        
        Ok(tree)
    }
}

/// Парсить UI дерево из строки
pub fn parse_ui(input: &str) -> Result<UITree, ParseError> {
    let tokens = tokenize(input).map_err(ParseError::LexerError)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple() {
        let input = r#"
ui {
  panel #main {
    text bind="title"
  }
}
"#;
        let tree = parse_ui(input).unwrap();
        assert!(tree.nodes.len() > 1);
    }
    
    #[test]
    fn test_parse_attributes() {
        let input = r#"
ui {
  panel width=100 height=200 visible=true {
  }
}
"#;
        let tree = parse_ui(input).unwrap();
        
        // Найти панель
        let panel_id = tree.nodes.iter()
            .find(|(_, node)| node.component_type == ComponentType::Panel)
            .map(|(id, _)| *id)
            .unwrap();
        
        let panel = tree.get_node(panel_id).unwrap();
        assert_eq!(panel.get_number_attr("width"), Some(100.0));
        assert_eq!(panel.get_number_attr("height"), Some(200.0));
        assert_eq!(panel.get_bool_attr("visible"), Some(true));
    }
}
