use crate::gpu_renderer::GpuRenderer;

/// Опции главного меню
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuOption {
    NewGame,
    LoadGame,
    Settings,
    Quit,
}

/// Состояние главного меню
pub struct MainMenu {
    pub selected_option: MenuOption,
    pub hovered_option: Option<MenuOption>,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            selected_option: MenuOption::NewGame,
            hovered_option: None,
        }
    }
    
    /// Обработка наведения мыши
    pub fn handle_hover(&mut self, x: i32, y: i32, width: i32, height: i32, base_scale: f32) {
        let scale = crate::ui::ui_scale(height, base_scale) as f32;
        let center_x = width as f32 / 2.0;
        let start_y = height as f32 / 2.0 - 100.0 * scale;
        let btn_height = 40.0 * scale;
        let btn_spacing = 50.0 * scale;
        
        let options = [
            MenuOption::NewGame,
            MenuOption::LoadGame,
            MenuOption::Settings,
            MenuOption::Quit,
        ];
        
        self.hovered_option = None;
        for (i, &option) in options.iter().enumerate() {
            let btn_y = start_y + (i as f32 * btn_spacing);
            let btn_x = center_x - 150.0 * scale;
            let btn_w = 300.0 * scale;
            
            if x as f32 >= btn_x && x as f32 <= btn_x + btn_w &&
               y as f32 >= btn_y && y as f32 <= btn_y + btn_height {
                self.hovered_option = Some(option);
                break;
            }
        }
    }
    
    /// Обработка навигации по меню (клавиатура)
    pub fn handle_key(&mut self, key: winit::keyboard::PhysicalKey) -> Option<MenuAction> {
        use winit::keyboard::{PhysicalKey, KeyCode};
        
        match key {
            PhysicalKey::Code(KeyCode::ArrowUp) | PhysicalKey::Code(KeyCode::KeyW) => {
                self.selected_option = match self.selected_option {
                    MenuOption::NewGame => MenuOption::Quit,
                    MenuOption::LoadGame => MenuOption::NewGame,
                    MenuOption::Settings => MenuOption::LoadGame,
                    MenuOption::Quit => MenuOption::Settings,
                };
                None
            }
            PhysicalKey::Code(KeyCode::ArrowDown) | PhysicalKey::Code(KeyCode::KeyS) => {
                self.selected_option = match self.selected_option {
                    MenuOption::NewGame => MenuOption::LoadGame,
                    MenuOption::LoadGame => MenuOption::Settings,
                    MenuOption::Settings => MenuOption::Quit,
                    MenuOption::Quit => MenuOption::NewGame,
                };
                None
            }
            PhysicalKey::Code(KeyCode::Enter) | PhysicalKey::Code(KeyCode::Space) => {
                Some(self.selected_option.into())
            }
            PhysicalKey::Code(KeyCode::Escape) => {
                Some(MenuAction::Quit)
            }
            _ => None,
        }
    }
    
    /// Обработка клика мыши
    pub fn handle_click(&mut self, x: i32, y: i32, width: i32, height: i32, base_scale: f32) -> Option<MenuAction> {
        let scale = crate::ui::ui_scale(height, base_scale) as f32;
        let center_x = width as f32 / 2.0;
        let start_y = height as f32 / 2.0 - 100.0 * scale;
        let btn_height = 40.0 * scale;
        let btn_spacing = 50.0 * scale;
        
        let options = [
            MenuOption::NewGame,
            MenuOption::LoadGame,
            MenuOption::Settings,
            MenuOption::Quit,
        ];
        
        for (i, &option) in options.iter().enumerate() {
            let btn_y = start_y + (i as f32 * btn_spacing);
            let btn_x = center_x - 150.0 * scale;
            let btn_w = 300.0 * scale;
            
            if x as f32 >= btn_x && x as f32 <= btn_x + btn_w &&
               y as f32 >= btn_y && y as f32 <= btn_y + btn_height {
                self.selected_option = option;
                return Some(option.into());
            }
        }
        
        None
    }
}

/// Действие, выбранное в меню
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    NewGame,
    LoadGame,
    Settings,
    Quit,
}

impl From<MenuOption> for MenuAction {
    fn from(option: MenuOption) -> Self {
        match option {
            MenuOption::NewGame => MenuAction::NewGame,
            MenuOption::LoadGame => MenuAction::LoadGame,
            MenuOption::Settings => MenuAction::Settings,
            MenuOption::Quit => MenuAction::Quit,
        }
    }
}

/// Рендеринг главного меню
pub fn draw_main_menu(
    gpu: &mut GpuRenderer,
    width: i32,
    height: i32,
    menu: &MainMenu,
    base_scale: f32,
) {
    let scale = crate::ui::ui_scale(height, base_scale) as f32;
    let center_x = width as f32 / 2.0;
    let start_y = height as f32 / 2.0 - 100.0 * scale;
    let btn_height = 40.0 * scale;
    let btn_spacing = 50.0 * scale;
    
    // Фон меню (полупрозрачный черный)
    gpu.add_ui_rect(0.0, 0.0, width as f32, height as f32, [0.0, 0.0, 0.0, 0.8]);
    
    // Заголовок игры
    let title = b"Strategy Game";
    let title_w = title.len() as f32 * 4.0 * 2.0 * scale;
    let title_x = center_x - title_w / 2.0;
    let title_y = start_y - 80.0 * scale;
    gpu.draw_text(title_x, title_y, title, [1.0, 1.0, 0.8, 1.0], scale * 1.5);
    
    // Опции меню
    let options: &[(MenuOption, &[u8])] = &[
        (MenuOption::NewGame, b"New Game"),
        (MenuOption::LoadGame, b"Load Game"),
        (MenuOption::Settings, b"Settings"),
        (MenuOption::Quit, b"Quit"),
    ];
    
    for (i, (option, label)) in options.iter().enumerate() {
        let btn_y = start_y + (i as f32 * btn_spacing);
        let btn_x = center_x - 150.0 * scale;
        let btn_w = 300.0 * scale;
        
        // Если мышь наведена на кнопку, подсвечиваем только её, иначе подсвечиваем выбранную клавиатурой
        let is_selected = if let Some(hovered) = menu.hovered_option {
            *option == hovered
        } else {
            *option == menu.selected_option
        };
        
        // Фон кнопки
        let bg_color = if is_selected {
            [185.0/255.0, 140.0/255.0, 95.0/255.0, 220.0/255.0]
        } else {
            [140.0/255.0, 105.0/255.0, 75.0/255.0, 180.0/255.0]
        };
        
        gpu.add_ui_rect(btn_x, btn_y, btn_w, btn_height, bg_color);
        
        // Верхний блик
        let band = (2.0 * scale).max(2.0);
        gpu.add_ui_rect(btn_x, btn_y, btn_w, band, [1.0, 1.0, 1.0, 0.27]);
        
        // Нижняя тень
        gpu.add_ui_rect(btn_x, btn_y + btn_height - band, btn_w, band, [0.0, 0.0, 0.0, 0.23]);
        
        // Текст кнопки
        let text_w = label.len() as f32 * 4.0 * 2.0 * scale;
        let text_x = btn_x + (btn_w - text_w) / 2.0;
        let text_y = btn_y + (btn_height - 5.0 * 2.0 * scale) / 2.0;
        
        let text_color = if is_selected {
            [1.0, 1.0, 0.9, 1.0]
        } else {
            [220.0/255.0, 220.0/255.0, 220.0/255.0, 1.0]
        };
        
        gpu.draw_text(text_x, text_y, *label, text_color, scale as f32);
    }
}

