// Базовые UI компоненты

use crate::ui_markup::ast::{UINode, AttributeValue};

/// Базовые свойства компонентов

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Relative,
    Absolute,
    Fixed,
}

impl Position {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "absolute" => Position::Absolute,
            "fixed" => Position::Fixed,
            _ => Position::Relative,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    Start,
    Center,
    End,
    Stretch,
}

impl Alignment {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "center" => Alignment::Center,
            "end" | "right" | "bottom" => Alignment::End,
            "stretch" | "fill" => Alignment::Stretch,
            _ => Alignment::Start,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Justification {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

impl Justification {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "center" => Justification::Center,
            "end" | "right" => Justification::End,
            "space-between" | "between" => Justification::SpaceBetween,
            "space-around" | "around" => Justification::SpaceAround,
            _ => Justification::Start,
        }
    }
}

/// Размеры и позиционирование
#[derive(Debug, Clone, Copy)]
pub struct SizeValue {
    pub value: f32,
    pub unit: SizeUnit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeUnit {
    Pixels,
    Percent,
    Auto,
}

impl SizeValue {
    pub fn parse(s: &str) -> Self {
        if s == "auto" {
            return SizeValue { value: 0.0, unit: SizeUnit::Auto };
        }
        
        if s.ends_with("px") {
            let value = s.trim_end_matches("px").parse().unwrap_or(0.0);
            SizeValue { value, unit: SizeUnit::Pixels }
        } else if s.ends_with('%') {
            let value = s.trim_end_matches('%').parse().unwrap_or(0.0);
            SizeValue { value, unit: SizeUnit::Percent }
        } else {
            // По умолчанию пиксели
            let value = s.parse().unwrap_or(0.0);
            SizeValue { value, unit: SizeUnit::Pixels }
        }
    }
    
    pub fn pixels(value: f32) -> Self {
        SizeValue { value, unit: SizeUnit::Pixels }
    }
    
    pub fn percent(value: f32) -> Self {
        SizeValue { value, unit: SizeUnit::Percent }
    }
    
