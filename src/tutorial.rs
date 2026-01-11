use serde::{Serialize, Deserialize};
use crate::types::{BuildingKind, Building};

/// Шаг туториала
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TutorialStep {
    /// Приветствие - первый шаг
    Welcome,
    /// Объяснение камеры и навигации
    CameraMovement,
    /// Выбор категории Housing
    SelectHousingCategory,
    /// Постройка первого дома
    BuildHouse,
    /// Объяснение жителей
    ExplainCitizens,
    /// Выбор категории Forestry
    SelectForestryCategory,
    /// Постройка лесопилки
    BuildLumberjack,
    /// Выбор категории Storage
    SelectStorageCategory,
    /// Постройка склада
    BuildWarehouse,
    /// Объяснение ресурсов
    ExplainResources,
    /// Объяснение управления скоростью
    ExplainSpeed,
    /// Объяснение дорог
    ExplainRoads,
    /// Объяснение депозитов
    ExplainDeposits,
    /// Завершение туториала
    Complete,
}

impl TutorialStep {
    /// Получить следующий шаг
    pub fn next(&self) -> Option<TutorialStep> {
        match self {
            TutorialStep::Welcome => Some(TutorialStep::CameraMovement),
            TutorialStep::CameraMovement => Some(TutorialStep::SelectHousingCategory),
            TutorialStep::SelectHousingCategory => Some(TutorialStep::BuildHouse),
            TutorialStep::BuildHouse => Some(TutorialStep::ExplainCitizens),
            TutorialStep::ExplainCitizens => Some(TutorialStep::SelectForestryCategory),
            TutorialStep::SelectForestryCategory => Some(TutorialStep::BuildLumberjack),
            TutorialStep::BuildLumberjack => Some(TutorialStep::SelectStorageCategory),
            TutorialStep::SelectStorageCategory => Some(TutorialStep::BuildWarehouse),
            TutorialStep::BuildWarehouse => Some(TutorialStep::ExplainResources),
            TutorialStep::ExplainResources => Some(TutorialStep::ExplainRoads),
            TutorialStep::ExplainRoads => Some(TutorialStep::ExplainDeposits),
            TutorialStep::ExplainDeposits => Some(TutorialStep::ExplainSpeed),
            TutorialStep::ExplainSpeed => Some(TutorialStep::Complete),
            TutorialStep::Complete => None,
        }
    }
    
