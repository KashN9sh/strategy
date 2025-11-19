// Контекст рендеринга с доступом к игровому состоянию

use std::collections::HashMap;

/// Контекст для рендеринга UI с доступом к игровым данным
pub struct RenderContext {
    /// Биндинги к игровым данным (например, "resources.gold" -> 100)
    bindings: HashMap<String, ContextValue>,
}

/// Значение из игрового контекста
#[derive(Debug, Clone)]
pub enum ContextValue {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
}

impl ContextValue {
    pub fn as_int(&self) -> Option<i32> {
        match self {
            ContextValue::Int(i) => Some(*i),
            ContextValue::Float(f) => Some(*f as i32),
            _ => None,
        }
    }
    
    pub fn as_float(&self) -> Option<f32> {
        match self {
            ContextValue::Float(f) => Some(*f),
            ContextValue::Int(i) => Some(*i as f32),
            _ => None,
        }
    }
    
    pub fn as_string(&self) -> String {
        match self {
            ContextValue::String(s) => s.clone(),
            ContextValue::Int(i) => i.to_string(),
            ContextValue::Float(f) => f.to_string(),
            ContextValue::Bool(b) => b.to_string(),
        }
    }
    
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ContextValue::Bool(b) => Some(*b),
            ContextValue::Int(i) => Some(*i != 0),
            _ => None,
        }
    }
}

impl RenderContext {
    pub fn new() -> Self {
        RenderContext {
            bindings: HashMap::new(),
        }
    }
    
    /// Установить значение биндинга
    pub fn set(&mut self, path: &str, value: ContextValue) {
        self.bindings.insert(path.to_string(), value);
    }
    
    /// Получить значение биндинга по пути
    pub fn get(&self, path: &str) -> Option<&ContextValue> {
        self.bindings.get(path)
    }
    
    /// Установить значения из игрового состояния
    pub fn update_from_game_state(
        &mut self,
        resources: &crate::types::Resources,
        population: i32,
        avg_happiness: f32,
        tax_rate: f32,
        paused: bool,
        speed: f32,
    ) {
        // Ресурсы
        self.set("resources.gold", ContextValue::Int(resources.gold));
        self.set("resources.wood", ContextValue::Int(resources.wood));
        self.set("resources.stone", ContextValue::Int(resources.stone));
        self.set("resources.clay", ContextValue::Int(resources.clay));
        self.set("resources.bricks", ContextValue::Int(resources.bricks));
        self.set("resources.wheat", ContextValue::Int(resources.wheat));
        self.set("resources.flour", ContextValue::Int(resources.flour));
        self.set("resources.bread", ContextValue::Int(resources.bread));
        self.set("resources.fish", ContextValue::Int(resources.fish));
        self.set("resources.iron_ore", ContextValue::Int(resources.iron_ore));
        self.set("resources.iron_ingots", ContextValue::Int(resources.iron_ingots));
        
        // Общие данные
        self.set("population", ContextValue::Int(population));
        self.set("happiness", ContextValue::Float(avg_happiness));
        self.set("tax_rate", ContextValue::Float(tax_rate));
        self.set("paused", ContextValue::Bool(paused));
        self.set("speed", ContextValue::Float(speed));
    }
    
    /// Разрешить биндинг и вернуть значение как строку
    pub fn resolve_binding(&self, binding: &str) -> String {
        if let Some(value) = self.get(binding) {
            value.as_string()
        } else {
            format!("?{}", binding) // Неразрешенный биндинг
        }
    }
    
    /// Вычислить условное выражение (для conditional rendering)
    /// Примеры: "population > 0", "paused == true", "resources.gold >= 100"
    pub fn evaluate_condition(&self, condition: &str) -> bool {
        // Простой парсер условий
        let parts: Vec<&str> = condition.split_whitespace().collect();
        
        if parts.len() == 3 {
            let left = parts[0];
            let op = parts[1];
            let right = parts[2];
            
            // Получаем значения
            let left_val = self.get(left);
            
            // Парсим правую часть (может быть числом или биндингом)
            let right_val = if let Ok(num) = right.parse::<i32>() {
                Some(ContextValue::Int(num))
            } else if let Ok(num) = right.parse::<f32>() {
                Some(ContextValue::Float(num))
            } else if right == "true" {
                Some(ContextValue::Bool(true))
            } else if right == "false" {
                Some(ContextValue::Bool(false))
            } else {
                self.get(right).cloned()
            };
            
            // Выполняем сравнение
            if let (Some(l), Some(r)) = (left_val, right_val) {
                return match op {
                    "==" => l.as_int() == r.as_int(),
                    "!=" => l.as_int() != r.as_int(),
                    ">" => l.as_int() > r.as_int(),
                    "<" => l.as_int() < r.as_int(),
                    ">=" => l.as_int() >= r.as_int(),
                    "<=" => l.as_int() <= r.as_int(),
                    _ => false,
                };
            }
        } else if parts.len() == 1 {
            // Простое условие - проверяем на true
            if let Some(val) = self.get(parts[0]) {
                return val.as_bool().unwrap_or(false);
            }
        }
        
        false
    }
    
    /// Очистить все биндинги
    pub fn clear(&mut self) {
        self.bindings.clear();
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context() {
        let mut ctx = RenderContext::new();
        ctx.set("gold", ContextValue::Int(100));
        
        assert_eq!(ctx.get("gold").unwrap().as_int(), Some(100));
        assert_eq!(ctx.resolve_binding("gold"), "100");
    }
    
    #[test]
    fn test_evaluate_condition() {
        let mut ctx = RenderContext::new();
        ctx.set("population", ContextValue::Int(10));
        ctx.set("paused", ContextValue::Bool(false));
        
        assert!(ctx.evaluate_condition("population > 0"));
        assert!(!ctx.evaluate_condition("population < 5"));
        assert!(!ctx.evaluate_condition("paused"));
    }
}