    pub fn auto() -> Self {
        SizeValue { value: 0.0, unit: SizeUnit::Auto }
    }
}

/// Отступы (padding/margin)
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub fn all(value: f32) -> Self {
        EdgeInsets {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
    
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        EdgeInsets {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
    
    pub fn from_node(node: &UINode) -> Self {
        // Поддержка сокращенных записей padding/margin
        if let Some(val) = node.get_number_attr("padding") {
            return EdgeInsets::all(val);
        }
        
        EdgeInsets {
            top: node.get_number_attr("padding-top").or_else(|| node.get_number_attr("pt")).unwrap_or(0.0),
            right: node.get_number_attr("padding-right").or_else(|| node.get_number_attr("pr")).unwrap_or(0.0),
            bottom: node.get_number_attr("padding-bottom").or_else(|| node.get_number_attr("pb")).unwrap_or(0.0),
            left: node.get_number_attr("padding-left").or_else(|| node.get_number_attr("pl")).unwrap_or(0.0),
        }
    }
}

/// Базовый стиль компонента (общие свойства)
#[derive(Debug, Clone)]
pub struct ComponentStyle {
    pub position: Position,
    pub width: SizeValue,
    pub height: SizeValue,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub visible: bool,
}

impl Default for ComponentStyle {
    fn default() -> Self {
        ComponentStyle {
            position: Position::Relative,
            width: SizeValue::auto(),
            height: SizeValue::auto(),
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            padding: EdgeInsets::default(),
            margin: EdgeInsets::default(),
            visible: true,
        }
    }
}

impl ComponentStyle {
    pub fn from_node(node: &UINode) -> Self {
        let position = node.get_string_attr("position")
            .map(Position::from_str)
            .unwrap_or(Position::Relative);
        
        let width = node.get_string_attr("width")
            .map(SizeValue::parse)
            .or_else(|| node.get_number_attr("width").map(SizeValue::pixels))
            .unwrap_or_else(SizeValue::auto);
        
        let height = node.get_string_attr("height")
            .map(SizeValue::parse)
            .or_else(|| node.get_number_attr("height").map(SizeValue::pixels))
            .unwrap_or_else(SizeValue::auto);
        
        ComponentStyle {
            position,
            width,
            height,
            min_width: node.get_number_attr("min-width"),
            max_width: node.get_number_attr("max-width"),
            min_height: node.get_number_attr("min-height"),
            max_height: node.get_number_attr("max-height"),
            padding: EdgeInsets::from_node(node),
            margin: EdgeInsets::default(), // TODO: добавить margin parsing
            visible: node.get_bool_attr("visible").unwrap_or(true),
        }
    }
}

/// Конкретные компоненты

/// Панель (контейнер)
#[derive(Debug, Clone)]
pub struct Panel {
    pub style: ComponentStyle,
    pub background_color: Option<[f32; 4]>,
    pub border_color: Option<[f32; 4]>,
    pub border_width: f32,
}

impl Panel {
    pub fn from_node(node: &UINode) -> Self {
        let bg_color = node.get_color_attr("background")
            .or_else(|| node.get_color_attr("bg"))
            .map(|c| [c.r, c.g, c.b, c.a]);
        
        let border_color = node.get_color_attr("border-color")
            .map(|c| [c.r, c.g, c.b, c.a]);
        
        let border_width = node.get_number_attr("border-width")
            .or_else(|| node.get_number_attr("border"))
            .unwrap_or(0.0);
        
        Panel {
            style: ComponentStyle::from_node(node),
            background_color: bg_color,
            border_color,
            border_width,
        }
    }
}

/// Кнопка
#[derive(Debug, Clone)]
pub struct Button {
    pub style: ComponentStyle,
    pub text: String,
    pub onclick: Option<String>, // Команда/событие при клике
    pub active: bool,
}

impl Button {
    pub fn from_node(node: &UINode) -> Self {
        let text = node.get_string_attr("text")
            .unwrap_or("")
            .to_string();
        
        let onclick = node.get_string_attr("onclick")
            .map(|s| s.to_string());
        
        let active = node.get_bool_attr("active")
            .or_else(|| {
                // Поддержка биндинга для активности
                node.get_attribute("active")
                    .and_then(|v| v.as_binding())
                    .map(|_| false) // По умолчанию false, значение будет из биндинга
            })
            .unwrap_or(false);
        
        Button {
            style: ComponentStyle::from_node(node),
            text,
            onclick,
            active,
        }
    }
}

/// Текстовый элемент
#[derive(Debug, Clone)]
pub struct Text {
    pub style: ComponentStyle,
    pub content: TextContent,
    pub color: [f32; 4],
    pub scale: f32,
}

#[derive(Debug, Clone)]
pub enum TextContent {
    Static(String),
    Binding(String), // bind="resources.gold"
}

impl Text {
    pub fn from_node(node: &UINode) -> Self {
        let content = if let Some(binding) = node.get_attribute("bind").and_then(|v| v.as_binding()) {
            TextContent::Binding(binding.to_string())
        } else if let Some(text) = node.get_string_attr("text") {
            TextContent::Static(text.to_string())
        } else {
            TextContent::Static(String::new())
        };
        
        let color = node.get_color_attr("color")
            .map(|c| [c.r, c.g, c.b, c.a])
            .unwrap_or([1.0, 1.0, 1.0, 1.0]);
        
        let scale = node.get_number_attr("scale")
            .unwrap_or(1.0);
        
        Text {
            style: ComponentStyle::from_node(node),
            content,
            color,
            scale,
        }
    }
}

/// Иконка
#[derive(Debug, Clone)]
pub struct Icon {
    pub style: ComponentStyle,
    pub sprite: String, // Например, "props:0" для индекса в props.png
    pub size: f32,
}

impl Icon {
    pub fn from_node(node: &UINode) -> Self {
        let sprite = node.get_string_attr("sprite")
            .unwrap_or("props:0")
            .to_string();
        
        let size = node.get_number_attr("size")
            .unwrap_or(12.0);
        
        Icon {
            style: ComponentStyle::from_node(node),
            sprite,
            size,
        }
    }
}

/// Прогресс бар
#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub style: ComponentStyle,
    pub progress: ProgressValue,
    pub color: [f32; 4],
    pub background_color: [f32; 4],
}

#[derive(Debug, Clone)]
pub enum ProgressValue {
    Static(f32),  // 0.0 - 1.0
    Binding(String),
}

impl ProgressBar {
    pub fn from_node(node: &UINode) -> Self {
        let progress = if let Some(binding) = node.get_attribute("bind").and_then(|v| v.as_binding()) {
            ProgressValue::Binding(binding.to_string())
        } else if let Some(value) = node.get_number_attr("progress") {
            ProgressValue::Static(value.clamp(0.0, 1.0))
        } else {
            ProgressValue::Static(0.0)
        };
        
        let color = node.get_color_attr("color")
            .map(|c| [c.r, c.g, c.b, c.a])
            .unwrap_or([0.2, 0.8, 0.2, 1.0]);
        
        let background_color = node.get_color_attr("background")
            .map(|c| [c.r, c.g, c.b, c.a])
            .unwrap_or([0.2, 0.2, 0.2, 1.0]);
        
        ProgressBar {
            style: ComponentStyle::from_node(node),
            progress,
            color,
            background_color,
        }
    }
}

/// Layout контейнеры
#[derive(Debug, Clone)]
pub struct HBox {
    pub style: ComponentStyle,
    pub gap: f32,
    pub align: Alignment,
    pub justify: Justification,
}

impl HBox {
    pub fn from_node(node: &UINode) -> Self {
        let gap = node.get_number_attr("gap").unwrap_or(0.0);
        
        let align = node.get_string_attr("align")
            .map(Alignment::from_str)
            .unwrap_or(Alignment::Start);
        
        let justify = node.get_string_attr("justify")
            .map(Justification::from_str)
            .unwrap_or(Justification::Start);
        
        HBox {
            style: ComponentStyle::from_node(node),
            gap,
            align,
            justify,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VBox {
    pub style: ComponentStyle,
    pub gap: f32,
    pub align: Alignment,
    pub justify: Justification,
}

impl VBox {
    pub fn from_node(node: &UINode) -> Self {
        let gap = node.get_number_attr("gap").unwrap_or(0.0);
        
        let align = node.get_string_attr("align")
            .map(Alignment::from_str)
            .unwrap_or(Alignment::Start);
        
        let justify = node.get_string_attr("justify")
            .map(Justification::from_str)
            .unwrap_or(Justification::Start);
        
        VBox {
            style: ComponentStyle::from_node(node),
            gap,
            align,
            justify,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_markup::ast::*;
    
    #[test]
    fn test_size_value_parse() {
        let px = SizeValue::parse("100px");
        assert_eq!(px.unit, SizeUnit::Pixels);
        assert_eq!(px.value, 100.0);
        
        let pct = SizeValue::parse("50%");
        assert_eq!(pct.unit, SizeUnit::Percent);
        assert_eq!(pct.value, 50.0);
        
        let auto = SizeValue::parse("auto");
        assert_eq!(auto.unit, SizeUnit::Auto);
    }
}