    /// Получить заголовок шага
    pub fn title(&self) -> &'static str {
        match self {
            TutorialStep::Welcome => "Welcome to Cozy Kingdom!",
            TutorialStep::CameraMovement => "Camera Controls",
            TutorialStep::SelectHousingCategory => "Building Houses",
            TutorialStep::BuildHouse => "Place Your First House",
            TutorialStep::ExplainCitizens => "Your Citizens",
            TutorialStep::SelectForestryCategory => "Gathering Resources",
            TutorialStep::BuildLumberjack => "Build a Lumberjack",
            TutorialStep::SelectStorageCategory => "Storage",
            TutorialStep::BuildWarehouse => "Build a Warehouse",
            TutorialStep::ExplainResources => "Resources Overview",
            TutorialStep::ExplainRoads => "Building Roads",
            TutorialStep::ExplainDeposits => "Resource Deposits",
            TutorialStep::ExplainSpeed => "Game Speed",
            TutorialStep::Complete => "Tutorial Complete!",
        }
    }
    
    /// Получить текст подсказки
    pub fn message(&self) -> &'static str {
        match self {
            TutorialStep::Welcome => 
                "Welcome, Your Majesty! This is your new kingdom.\nLet's learn the basics of building a prosperous settlement.",
            TutorialStep::CameraMovement => 
                "Use WASD to move the camera.\nScroll the mouse wheel to zoom in and out.\nPress SPACE to continue.",
            TutorialStep::SelectHousingCategory => 
                "Click on the 'Housing' category in the bottom panel\nto see available housing buildings.",
            TutorialStep::BuildHouse => 
                "Select 'House' and click on the map to place it.\nHouses provide shelter for your citizens.",
            TutorialStep::ExplainCitizens => 
                "Citizens will move into houses automatically.\nThey need food and work to be happy.\nPress SPACE to continue.",
            TutorialStep::SelectForestryCategory => 
                "Click on the 'Forestry' category\nto see wood production buildings.",
            TutorialStep::BuildLumberjack => 
                "Build a Lumberjack near trees.\nLumberjacks cut down trees to produce wood.",
            TutorialStep::SelectStorageCategory => 
                "Click on the 'Storage' category\nto see storage buildings.",
            TutorialStep::BuildWarehouse => 
                "Build a Warehouse to store your resources.\nWarehouses increase your storage capacity.",
            TutorialStep::ExplainResources => 
                "Resources are shown in the top panel:\nWood, Stone, Food, and Gold.\nPress SPACE to continue.",
            TutorialStep::ExplainSpeed => 
                "Press 1, 2, 3 to change game speed.\nPress SPACE to pause the game.\nPress SPACE to continue.",
            TutorialStep::ExplainRoads => 
                "Press R to enter road building mode.\nClick and drag to build roads.\nRoads help citizens move faster.\nPress SPACE to continue.",
            TutorialStep::ExplainDeposits => 
                "Click the 'Deposits' button to see\nresource deposits on the map.\nBuild mines near deposits for better production.\nPress SPACE to continue.",
            TutorialStep::Complete => 
                "Congratulations! You've completed the tutorial.\nNow build your kingdom and make it prosper!",
        }
    }
    
    /// Получить элемент UI для подсветки (если есть)
    pub fn highlight_element(&self) -> Option<TutorialHighlight> {
        match self {
            TutorialStep::SelectHousingCategory => Some(TutorialHighlight::Category(crate::ui::UICategory::Housing)),
            TutorialStep::BuildHouse => Some(TutorialHighlight::Building(BuildingKind::House)),
            TutorialStep::SelectForestryCategory => Some(TutorialHighlight::Category(crate::ui::UICategory::Forestry)),
            TutorialStep::BuildLumberjack => Some(TutorialHighlight::Building(BuildingKind::Lumberjack)),
            TutorialStep::SelectStorageCategory => Some(TutorialHighlight::Category(crate::ui::UICategory::Storage)),
            TutorialStep::BuildWarehouse => Some(TutorialHighlight::Building(BuildingKind::Warehouse)),
            _ => None,
        }
    }
    
    /// Проверить условие завершения шага
    pub fn is_complete(&self, context: &TutorialContext) -> bool {
        match self {
            TutorialStep::Welcome => context.space_pressed,
            TutorialStep::CameraMovement => context.space_pressed,
            TutorialStep::SelectHousingCategory => context.current_category == crate::ui::UICategory::Housing,
            TutorialStep::BuildHouse => context.house_count > 0,
            TutorialStep::ExplainCitizens => context.space_pressed,
            TutorialStep::SelectForestryCategory => context.current_category == crate::ui::UICategory::Forestry,
            TutorialStep::BuildLumberjack => context.lumberjack_count > 0,
            TutorialStep::SelectStorageCategory => context.current_category == crate::ui::UICategory::Storage,
            TutorialStep::BuildWarehouse => context.warehouse_count > 0,
            TutorialStep::ExplainResources => context.space_pressed,
            TutorialStep::ExplainRoads => context.space_pressed,
            TutorialStep::ExplainDeposits => context.space_pressed,
            TutorialStep::ExplainSpeed => context.space_pressed,
            TutorialStep::Complete => context.space_pressed,
        }
    }
    
    /// Требует ли шаг нажатия пробела для продолжения
    pub fn requires_space(&self) -> bool {
        matches!(self, 
            TutorialStep::Welcome | 
            TutorialStep::CameraMovement | 
            TutorialStep::ExplainCitizens |
            TutorialStep::ExplainResources |
            TutorialStep::ExplainRoads |
            TutorialStep::ExplainDeposits |
            TutorialStep::ExplainSpeed |
            TutorialStep::Complete
        )
    }
    
    /// Требует ли шаг интеракции (клика) - в этом случае панель должна быть в углу
    pub fn requires_interaction(&self) -> bool {
        // Шаги с подсветкой элементов UI
        if self.highlight_element().is_some() {
            return true;
        }
        // Шаг про дороги тоже требует интеракции (строительство дорог)
        matches!(self, TutorialStep::ExplainRoads)
    }
}

/// Элемент для подсветки в туториале
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TutorialHighlight {
    /// Подсветить категорию
    Category(crate::ui::UICategory),
    /// Подсветить кнопку здания
    Building(BuildingKind),
    /// Подсветить область на карте
    MapArea { x: i32, y: i32, radius: i32 },
}

/// Контекст для проверки условий туториала
#[derive(Default)]
pub struct TutorialContext {
    pub space_pressed: bool,
    pub current_category: crate::ui::UICategory,
    pub house_count: i32,
    pub lumberjack_count: i32,
    pub warehouse_count: i32,
}

impl TutorialContext {
    /// Создать контекст из текущего состояния игры
    pub fn from_game_state(
        category: crate::ui::UICategory,
        buildings: &[Building],
    ) -> Self {
        let house_count = buildings.iter().filter(|b| b.kind == BuildingKind::House).count() as i32;
        let lumberjack_count = buildings.iter().filter(|b| b.kind == BuildingKind::Lumberjack).count() as i32;
        let warehouse_count = buildings.iter().filter(|b| b.kind == BuildingKind::Warehouse).count() as i32;
        
        Self {
            space_pressed: false,
            current_category: category,
            house_count,
            lumberjack_count,
            warehouse_count,
        }
    }
}

