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
    pub selected_option: Option<MenuOption>, // Единый буфер выделения для клавиатуры и мыши
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            selected_option: None, // Начинаем без выделения
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
        
        // При наведении мыши обновляем единый буфер выделения
        let mut found_hover = false;
        for (i, &option) in options.iter().enumerate() {
            let btn_y = start_y + (i as f32 * btn_spacing);
            let btn_x = center_x - 150.0 * scale;
            let btn_w = 300.0 * scale;
            
            if x as f32 >= btn_x && x as f32 <= btn_x + btn_w &&
               y as f32 >= btn_y && y as f32 <= btn_y + btn_height {
                self.selected_option = Some(option);
                found_hover = true;
                break;
            }
        }
        
        // Если мышь не на кнопке, убираем выделение
        if !found_hover {
            self.selected_option = None;
        }
    }
    
    /// Обработка навигации по меню (клавиатура)
    pub fn handle_key(&mut self, key: winit::keyboard::PhysicalKey) -> Option<MenuAction> {
        use winit::keyboard::{PhysicalKey, KeyCode};
        
        match key {
            PhysicalKey::Code(KeyCode::ArrowUp) | PhysicalKey::Code(KeyCode::KeyW) => {
                // При навигации клавиатурой используем текущее выделение как начальную точку
                // Если выделения нет, начинаем с первой опции
                let start_option = self.selected_option.unwrap_or(MenuOption::NewGame);
                // Обновляем selected_option на основе текущей позиции
                self.selected_option = Some(match start_option {
                    MenuOption::NewGame => MenuOption::Quit,
                    MenuOption::LoadGame => MenuOption::NewGame,
                    MenuOption::Settings => MenuOption::LoadGame,
                    MenuOption::Quit => MenuOption::Settings,
                });
                None
            }
            PhysicalKey::Code(KeyCode::ArrowDown) | PhysicalKey::Code(KeyCode::KeyS) => {
                // При навигации клавиатурой используем текущее выделение как начальную точку
                // Если выделения нет, начинаем с первой опции
                let start_option = self.selected_option.unwrap_or(MenuOption::NewGame);
                // Обновляем selected_option на основе текущей позиции
                self.selected_option = Some(match start_option {
                    MenuOption::NewGame => MenuOption::LoadGame,
                    MenuOption::LoadGame => MenuOption::Settings,
                    MenuOption::Settings => MenuOption::Quit,
                    MenuOption::Quit => MenuOption::NewGame,
                });
                None
            }
            PhysicalKey::Code(KeyCode::Enter) | PhysicalKey::Code(KeyCode::Space) => {
                // При выборе используем текущее выделение
                if let Some(option) = self.selected_option {
                    Some(option.into())
                } else {
                    None
                }
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
                // При клике мыши используем текущее выделение (которое уже установлено при наведении)
                return Some(option.into());
            }
        }
        
        None
    }
}

/// Опции меню паузы
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseMenuOption {
    Resume,
    SaveGame,
    Settings,
    QuitToMenu,
}

/// Состояние меню паузы
pub struct PauseMenu {
    pub selected_option: Option<PauseMenuOption>,
    pub save_message: Option<String>, // Сообщение о сохранении (показывается временно)
    pub save_message_timer: f32, // Таймер для исчезновения сообщения
}

impl PauseMenu {
    pub fn new() -> Self {
        Self {
            selected_option: None,
            save_message: None,
            save_message_timer: 0.0,
        }
    }
    
    /// Установить сообщение о сохранении
    pub fn set_save_message(&mut self, message: String) {
        self.save_message = Some(message);
        self.save_message_timer = 3000.0; // 3 секунды
    }
    
    /// Обновить таймер сообщения
    pub fn update(&mut self, delta_ms: f32) {
        if self.save_message.is_some() {
            self.save_message_timer -= delta_ms;
            if self.save_message_timer <= 0.0 {
                self.save_message = None;
            }
        }
    }
    
    /// Обработка наведения мыши
    pub fn handle_hover(&mut self, x: i32, y: i32, width: i32, height: i32, base_scale: f32) {
        let scale = crate::ui::ui_scale(height, base_scale) as f32;
        let center_x = width as f32 / 2.0;
        let start_y = height as f32 / 2.0 - 50.0 * scale;
        let btn_height = 40.0 * scale;
        let btn_spacing = 50.0 * scale;
        
        let options = [
            PauseMenuOption::Resume,
            PauseMenuOption::SaveGame,
            PauseMenuOption::Settings,
            PauseMenuOption::QuitToMenu,
        ];
        
        let mut found_hover = false;
        for (i, &option) in options.iter().enumerate() {
            let btn_y = start_y + (i as f32 * btn_spacing);
            let btn_x = center_x - 150.0 * scale;
            let btn_w = 300.0 * scale;
            
            if x as f32 >= btn_x && x as f32 <= btn_x + btn_w &&
               y as f32 >= btn_y && y as f32 <= btn_y + btn_height {
                self.selected_option = Some(option);
                found_hover = true;
                break;
            }
        }
        
        if !found_hover {
            self.selected_option = None;
        }
    }
    
