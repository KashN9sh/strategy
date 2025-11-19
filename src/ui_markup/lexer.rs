// Токенизатор для парсера разметки UI

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Идентификаторы и литералы
    Identifier(String),
    String(String),
    Number(f32),
    
    // Символы
    LeftBrace,      // {
    RightBrace,     // }
    Equals,         // =
    Hash,           // #
    
    // Специальные
    Newline,
    Indent(usize),  // Количество пробелов отступа
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::String(s) => write!(f, "String(\"{}\")", s),
            Token::Number(n) => write!(f, "Number({})", n),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Equals => write!(f, "="),
            Token::Hash => write!(f, "#"),
            Token::Newline => write!(f, "Newline"),
            Token::Indent(n) => write!(f, "Indent({})", n),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    fn current_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }
    
    fn peek_char(&self, offset: usize) -> Option<char> {
        let pos = self.position + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }
    
    fn advance(&mut self) -> Option<char> {
        let ch = self.current_char();
        if let Some(c) = ch {
            self.position += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_line_comment(&mut self) {
        // Комментарии начинаются с //
        if self.current_char() == Some('/') && self.peek_char(1) == Some('/') {
            while self.current_char().is_some() && self.current_char() != Some('\n') {
                self.advance();
            }
        }
    }
    
    fn read_string(&mut self) -> Result<String, String> {
        let quote_char = self.advance().unwrap(); // " или '
        let mut result = String::new();
        
        loop {
            match self.current_char() {
                Some(ch) if ch == quote_char => {
                    self.advance();
                    return Ok(result);
                }
                Some('\\') => {
                    self.advance();
                    match self.current_char() {
                        Some('n') => {
                            result.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            result.push('\t');
                            self.advance();
                        }
                        Some(ch) => {
                            result.push(ch);
                            self.advance();
                        }
                        None => {
                            return Err(format!("Unexpected end of input in string at line {}", self.line));
                        }
                    }
                }
                Some(ch) => {
                    result.push(ch);
                    self.advance();
                }
                None => {
                    return Err(format!("Unclosed string at line {}", self.line));
                }
            }
        }
    }
    
    fn read_number(&mut self) -> Result<f32, String> {
        let mut num_str = String::new();
        let mut has_dot = false;
        
        // Поддержка отрицательных чисел
        if self.current_char() == Some('-') {
            num_str.push('-');
            self.advance();
        }
        
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        num_str.parse::<f32>()
            .map_err(|_| format!("Invalid number '{}' at line {}", num_str, self.line))
    }
    
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == ':' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        result
    }
    
    pub fn next_token(&mut self) -> Result<Token, String> {
        // Пропускаем комментарии
        self.skip_line_comment();
        
        // Обработка отступов в начале строки
        if self.column == 1 && self.current_char() == Some(' ') {
            let mut indent = 0;
            while self.current_char() == Some(' ') {
                indent += 1;
                self.advance();
            }
            
            // Если после отступов идет newline или EOF, пропускаем этот indent
            if self.current_char() == Some('\n') || self.current_char().is_none() {
                return self.next_token();
            }
            
            return Ok(Token::Indent(indent));
        }
        
        // Пропускаем пробелы и табы (но не в начале строки)
        if self.column > 1 {
            self.skip_whitespace();
        }
        
        match self.current_char() {
            None => Ok(Token::Eof),
            Some('\n') => {
                self.advance();
                Ok(Token::Newline)
            }
            Some('\r') => {
                self.advance();
                if self.current_char() == Some('\n') {
                    self.advance();
                }
                Ok(Token::Newline)
            }
            Some('{') => {
                self.advance();
                Ok(Token::LeftBrace)
            }
            Some('}') => {
                self.advance();
                Ok(Token::RightBrace)
            }
            Some('=') => {
                self.advance();
                Ok(Token::Equals)
            }
            Some('#') => {
                self.advance();
                Ok(Token::Hash)
            }
            Some('"') | Some('\'') => {
                let s = self.read_string()?;
                Ok(Token::String(s))
            }
            Some(ch) if ch.is_ascii_digit() || (ch == '-' && self.peek_char(1).map_or(false, |c| c.is_ascii_digit())) => {
                let num = self.read_number()?;
                Ok(Token::Number(num))
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let id = self.read_identifier();
                Ok(Token::Identifier(id))
            }
            Some(ch) => {
                Err(format!("Unexpected character '{}' at line {}:{}", ch, self.line, self.column))
            }
        }
    }
}

/// Токенизировать входную строку
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    
    loop {
        let token = lexer.next_token()?;
        if token == Token::Eof {
            tokens.push(token);
            break;
        }
        tokens.push(token);
    }
    
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_simple() {
        let input = "panel #id position=top";
        let tokens = tokenize(input).unwrap();
        
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert!(matches!(tokens[1], Token::Hash));
        assert!(matches!(tokens[2], Token::Identifier(_)));
    }
    
    #[test]
    fn test_tokenize_string() {
        let input = r#"text="Hello World""#;
        let tokens = tokenize(input).unwrap();
        
        if let Token::String(s) = &tokens[2] {
            assert_eq!(s, "Hello World");
        } else {
            panic!("Expected string token");
        }
    }
    
    #[test]
    fn test_tokenize_number() {
        let input = "width=100 height=50.5";
        let tokens = tokenize(input).unwrap();
        
        assert!(matches!(tokens[1], Token::Number(_)));
    }
}