/// Система туториала
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TutorialSystem {
    /// Текущий шаг туториала
    pub current_step: Option<TutorialStep>,
    /// Туториал активен
    pub active: bool,
    /// Туториал завершён
    pub completed: bool,
    /// Время показа текущего сообщения (для анимации)
    pub message_time_ms: f32,
    /// Флаг для отслеживания нажатия пробела
    #[serde(skip)]
    pub space_pressed_this_frame: bool,
    /// Позиция панели (0.0 = центр, 1.0 = правый верхний угол)
    pub panel_position: f32,
    /// Целевая позиция панели
    pub target_panel_position: f32,
}

impl TutorialSystem {
    /// Создать новую систему туториала
    pub fn new() -> Self {
        Self {
            current_step: Some(TutorialStep::Welcome),
            active: true,
            completed: false,
            message_time_ms: 0.0,
            space_pressed_this_frame: false,
            panel_position: 0.0,
            target_panel_position: 0.0,
        }
    }
    
    /// Создать систему туториала в выключенном состоянии (для загруженных игр)
    pub fn new_inactive() -> Self {
        Self {
            current_step: None,
            active: false,
            completed: true,
            message_time_ms: 0.0,
            space_pressed_this_frame: false,
            panel_position: 0.0,
            target_panel_position: 0.0,
        }
    }
    
    /// Обновить туториал
    pub fn update(&mut self, delta_ms: f32, context: &TutorialContext) {
        if !self.active || self.completed {
            return;
        }
        
        self.message_time_ms += delta_ms;
        
        // Определяем целевую позицию панели
        if let Some(ref step) = self.current_step {
            if step.requires_interaction() {
                // Если шаг требует интеракции - панель в правом верхнем углу
                self.target_panel_position = 1.0;
            } else {
                // Иначе - панель в центре
                self.target_panel_position = 0.0;
            }
        }
        
        // Плавная анимация перемещения панели
        let move_speed = 2.0; // Скорость перемещения (1.0 = мгновенно)
        let move_delta = (delta_ms / 1000.0) * move_speed;
        if self.panel_position < self.target_panel_position {
            self.panel_position = (self.panel_position + move_delta).min(self.target_panel_position);
        } else if self.panel_position > self.target_panel_position {
            self.panel_position = (self.panel_position - move_delta).max(self.target_panel_position);
        }
        
        // Создаём контекст с учётом нажатия пробела
        let ctx = TutorialContext {
            space_pressed: self.space_pressed_this_frame,
            current_category: context.current_category,
            house_count: context.house_count,
            lumberjack_count: context.lumberjack_count,
            warehouse_count: context.warehouse_count,
        };
        
        // Проверяем условие завершения текущего шага
        if let Some(ref step) = self.current_step {
            if step.is_complete(&ctx) {
                self.advance();
            }
        }
        
        // Сбрасываем флаг нажатия пробела
        self.space_pressed_this_frame = false;
    }
    
    /// Перейти к следующему шагу
    pub fn advance(&mut self) {
        if let Some(ref step) = self.current_step {
            if let Some(next) = step.next() {
                self.current_step = Some(next);
                self.message_time_ms = 0.0;
            } else {
                // Туториал завершён
                self.current_step = None;
                self.active = false;
                self.completed = true;
            }
        }
    }
    
    /// Пропустить туториал
    pub fn skip(&mut self) {
        self.current_step = None;
        self.active = false;
        self.completed = true;
    }
    
    /// Обработать нажатие пробела
    pub fn handle_space(&mut self) {
        self.space_pressed_this_frame = true;
    }
    
    /// Получить текущий элемент для подсветки
    pub fn current_highlight(&self) -> Option<TutorialHighlight> {
        self.current_step.as_ref().and_then(|s| s.highlight_element())
    }
    
    /// Получить текущий заголовок
    pub fn current_title(&self) -> Option<&'static str> {
        self.current_step.as_ref().map(|s| s.title())
    }
    
    /// Получить текущее сообщение
    pub fn current_message(&self) -> Option<&'static str> {
        self.current_step.as_ref().map(|s| s.message())
    }
    
    /// Требует ли текущий шаг нажатия пробела
    pub fn requires_space(&self) -> bool {
        self.current_step.as_ref().map(|s| s.requires_space()).unwrap_or(false)
    }
}

impl Default for TutorialSystem {
    fn default() -> Self {
        Self::new()
    }
}