    /// Обработка навигации по меню (клавиатура)
    pub fn handle_key(&mut self, key: winit::keyboard::PhysicalKey) -> Option<PauseMenuAction> {
        use winit::keyboard::{PhysicalKey, KeyCode};
        
        match key {
            PhysicalKey::Code(KeyCode::ArrowUp) | PhysicalKey::Code(KeyCode::KeyW) => {
                let start_option = self.selected_option.unwrap_or(PauseMenuOption::Resume);
                self.selected_option = Some(match start_option {
                    PauseMenuOption::Resume => PauseMenuOption::QuitToMenu,
                    PauseMenuOption::SaveGame => PauseMenuOption::Resume,
                    PauseMenuOption::Settings => PauseMenuOption::SaveGame,
                    PauseMenuOption::QuitToMenu => PauseMenuOption::Settings,
                });
                None
            }
            PhysicalKey::Code(KeyCode::ArrowDown) | PhysicalKey::Code(KeyCode::KeyS) => {
                let start_option = self.selected_option.unwrap_or(PauseMenuOption::Resume);
                self.selected_option = Some(match start_option {
                    PauseMenuOption::Resume => PauseMenuOption::SaveGame,
                    PauseMenuOption::SaveGame => PauseMenuOption::Settings,
                    PauseMenuOption::Settings => PauseMenuOption::QuitToMenu,
                    PauseMenuOption::QuitToMenu => PauseMenuOption::Resume,
                });
                None
            }
            PhysicalKey::Code(KeyCode::Enter) | PhysicalKey::Code(KeyCode::Space) => {
                if let Some(option) = self.selected_option {
                    Some(option.into())
                } else {
                    Some(PauseMenuAction::Resume) // По умолчанию Resume
                }
            }
            PhysicalKey::Code(KeyCode::Escape) => {
                Some(PauseMenuAction::Resume)
            }
            _ => None,
        }
    }
    
    /// Обработка клика мыши
    pub fn handle_click(&mut self, x: i32, y: i32, width: i32, height: i32, base_scale: f32) -> Option<PauseMenuAction> {
        let scale = crate::ui::ui_scale(height, base_scale) as f32;
        let center_x = width as f32 / 2.0;
        let start_y = height as f32 / 2.0 - 50.0 * scale;
        let btn_height = 40.0 * scale;
        let btn_spacing = 50.0 * scale;
        
        let options = [
            PauseMenuOption::Resume,
            PauseMenuOption::SaveGame,
            PauseMenuOption::Settings,
            PauseMenuOption::QuitToMenu,
        ];
        
        for (i, &option) in options.iter().enumerate() {
            let btn_y = start_y + (i as f32 * btn_spacing);
            let btn_x = center_x - 150.0 * scale;
            let btn_w = 300.0 * scale;
            
            if x as f32 >= btn_x && x as f32 <= btn_x + btn_w &&
               y as f32 >= btn_y && y as f32 <= btn_y + btn_height {
                return Some(option.into());
            }
        }
        
        None
    }
}

/// Действие, выбранное в меню паузы
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseMenuAction {
    Resume,
    SaveGame,
    Settings,
    QuitToMenu,
}

