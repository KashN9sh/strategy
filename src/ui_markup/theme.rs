// Система тематизации UI

use std::collections::HashMap;
use crate::ui_markup::ast::Color;

/// Цветовая палитра темы
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub primary: Color,
    pub success: Color,
    pub danger: Color,
    pub warning: Color,
    pub info: Color,
    pub dark: Color,
    pub light: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        ColorPalette {
            primary: Color::from_hex("#3498db").unwrap(),
            success: Color::from_hex("#2ecc71").unwrap(),
            danger: Color::from_hex("#e74c3c").unwrap(),
            warning: Color::from_hex("#f39c12").unwrap(),
            info: Color::from_hex("#1abc9c").unwrap(),
            dark: Color::from_hex("#2c3e50").unwrap(),
            light: Color::from_hex("#ecf0f1").unwrap(),
        }
    }
}

/// Шкала отступов
#[derive(Debug, Clone)]
pub struct SpacingScale {
    pub base: f32,
    pub scale: Vec<f32>, // [0, 8, 16, 24, 32, 40]
}

impl Default for SpacingScale {
    fn default() -> Self {
        SpacingScale {
            base: 8.0,
            scale: vec![0.0, 8.0, 16.0, 24.0, 32.0, 40.0],
        }
    }
}

impl SpacingScale {
    pub fn get(&self, level: usize) -> f32 {
        self.scale.get(level).copied().unwrap_or(self.base * level as f32)
    }
}

/// Шкала типографики
#[derive(Debug, Clone)]
pub struct TypographyScale {
    pub base_scale: f32,
    pub sizes: Vec<f32>, // [0.75, 1.0, 1.25, 1.5, 2.0]
}

impl Default for TypographyScale {
    fn default() -> Self {
        TypographyScale {
            base_scale: 1.0,
            sizes: vec![0.75, 1.0, 1.25, 1.5, 2.0],
        }
    }
}

impl TypographyScale {
    pub fn get(&self, level: usize) -> f32 {
        self.sizes.get(level).copied().unwrap_or(self.base_scale) * self.base_scale
    }
}

/// Responsive breakpoints
#[derive(Debug, Clone)]
pub struct Breakpoints {
    pub sm: f32,  // 576
    pub md: f32,  // 768
    pub lg: f32,  // 1024
    pub xl: f32,  // 1440
}

impl Default for Breakpoints {
    fn default() -> Self {
        Breakpoints {
            sm: 576.0,
            md: 768.0,
            lg: 1024.0,
            xl: 1440.0,
        }
    }
}

/// Тема UI
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ColorPalette,
    pub spacing: SpacingScale,
    pub typography: TypographyScale,
    pub breakpoints: Breakpoints,
    pub custom_values: HashMap<String, String>,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            name: "Default".to_string(),
            colors: ColorPalette::default(),
            spacing: SpacingScale::default(),
            typography: TypographyScale::default(),
            breakpoints: Breakpoints::default(),
            custom_values: HashMap::new(),
        }
    }
}

impl Theme {
    pub fn new(name: &str) -> Self {
        Theme {
            name: name.to_string(),
            ..Default::default()
        }
    }
    
    /// Загрузить тему из файла .ui
    pub fn from_file(path: &str) -> Result<Self, String> {
        // TODO: Реализовать загрузку темы из файла
        // Пока возвращаем тему по умолчанию
        Ok(Theme::default())
    }
    
    /// Получить цвет по имени
    pub fn get_color(&self, name: &str) -> Option<Color> {
        match name.to_lowercase().as_str() {
            "primary" => Some(self.colors.primary),
            "success" => Some(self.colors.success),
            "danger" => Some(self.colors.danger),
            "warning" => Some(self.colors.warning),
            "info" => Some(self.colors.info),
            "dark" => Some(self.colors.dark),
            "light" => Some(self.colors.light),
            _ => None,
        }
    }
    
    /// Получить отступ по уровню
    pub fn get_spacing(&self, level: usize) -> f32 {
        self.spacing.get(level)
    }
    
    /// Получить размер шрифта по уровню
    pub fn get_font_size(&self, level: usize) -> f32 {
        self.typography.get(level)
    }
    
    /// Определить текущий breakpoint по ширине экрана
    pub fn get_breakpoint(&self, width: f32) -> &str {
        if width < self.breakpoints.sm {
            "xs"
        } else if width < self.breakpoints.md {
            "sm"
        } else if width < self.breakpoints.lg {
            "md"
        } else if width < self.breakpoints.xl {
            "lg"
        } else {
            "xl"
        }
    }
    
    /// Установить кастомное значение
    pub fn set_custom(&mut self, key: &str, value: &str) {
        self.custom_values.insert(key.to_string(), value.to_string());
    }
    
    /// Получить кастомное значение
    pub fn get_custom(&self, key: &str) -> Option<&str> {
        self.custom_values.get(key).map(|s| s.as_str())
    }
}

/// Глобальный менеджер тем
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    active_theme: String,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut manager = ThemeManager {
            themes: HashMap::new(),
            active_theme: "default".to_string(),
        };
        
        // Регистрируем тему по умолчанию
        manager.register_theme(Theme::default());
        
        manager
    }
    
    pub fn register_theme(&mut self, theme: Theme) {
        let name = theme.name.clone().to_lowercase();
        self.themes.insert(name, theme);
    }
    
    pub fn set_active_theme(&mut self, name: &str) -> Result<(), String> {
        let name_lower = name.to_lowercase();
        if self.themes.contains_key(&name_lower) {
            self.active_theme = name_lower;
            Ok(())
        } else {
            Err(format!("Theme '{}' not found", name))
        }
    }
    
    pub fn get_active_theme(&self) -> &Theme {
        self.themes.get(&self.active_theme).unwrap()
    }
    
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(&name.to_lowercase())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "Default");
        
        let primary = theme.get_color("primary").unwrap();
        assert!(primary.r > 0.0);
    }
    
    #[test]
    fn test_spacing() {
        let theme = Theme::default();
        assert_eq!(theme.get_spacing(0), 0.0);
        assert_eq!(theme.get_spacing(1), 8.0);
    }
    
    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        assert_eq!(manager.get_active_theme().name, "Default");
        
        let dark_theme = Theme::new("Dark");
        manager.register_theme(dark_theme);
        assert!(manager.set_active_theme("dark").is_ok());
    }
}
