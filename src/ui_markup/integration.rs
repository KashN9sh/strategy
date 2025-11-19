// Интеграция системы разметки UI с игрой

use crate::ui_markup::*;
use crate::ui_markup::context::{RenderContext, ContextValue};
use crate::ui_markup::renderer::UIMarkupRenderer;
use crate::ui_markup::event::{EventSystem, UIEvent};
use crate::ui_markup::bindings::BindingSystem;
use crate::game_state::GameState;
use crate::gpu_renderer::GpuRenderer;
use crate::types;
use std::collections::HashMap;

/// Менеджер UI разметки для игры
pub struct UIMarkupManager {
    // Загруженные UI деревья
    main_ui: Option<UITree>,
    research_ui: Option<UITree>,
    building_panel_ui: Option<UITree>,
    console_ui: Option<UITree>,
    notifications_ui: Option<UITree>,
    
    // Системы
    renderer: UIMarkupRenderer,
    context: RenderContext,
    event_system: EventSystem,
    binding_system: BindingSystem,
    theme_manager: theme::ThemeManager,
    
    // Состояние
    initialized: bool,
}

impl UIMarkupManager {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        UIMarkupManager {
            main_ui: None,
            research_ui: None,
            building_panel_ui: None,
            console_ui: None,
            notifications_ui: None,
            renderer: UIMarkupRenderer::new(viewport_width, viewport_height),
            context: RenderContext::new(),
            event_system: EventSystem::new(),
            binding_system: BindingSystem::new(),
            theme_manager: theme::ThemeManager::new(),
            initialized: false,
        }
    }
    
    /// Инициализировать - загрузить все UI файлы
    pub fn initialize(&mut self) -> Result<(), String> {
        // Загружаем UI файлы
        self.main_ui = Some(load_ui_from_file("assets/ui/main.ui")?);
        self.research_ui = Some(load_ui_from_file("assets/ui/research_tree.ui")?);
        self.building_panel_ui = Some(load_ui_from_file("assets/ui/building_panel.ui")?);
        self.console_ui = Some(load_ui_from_file("assets/ui/console.ui")?);
        self.notifications_ui = Some(load_ui_from_file("assets/ui/notifications.ui")?);
        
        // Собираем биндинги
        if let Some(ref ui) = self.main_ui {
            self.binding_system.collect_bindings(ui);
        }
        
        // Регистрируем обработчики событий из UI деревьев
        self.register_event_handlers();
        
        self.initialized = true;
        Ok(())
    }
    
    /// Обновить контекст данными из игрового состояния
    pub fn update_context(&mut self, game_state: &GameState) {
        let visible = types::total_resources(&game_state.warehouses, &game_state.resources);
        let stats = types::count_citizen_states(&game_state.citizens);
        let avg_hap = if game_state.citizens.is_empty() {
            50.0
        } else {
            game_state.citizens.iter().map(|c| c.happiness as i32).sum::<i32>() as f32
                / game_state.citizens.len() as f32
        };
        
        // Обновляем контекст
        self.context.update_from_game_state(
            &visible,
            game_state.population,
            avg_hap,
            game_state.tax_rate,
            game_state.paused,
            game_state.speed_mult,
        );
        
        // Добавляем дополнительные данные
        self.context.set("citizens.idle", ContextValue::Int(stats.idle));
        self.context.set("citizens.working", ContextValue::Int(stats.working));
        self.context.set("citizens.sleeping", ContextValue::Int(stats.sleeping));
        self.context.set("citizens.hauling", ContextValue::Int(stats.hauling));
        self.context.set("citizens.fetching", ContextValue::Int(stats.fetching));
        
        // UI состояние
        self.context.set("ui_tab", ContextValue::String(format!("{:?}", game_state.ui_tab)));
        self.context.set("ui_category", ContextValue::String(format!("{:?}", game_state.ui_category)));
        
        // Day progress
        let day_progress = (game_state.world_clock_ms / crate::game_loop::DAY_LENGTH_MS).clamp(0.0, 1.0);
        self.context.set("day_progress", ContextValue::Float(day_progress));
        
        // Research
        self.context.set("has_research_lab", ContextValue::Bool(game_state.research_system.has_research_lab));
        
        // Console
        self.context.set("console.open", ContextValue::Bool(game_state.console.open));
        self.context.set("console.input", ContextValue::String(game_state.console.input.clone()));
    }
    
    /// Отрендерить UI
    pub fn render(&mut self, gpu: &mut GpuRenderer, game_state: &GameState) {
        if !self.initialized {
            return;
        }
        
        // Обновляем контекст
        self.update_context(game_state);
        
        // Рендерим основной UI
        if let Some(ref ui) = self.main_ui {
            self.renderer.render(ui, &self.context, gpu);
        }
        
        // Рендерим окно исследований (если открыто)
        if game_state.show_research_tree {
            if let Some(ref ui) = self.research_ui {
                self.renderer.render(ui, &self.context, gpu);
            }
        }
        
        // Рендерим панель здания (если активна)
        if game_state.active_building_panel.is_some() {
            if let Some(ref ui) = self.building_panel_ui {
                self.renderer.render(ui, &self.context, gpu);
            }
        }
        
        // Рендерим консоль (если открыта)
        if game_state.console.open {
            if let Some(ref ui) = self.console_ui {
                self.renderer.render(ui, &self.context, gpu);
            }
        }
        
        // Рендерим уведомления
        if let Some(ref ui) = self.notifications_ui {
            self.renderer.render(ui, &self.context, gpu);
        }
    }
    
    /// Обработать событие (клик, hover и т.д.)
    pub fn handle_event(&mut self, event: UIEvent) -> Option<String> {
        if !self.initialized || self.main_ui.is_none() {
            return None;
        }
        
        let ui = self.main_ui.as_ref().unwrap();
        let layout = self.renderer.layout_engine();
        
        self.event_system.handle_event(&event, ui, layout)
    }
    
    /// Обработать клик мыши
    pub fn handle_click(&mut self, x: f32, y: f32) -> Option<String> {
        self.handle_event(UIEvent::Click { x, y })
    }
    
    /// Обработать движение мыши
    pub fn handle_mouse_move(&mut self, x: f32, y: f32) -> Option<String> {
        self.handle_event(UIEvent::MouseMove { x, y })
    }
    
    /// Получить наведенный узел (для тултипов)
    pub fn get_hovered_node(&self) -> Option<NodeId> {
        self.event_system.get_hovered_node()
    }
    
    /// Обновить размер viewport
    pub fn resize(&mut self, width: f32, height: f32) {
        self.renderer.set_viewport(width, height);
    }
    
    /// Установить масштаб UI
    pub fn set_scale(&mut self, scale: f32) {
        self.renderer.set_scale(scale);
    }
    
    /// Зарегистрировать обработчики событий из UI
    fn register_event_handlers(&mut self) {
        if let Some(ref ui) = self.main_ui {
            // Обходим дерево и регистрируем обработчики из атрибутов onclick, onhover и т.д.
            ui.traverse(ui.root, &mut |node| {
                if let Some(onclick) = node.get_string_attr("onclick") {
                    self.event_system.register_handler(
                        node.id,
                        event::EventType::Click,
                        onclick.to_string(),
                    );
                }
                
                if let Some(onhover) = node.get_string_attr("onhover") {
                    self.event_system.register_handler(
                        node.id,
                        event::EventType::Hover,
                        onhover.to_string(),
                    );
                }
            });
        }
    }
    
    /// Перезагрузить UI (для hot-reload в dev режиме)
    pub fn reload(&mut self) -> Result<(), String> {
        self.initialized = false;
        self.event_system.clear();
        self.binding_system.clear();
        self.initialize()
    }
}