impl From<PauseMenuOption> for PauseMenuAction {
    fn from(option: PauseMenuOption) -> Self {
        match option {
            PauseMenuOption::Resume => PauseMenuAction::Resume,
            PauseMenuOption::SaveGame => PauseMenuAction::SaveGame,
            PauseMenuOption::Settings => PauseMenuAction::Settings,
            PauseMenuOption::QuitToMenu => PauseMenuAction::QuitToMenu,
        }
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

/// Рендеринг меню паузы (оверлей поверх игры)
pub fn draw_pause_menu(
    gpu: &mut GpuRenderer,
    width: i32,
    height: i32,
    menu: &PauseMenu,
    base_scale: f32,
) {
    let scale = crate::ui::ui_scale(height, base_scale) as f32;
    let center_x = width as f32 / 2.0;
    let start_y = height as f32 / 2.0 - 50.0 * scale;
    let btn_height = 40.0 * scale;
    let btn_spacing = 50.0 * scale;
    
    // Полупрозрачный оверлей поверх игры
    gpu.add_ui_rect(0.0, 0.0, width as f32, height as f32, [0.0, 0.0, 0.0, 0.7]);
    
    // Заголовок "PAUSED"
    let title = b"PAUSED";
    let title_scale = scale * 2.0;
    // Ширина текста: каждый символ занимает 4.0 * px, где px = 2.0 * scale
    // Для title_scale: px = 2.0 * title_scale, char_width = 4.0 * px = 8.0 * title_scale
    let title_w = title.len() as f32 * 8.0 * title_scale;
    let title_x = center_x - title_w / 2.0;
    let title_y = start_y - 80.0 * scale;
    gpu.draw_text(title_x, title_y, title, [1.0, 1.0, 0.8, 1.0], title_scale);
    
    // Опции меню
    let options: &[(PauseMenuOption, &[u8])] = &[
        (PauseMenuOption::Resume, b"Resume"),
        (PauseMenuOption::SaveGame, b"Save Game"),
        (PauseMenuOption::Settings, b"Settings"),
        (PauseMenuOption::QuitToMenu, b"Quit to Menu"),
    ];
    
    for (i, (option, label)) in options.iter().enumerate() {
        let btn_y = start_y + (i as f32 * btn_spacing);
        let btn_x = center_x - 150.0 * scale;
        let btn_w = 300.0 * scale;
        
        let is_selected = menu.selected_option == Some(*option);
        
        let bg_color = if is_selected {
            [185.0/255.0, 140.0/255.0, 95.0/255.0, 220.0/255.0]
        } else {
            [140.0/255.0, 105.0/255.0, 75.0/255.0, 180.0/255.0]
        };
        
        gpu.add_ui_rect(btn_x, btn_y, btn_w, btn_height, bg_color);
        
        let band = (2.0 * scale).max(2.0);
        gpu.add_ui_rect(btn_x, btn_y, btn_w, band, [1.0, 1.0, 1.0, 0.27]);
        gpu.add_ui_rect(btn_x, btn_y + btn_height - band, btn_w, band, [0.0, 0.0, 0.0, 0.23]);
        
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
    
    // Отображаем сообщение о сохранении, если есть
    if let Some(ref message) = menu.save_message {
        let alpha = (menu.save_message_timer / 1000.0).min(1.0); // Fade out в последнюю секунду
        let msg_text = message.as_bytes();
        let msg_scale = scale * 1.2;
        let msg_w = msg_text.len() as f32 * 4.0 * 2.0 * msg_scale;
        let msg_x = center_x - msg_w / 2.0;
        let msg_y = start_y + (options.len() as f32 * btn_spacing) + 30.0 * scale;
        
        // Текст сообщения без фона
        gpu.draw_text(msg_x, msg_y, msg_text, [1.0, 1.0, 0.8, alpha], msg_scale);
    }
}

/// Рендеринг главного меню
pub fn draw_main_menu(
    gpu: &mut GpuRenderer,
    width: i32,
    height: i32,
    menu: &MainMenu,
    base_scale: f32,
    cursor_x: i32,
    cursor_y: i32,
) {
    // Рендерим параллакс-фон
    draw_menu_background(gpu, width, height, cursor_x, cursor_y);
    
    let scale = crate::ui::ui_scale(height, base_scale) as f32;
    let center_x = width as f32 / 2.0;
    let start_y = height as f32 / 2.0 - 100.0 * scale;
    let btn_height = 40.0 * scale;
    let btn_spacing = 50.0 * scale;
    
    // Полупрозрачный оверлей поверх фона для лучшей читаемости
    // gpu.add_ui_rect(0.0, 0.0, width as f32, height as f32, [0.0, 0.0, 0.0, 0.3]);
    
    // Заголовок игры
    let title = b"Strategy Game";
    let title_scale = scale * 2.5; // Увеличенный размер заголовка
    let title_w = title.len() as f32 * 4.0 * 2.0 * title_scale;
    let title_x = center_x - title_w / 2.0;
    let title_y = height as f32 / 2.0 - 200.0 * scale; // Ближе к центру экрана
    gpu.draw_text(title_x, title_y, title, [1.0, 1.0, 0.8, 1.0], title_scale);
    
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
        
        // Подсвечиваем кнопку, если она выделена (единый буфер для клавиатуры и мыши)
        let is_selected = menu.selected_option == Some(*option);
        
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

/// Рендеринг параллакс-фона главного меню
fn draw_menu_background(
    gpu: &mut GpuRenderer,
    width: i32,
    height: i32,
    cursor_x: i32,
    cursor_y: i32,
) {
    // Загружаем текстуры фона при первом использовании
    gpu.ensure_menu_background_textures();
    
    // Вычисляем смещение параллакса на основе позиции курсора
    // Нормализуем позицию курсора к диапазону -1.0..1.0
    let parallax_x = (cursor_x as f32 / width as f32 - 0.5) * 2.0;
    let parallax_y = (cursor_y as f32 / height as f32 - 0.5) * 2.0;
    
    // Коэффициенты параллакса для разных слоев (дальние слои двигаются медленнее)
    let parallax_factors = [
        (0.0, 0.0),      // sky - не двигается
        (0.1, 0.1),      // far_mountains
        (0.2, 0.2),      // grassy_mountains
        (0.3, 0.3),      // hill
        (0.4, 0.4),      // clouds_mid
        (0.5, 0.5),      // clouds_mid_t
        (0.6, 0.6),      // clouds_front
        (0.7, 0.7),      // clouds_front_t
    ];
    
    // Максимальное смещение в пикселях
    let max_offset = 50.0;
    
    // Максимальный коэффициент параллакса (для самого быстрого слоя)
    let max_parallax_factor = 0.7; // CloudsFront
    
    // Увеличиваем размер слоев, чтобы покрыть экран даже при максимальном смещении параллакса
    // Максимальное смещение = max_offset * max_parallax_factor в каждую сторону
    let padding = max_offset * max_parallax_factor * 2.0; // *2 для обеих сторон
    let w = width as f32 + padding;
    let h = height as f32 + padding;
    
    // Смещение для центрирования увеличенной текстуры на экране
    // Текстура увеличена на padding, поэтому нужно сместить её влево и вверх на padding/2
    // Поскольку мы передаем -offset_x в функцию, нужно положительное значение для смещения влево
    let center_offset_x = padding * 0.5;
    let center_offset_y = padding * 0.5;
    
    // Рендерим слои от дальних к ближним: Sky -> FarMountains -> GrassyMountains -> CloudsMid -> Hill -> CloudsFront
    use crate::gpu_renderer::MenuBackgroundLayer;
    
    // Sky (самый дальний слой, не двигается)
    let offset_x = center_offset_x + parallax_x * parallax_factors[0].0 * max_offset;
    let offset_y = center_offset_y + parallax_y * parallax_factors[0].1 * max_offset;
    gpu.draw_menu_background_layer(
        MenuBackgroundLayer::Sky,
        -offset_x,
        -offset_y,
        w,
        h,
    );
    
    // FarMountains (дальний слой)
    let offset_x = center_offset_x + parallax_x * parallax_factors[1].0 * max_offset;
    let offset_y = center_offset_y + parallax_y * parallax_factors[1].1 * max_offset;
    gpu.draw_menu_background_layer(
        MenuBackgroundLayer::FarMountains,
        -offset_x,
        -offset_y,
        w,
        h,
    );
    
    // GrassyMountains (дальний слой)
    let offset_x = center_offset_x + parallax_x * parallax_factors[2].0 * max_offset;
    let offset_y = center_offset_y + parallax_y * parallax_factors[2].1 * max_offset;
    gpu.draw_menu_background_layer(
        MenuBackgroundLayer::GrassyMountains,
        -offset_x,
        -offset_y,
        w,
        h,
    );
    
    // CloudsMid (средний слой)
    let offset_x = center_offset_x + parallax_x * parallax_factors[4].0 * max_offset;
    let offset_y = center_offset_y + parallax_y * parallax_factors[4].1 * max_offset;
    gpu.draw_menu_background_layer(
        MenuBackgroundLayer::CloudsMid,
        -offset_x,
        -offset_y,
        w,
        h,
    );
    
    // Hill (ближний слой)
    let offset_x = center_offset_x + parallax_x * parallax_factors[3].0 * max_offset;
    let offset_y = center_offset_y + parallax_y * parallax_factors[3].1 * max_offset;
    gpu.draw_menu_background_layer(
        MenuBackgroundLayer::Hill,
        -offset_x,
        -offset_y,
        w,
        h,
    );
    
    // CloudsFront (самый ближний слой, перед всеми)
    let offset_x = center_offset_x + parallax_x * parallax_factors[6].0 * max_offset;
    let offset_y = center_offset_y + parallax_y * parallax_factors[6].1 * max_offset;
    gpu.draw_menu_background_layer(
        MenuBackgroundLayer::CloudsFront,
        -offset_x,
        -offset_y,
        w,
        h,
    );
}