/// Выполнить игровую команду из UI
pub fn execute_ui_command(command: &str, game_state: &mut GameState) -> bool {
    let parts: Vec<&str> = command.split(':').collect();
    let cmd = parts[0];
    let args = if parts.len() > 1 { parts[1..].to_vec() } else { vec![] };
    
    match cmd {
        "switch_tab" => {
            if let Some(tab_name) = args.first() {
                match *tab_name {
                    "build" => game_state.ui_tab = crate::ui::UITab::Build,
                    "economy" => game_state.ui_tab = crate::ui::UITab::Economy,
                    _ => return false,
                }
                return true;
            }
        }
        "select_category" => {
            if let Some(cat_name) = args.first() {
                use crate::ui::UICategory;
                game_state.ui_category = match *cat_name {
                    "housing" => UICategory::Housing,
                    "storage" => UICategory::Storage,
                    "forestry" => UICategory::Forestry,
                    "mining" => UICategory::Mining,
                    "food" => UICategory::Food,
                    "logistics" => UICategory::Logistics,
                    "research" => UICategory::Research,
                    _ => return false,
                };
                return true;
            }
        }
        "toggle_deposits" => {
            game_state.show_deposits = !game_state.show_deposits;
            return true;
        }
        "toggle_research" => {
            game_state.show_research_tree = !game_state.show_research_tree;
            return true;
        }
        "close_research_tree" => {
            game_state.show_research_tree = false;
            return true;
        }
        "decrease_tax" => {
            game_state.tax_rate = (game_state.tax_rate - 0.05).max(0.0);
            return true;
        }
        "increase_tax" => {
            game_state.tax_rate = (game_state.tax_rate + 0.05).min(1.0);
            return true;
        }
        "set_food_policy" => {
            if let Some(policy_name) = args.first() {
                use crate::types::FoodPolicy;
                game_state.food_policy = match *policy_name {
                    "balanced" => FoodPolicy::Balanced,
                    "bread" => FoodPolicy::BreadFirst,
                    "fish" => FoodPolicy::FishFirst,
                    _ => return false,
                };
                return true;
            }
        }
        "decrease_workers" => {
            if let Some(pos) = game_state.active_building_panel {
                if let Some(b) = game_state.buildings.iter_mut().find(|bb| bb.pos == pos) {
                    b.workers_target = (b.workers_target - 1).max(0);
                }
                return true;
            }
        }
        "increase_workers" => {
            if let Some(pos) = game_state.active_building_panel {
                if let Some(b) = game_state.buildings.iter_mut().find(|bb| bb.pos == pos) {
                    b.workers_target = (b.workers_target + 1).min(9);
                }
                return true;
            }
        }
        "demolish_building" => {
            if let Some(pos) = game_state.active_building_panel {
                if let Some(idx) = game_state.buildings.iter().position(|bb| bb.pos == pos) {
                    let b = game_state.buildings.remove(idx);
                    game_state.world.occupied.remove(&(pos.x, pos.y));
                    
                    // Возврат половины стоимости
                    let cost = types::building_cost(b.kind);
                    game_state.resources.wood += (cost.wood as f32 * 0.5).round() as i32;
                    game_state.resources.gold += (cost.gold as f32 * 0.5).round() as i32;
                    
                    game_state.buildings_dirty = true;
                    game_state.active_building_panel = None;
                }
                return true;
            }
        }
        _ => {}
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ui_manager_creation() {
        let manager = UIMarkupManager::new(800.0, 600.0);
        assert!(!manager.initialized);
    }
}
