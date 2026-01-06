use anyhow::Result;
use glam::Vec2;
mod types;
mod world;
mod atlas;
mod ui;
mod ui_gpu;
mod input;
mod config;
mod save;
mod path;
mod jobs;
mod controls;
mod ui_interaction;
mod game;
mod palette;
mod gpu_renderer;
mod weather;
mod camera;
mod console;
mod game_state;
mod event_handler;
mod game_loop;
mod render_prep;
mod building_production;
mod citizen_state;
mod resource_visitor;
mod commands;
mod music;
mod research;
mod notifications;
mod menu;
use gpu_renderer::GpuRenderer;
use menu::{MainMenu, MenuAction};
use std::time::Instant;
use rand::{rngs::StdRng, SeedableRng};
use std::sync::atomic::{AtomicI32, Ordering};
use winit::dpi::LogicalSize;
use glam::IVec2;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

static MINIMAP_CELL_PX: AtomicI32 = AtomicI32::new(0);

type ResolvedInput = input::ResolvedInput;

fn main() -> Result<()> {
    run()
}

fn run() -> Result<()> {
    use std::sync::Arc;
    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Strategy Isometric Prototype")
        .with_inner_size(LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)?);

    env_logger::init();

    let size = window.inner_size();
    let mut gpu_renderer = pollster::block_on(GpuRenderer::new(window.clone()))?;
    gpu_renderer.load_faces_texture()?;
    let (config, input) = config::load_or_create("config.toml")?;
    let input = ResolvedInput::from(&input);

    let mut camera = camera::Camera::new(Vec2::new(0.0, 0.0), 2.0);
    let mut rng_init = StdRng::seed_from_u64(42);
    let mut game_state = game_state::GameState::new(&mut rng_init, &config);
    let mut main_menu = MainMenu::new();
    let mut pause_menu = menu::PauseMenu::new();
    
    // Загрузить все текстуры
    atlas::load_textures(
        &mut game_state.atlas,
        &mut game_state.building_atlas,
        &mut game_state.tree_atlas,
        &mut game_state.props_atlas,
    );
    game_state.width_i32 = size.width as i32;
    game_state.height_i32 = size.height as i32;
    
    // Инициализировать менеджер музыки
    match music::MusicManager::new() {
        Ok(music_manager) => {
            game_state.music_manager = Some(music_manager);
        }
        Err(e) => {
            eprintln!("Не удалось инициализировать музыку: {}", e);
        }
    }

    let window = window.clone();
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    // Обрабатываем только нажатия клавиш, не отпускания
                    if event.state != winit::event::ElementState::Pressed {
                        return;
                    }
                    
                    // Обработка главного меню
                    if game_state.app_state == game_state::AppState::MainMenu {
                        if let Some(action) = main_menu.handle_key(event.physical_key) {
                            match action {
                                MenuAction::NewGame => {
                                    // Создаем новую игру
                                    let mut new_rng = StdRng::seed_from_u64(42);
                                    game_state = game_state::GameState::new(&mut new_rng, &config);
                                    // Используем текущий размер окна
                                    let current_size = window.inner_size();
                                    game_state.width_i32 = current_size.width as i32;
                                    game_state.height_i32 = current_size.height as i32;
                                    // Обновляем размер в gpu_renderer
                                    gpu_renderer.resize(current_size);
                                    atlas::load_textures(
                                        &mut game_state.atlas,
                                        &mut game_state.building_atlas,
                                        &mut game_state.tree_atlas,
                                        &mut game_state.props_atlas,
                                    );
                                    match music::MusicManager::new() {
                                        Ok(music_manager) => {
                                            game_state.music_manager = Some(music_manager);
                                        }
                                        Err(e) => {
                                            eprintln!("Не удалось инициализировать музыку: {}", e);
                                        }
                                    }
                                    game_state.app_state = game_state::AppState::Playing;
                                }
                                MenuAction::LoadGame => {
                                    // Загружаем игру
                                    if let Ok(save) = save::load_game() {
                                        let mut new_rng = StdRng::seed_from_u64(save.seed);
                                        game_state = game_state::GameState::new(&mut new_rng, &config);
                                        // Используем текущий размер окна
                                        let current_size = window.inner_size();
                                        game_state.width_i32 = current_size.width as i32;
                                        game_state.height_i32 = current_size.height as i32;
                                        // Обновляем размер в gpu_renderer
                                        gpu_renderer.resize(current_size);
                                        
                                        // Восстанавливаем состояние из сохранения
                                        game_state.seed = save.seed;
                                        game_state.world.reset_noise(save.seed);
                                        game_state.buildings = save.to_buildings();
                                        game_state.buildings_dirty = true;
                                        game_state.resources = save.resources;
                                        camera.pos = glam::Vec2::new(save.cam_x, save.cam_y);
                                        camera.zoom = save.zoom;
                                        
                                        // Восстанавливаем граждан
                                        game_state.citizens = save.citizens;
                                        
                                        // Восстанавливаем работы
                                        game_state.jobs = save.jobs;
                                        game_state.next_job_id = save.next_job_id;
                                        
                                        // Восстанавливаем поленья на земле
                                        game_state.logs_on_ground = save.logs_on_ground;
                                        
                                        // Восстанавливаем склады
                                        game_state.warehouses = save.warehouses;
                                        
                                        // Восстанавливаем население
                                        game_state.population = save.population;
                                        
                                        // Восстанавливаем время игры
                                        game_state.world_clock_ms = save.world_clock_ms;
                                        
                                        // Восстанавливаем экономические параметры
                                        game_state.tax_rate = save.tax_rate;
                                        game_state.speed_mult = save.speed_mult;
                                        game_state.food_policy = save.food_policy;
                                        
                                        // Восстанавливаем занятые клетки
                                        game_state.world.occupied.clear();
                                        for b in &game_state.buildings {
                                            game_state.world.occupy(b.pos);
                                        }
                                        
                                        // Восстанавливаем деревья
                                        game_state.world.trees.clear();
                                        game_state.world.removed_trees.clear();
                                        for t in &save.trees {
                                            game_state.world.trees.insert((t.x, t.y), crate::world::Tree {
                                                stage: t.stage,
                                                age_ms: t.age_ms,
                                            });
                                        }
                                        
                                        // Восстанавливаем туман войны (разведанные тайлы)
                                        game_state.world.explored_tiles.clear();
                                        for &(x, y) in &save.explored_tiles {
                                            game_state.world.explored_tiles.insert((x, y));
                                        }
                                        
                                        // Восстанавливаем дороги
                                        game_state.world.roads.clear();
                                        for &(x, y) in &save.roads {
                                            game_state.world.roads.insert((x, y));
                                        }
                                        
                                        // Восстанавливаем системы исследований и уведомлений
                                        if let Some(research_system) = save.research_system {
                                            game_state.research_system = research_system;
                                        }
                                        if let Some(notification_system) = save.notification_system {
                                            game_state.notification_system = notification_system;
                                        }
                                        
                                        // Загружаем текстуры
                                        atlas::load_textures(
                                            &mut game_state.atlas,
                                            &mut game_state.building_atlas,
                                            &mut game_state.tree_atlas,
                                            &mut game_state.props_atlas,
                                        );
                                        
                                        // Инициализируем музыку
                                        match music::MusicManager::new() {
                                            Ok(music_manager) => {
                                                game_state.music_manager = Some(music_manager);
                                            }
                                            Err(e) => {
                                                eprintln!("Не удалось инициализировать музыку: {}", e);
                                            }
                                        }
                                        
                                        game_state.app_state = game_state::AppState::Playing;
                                    } else {
                                        eprintln!("Не удалось загрузить игру: файл save.json не найден или поврежден");
                                    }
                                }
                                MenuAction::Settings => {
                                    // TODO: Реализовать настройки
                                    eprintln!("Настройки пока не реализованы");
                                }
                                MenuAction::Quit => elwt.exit(),
                            }
                        }
                        return;
                    }
                    
                    // Обработка меню паузы
                    if game_state.app_state == game_state::AppState::Paused {
                        use menu::PauseMenuAction;
                        if let Some(action) = pause_menu.handle_key(event.physical_key) {
                            match action {
                                PauseMenuAction::Resume => {
                                    game_state.app_state = game_state::AppState::Playing;
                                }
                                PauseMenuAction::SaveGame => {
                                    // Сохраняем игру
                                    let save_data = save::SaveData::from_runtime(
                                        game_state.seed,
                                        &game_state.resources,
                                        &game_state.buildings,
                                        camera.pos,
                                        camera.zoom,
                                        &game_state.world,
                                        &game_state.research_system,
                                        &game_state.notification_system,
                                        &game_state.citizens,
                                        &game_state.jobs,
                                        game_state.next_job_id,
                                        &game_state.logs_on_ground,
                                        &game_state.warehouses,
                                        game_state.population,
                                        game_state.world_clock_ms,
                                        game_state.tax_rate,
                                        game_state.speed_mult,
                                        game_state.food_policy,
                                    );
                                    match save::save_game(&save_data) {
                                        Ok(_) => {
                                            eprintln!("Игра успешно сохранена");
                                            pause_menu.set_save_message("Game saved!".to_string());
                                        }
                                        Err(e) => {
                                            eprintln!("Ошибка при сохранении игры: {}", e);
                                            pause_menu.set_save_message(format!("Save error: {}", e));
                                        }
                                    }
                                }
                                PauseMenuAction::Settings => {
                                    // TODO: Реализовать настройки
                                    eprintln!("Настройки пока не реализованы");
                                }
                                PauseMenuAction::QuitToMenu => {
                                    game_state.app_state = game_state::AppState::MainMenu;
                                }
                            }
                        }
                        return;
                    }
                    
                    // Обработка событий клавиатуры в игре (только если мы в состоянии Playing)
                    if game_state.app_state == game_state::AppState::Playing {
                        if event_handler::handle_keyboard_input(
                            event.physical_key,
                            &event.state,
                            elwt,
                            &mut game_state,
                            &mut camera,
                            &input,
                            &config,
                            &mut gpu_renderer,
                        ) {
                            return;
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    // Обновляем позицию курсора для меню тоже
                    game_state.cursor_xy = IVec2::new(position.x as i32, position.y as i32);
                    
                    if game_state.app_state == game_state::AppState::MainMenu {
                        main_menu.handle_hover(
                            game_state.cursor_xy.x,
                            game_state.cursor_xy.y,
                            game_state.width_i32,
                            game_state.height_i32,
                            config.ui_scale_base,
                        );
                    } else if game_state.app_state == game_state::AppState::Paused {
                        pause_menu.handle_hover(
                            game_state.cursor_xy.x,
                            game_state.cursor_xy.y,
                            game_state.width_i32,
                            game_state.height_i32,
                            config.ui_scale_base,
                        );
                    } else if game_state.app_state == game_state::AppState::Playing {
                        event_handler::handle_cursor_moved(position, &mut game_state, &camera);
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if game_state.app_state == game_state::AppState::Paused {
                        if let winit::event::MouseButton::Left = button {
                            if state == winit::event::ElementState::Pressed {
                                use menu::PauseMenuAction;
                                if let Some(action) = pause_menu.handle_click(
                                    game_state.cursor_xy.x,
                                    game_state.cursor_xy.y,
                                    game_state.width_i32,
                                    game_state.height_i32,
                                    config.ui_scale_base,
                                ) {
                                    match action {
                                        PauseMenuAction::Resume => {
                                            game_state.app_state = game_state::AppState::Playing;
                                        }
                                        PauseMenuAction::SaveGame => {
                                            // Сохраняем игру
                                            let save_data = save::SaveData::from_runtime(
                                                game_state.seed,
                                                &game_state.resources,
                                                &game_state.buildings,
                                                camera.pos,
                                                camera.zoom,
                                                &game_state.world,
                                                &game_state.research_system,
                                                &game_state.notification_system,
                                                &game_state.citizens,
                                                &game_state.jobs,
                                                game_state.next_job_id,
                                                &game_state.logs_on_ground,
                                                &game_state.warehouses,
                                                game_state.population,
                                                game_state.world_clock_ms,
                                                game_state.tax_rate,
                                                game_state.speed_mult,
                                                game_state.food_policy,
                                            );
                                            match save::save_game(&save_data) {
                                                Ok(_) => {
                                                    eprintln!("Игра успешно сохранена");
                                                    pause_menu.set_save_message("Game saved!".to_string());
                                                }
                                                Err(e) => {
                                                    eprintln!("Ошибка при сохранении игры: {}", e);
                                                    pause_menu.set_save_message(format!("Save error: {}", e));
                                                }
                                            }
                                        }
                                        PauseMenuAction::Settings => {
                                            // TODO: Реализовать настройки
                                            eprintln!("Настройки пока не реализованы");
                                        }
                                        PauseMenuAction::QuitToMenu => {
                                            game_state.app_state = game_state::AppState::MainMenu;
                                        }
                                    }
                                }
                            }
                        }
                        return;
                    }
                    
                    if game_state.app_state == game_state::AppState::MainMenu {
                        if let winit::event::MouseButton::Left = button {
                            if state == winit::event::ElementState::Pressed {
                                if let Some(action) = main_menu.handle_click(
                                    game_state.cursor_xy.x,
                                    game_state.cursor_xy.y,
                                    game_state.width_i32,
                                    game_state.height_i32,
                                    config.ui_scale_base,
                                ) {
                                    match action {
                                        MenuAction::NewGame => {
                                            let mut new_rng = StdRng::seed_from_u64(42);
                                            game_state = game_state::GameState::new(&mut new_rng, &config);
                                            // Используем текущий размер окна
                                            let current_size = window.inner_size();
                                            game_state.width_i32 = current_size.width as i32;
                                            game_state.height_i32 = current_size.height as i32;
                                            // Обновляем размер в gpu_renderer
                                            gpu_renderer.resize(current_size);
                                            atlas::load_textures(
                                                &mut game_state.atlas,
                                                &mut game_state.building_atlas,
                                                &mut game_state.tree_atlas,
                                                &mut game_state.props_atlas,
                                            );
                                            match music::MusicManager::new() {
                                                Ok(music_manager) => {
                                                    game_state.music_manager = Some(music_manager);
                                                }
                                                Err(e) => {
                                                    eprintln!("Не удалось инициализировать музыку: {}", e);
                                                }
                                            }
                                            game_state.app_state = game_state::AppState::Playing;
                                        }
                                        MenuAction::LoadGame => {
                                            // Загружаем игру
                                            if let Ok(save) = save::load_game() {
                                                let mut new_rng = StdRng::seed_from_u64(save.seed);
                                                game_state = game_state::GameState::new(&mut new_rng, &config);
                                                // Используем текущий размер окна
                                                let current_size = window.inner_size();
                                                game_state.width_i32 = current_size.width as i32;
                                                game_state.height_i32 = current_size.height as i32;
                                                // Обновляем размер в gpu_renderer
                                                gpu_renderer.resize(current_size);
                                                
                                                // Восстанавливаем состояние из сохранения
                                                game_state.seed = save.seed;
                                                game_state.world.reset_noise(save.seed);
                                                game_state.buildings = save.to_buildings();
                                                game_state.buildings_dirty = true;
                                                game_state.resources = save.resources;
                                                camera.pos = glam::Vec2::new(save.cam_x, save.cam_y);
                                                camera.zoom = save.zoom;
                                                
                                                // Восстанавливаем граждан
                                                game_state.citizens = save.citizens;
                                                
                                                // Восстанавливаем работы
                                                game_state.jobs = save.jobs;
                                                game_state.next_job_id = save.next_job_id;
                                                
                                                // Восстанавливаем поленья на земле
                                                game_state.logs_on_ground = save.logs_on_ground;
                                                
                                                // Восстанавливаем склады
                                                game_state.warehouses = save.warehouses;
                                                
                                                // Восстанавливаем население
                                                game_state.population = save.population;
                                                
                                                // Восстанавливаем время игры
                                                game_state.world_clock_ms = save.world_clock_ms;
                                                
                                                // Восстанавливаем экономические параметры
                                                game_state.tax_rate = save.tax_rate;
                                                game_state.speed_mult = save.speed_mult;
                                                game_state.food_policy = save.food_policy;
                                                
                                                // Восстанавливаем занятые клетки
                                                game_state.world.occupied.clear();
                                                for b in &game_state.buildings {
                                                    game_state.world.occupy(b.pos);
                                                }
                                                
                                                // Восстанавливаем деревья
                                                game_state.world.trees.clear();
                                                game_state.world.removed_trees.clear();
                                                for t in &save.trees {
                                                    game_state.world.trees.insert((t.x, t.y), crate::world::Tree {
                                                        stage: t.stage,
                                                        age_ms: t.age_ms,
                                                    });
                                                }
                                                
                                                // Восстанавливаем туман войны (разведанные тайлы)
                                                game_state.world.explored_tiles.clear();
                                                for &(x, y) in &save.explored_tiles {
                                                    game_state.world.explored_tiles.insert((x, y));
                                                }
                                                
                                                // Восстанавливаем дороги
                                                game_state.world.roads.clear();
                                                for &(x, y) in &save.roads {
                                                    game_state.world.roads.insert((x, y));
                                                }
                                                
                                                // Восстанавливаем системы исследований и уведомлений
                                                if let Some(research_system) = save.research_system {
                                                    game_state.research_system = research_system;
                                                }
                                                if let Some(notification_system) = save.notification_system {
                                                    game_state.notification_system = notification_system;
                                                }
                                                
                                                // Загружаем текстуры
                                                atlas::load_textures(
                                                    &mut game_state.atlas,
                                                    &mut game_state.building_atlas,
                                                    &mut game_state.tree_atlas,
                                                    &mut game_state.props_atlas,
                                                );
                                                
                                                // Инициализируем музыку
                                                match music::MusicManager::new() {
                                                    Ok(music_manager) => {
                                                        game_state.music_manager = Some(music_manager);
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Не удалось инициализировать музыку: {}", e);
                                                    }
                                                }
                                                
                                                game_state.app_state = game_state::AppState::Playing;
                                            } else {
                                                eprintln!("Не удалось загрузить игру: файл save.json не найден или поврежден");
                                            }
                                        }
                                        MenuAction::Settings => {
                                            eprintln!("Настройки пока не реализованы");
                                        }
                                        MenuAction::Quit => elwt.exit(),
                                    }
                                }
                            }
                        }
                        return;
                    }
                    
                    if event_handler::handle_mouse_input(button, state, &mut game_state, &config, &mut gpu_renderer) {
                        return;
                    }
                }
                WindowEvent::Resized(new_size) => {
                    event_handler::handle_resize(new_size, &mut game_state, &mut gpu_renderer);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    event_handler::handle_mouse_wheel(delta, &mut camera, &mut game_state);
                }
                WindowEvent::RedrawRequested => {
                    // Рендеринг главного меню
                    if game_state.app_state == game_state::AppState::MainMenu {
                        gpu_renderer.clear_ui();
                        menu::draw_main_menu(
                            &mut gpu_renderer,
                            game_state.width_i32,
                            game_state.height_i32,
                            &main_menu,
                            config.ui_scale_base,
                            game_state.cursor_xy.x,
                            game_state.cursor_xy.y,
                        );
                        if let Err(err) = gpu_renderer.render() {
                            eprintln!("gpu_renderer.render() failed: {err}");
                            elwt.exit();
                        }
                        return;
                    }
                    
                    if MINIMAP_CELL_PX.load(Ordering::Relaxed) == 0 {
                        let s0 = ui::ui_scale(game_state.height_i32, config.ui_scale_base);
                        MINIMAP_CELL_PX.store(3 * s0, Ordering::Relaxed);
                    }

                    let (min_tx, min_ty, max_tx, max_ty) = render_prep::prepare_rendering_data(&mut game_state, &camera, &mut gpu_renderer);
                    
                    // TODO: Реализовать GPU версию draw_debug_path для отладочного пути
                    if game_state.show_ui {
                        let visible = types::total_resources(&game_state.warehouses, &game_state.resources);
                        let stats = types::count_citizen_states(&game_state.citizens);
                        let idle = stats.idle;
                        let working = stats.working;
                        let sleeping = stats.sleeping;
                        let hauling = stats.hauling;
                        let fetching = stats.fetching;
                        let day_progress = (game_state.world_clock_ms / game_loop::DAY_LENGTH_MS).clamp(0.0, 1.0);
                        let avg_hap: f32 = if game_state.citizens.is_empty() { 50.0 } else { game_state.citizens.iter().map(|c| c.happiness as i32).sum::<i32>() as f32 / game_state.citizens.len() as f32 };
                        let pop_show = game_state.citizens.len() as i32;
                        let (wlabel, wcol) = game_state.weather_system.ui_label_and_color();
                        let hovered_building = if let Some(tp) = game_state.hovered_tile {
                            game_state.buildings.iter().find(|b| b.pos == tp).cloned()
                        } else {
                            None
                        };
                        
                        for building in &mut game_state.buildings {
                            building.is_highlighted = if let Some(ref hovered) = hovered_building {
                                building.pos == hovered.pos
                            } else {
                                false
                            };
                        }
                        
                        let hovered_button = if hovered_building.is_none() {
                            ui_interaction::get_hovered_button(
                                game_state.cursor_xy,
                                game_state.width_i32,
                                game_state.height_i32,
                                &config,
                                game_state.ui_category,
                                game_state.ui_tab,
                                game_state.paused,
                                game_state.speed_mult,
                                game_state.tax_rate,
                                game_state.food_policy,
                            )
                        } else {
                            None
                        };
                        
                        let hovered_resource = if hovered_building.is_none() && hovered_button.is_none() {
                            ui_interaction::get_hovered_resource(
                                game_state.cursor_xy,
                                game_state.width_i32,
                                game_state.height_i32,
                                &config,
                                &visible,
                                visible.wood,
                                pop_show,
                                avg_hap,
                                game_state.tax_rate,
                                idle,
                                working,
                                sleeping,
                                hauling,
                                fetching,
                                wlabel, // Передаем метку погоды для проверки наведения
                            )
                        } else {
                            None
                        };
                        
                        let intensity = game_state.weather_system.intensity();
                        gpu_renderer.update_weather(game_state.weather_system.current(), game_state.world_clock_ms / 1000.0, intensity);
                        gpu_renderer.update_building_particles(&game_state.buildings, game_state.world_clock_ms / 1000.0);
                        let wcol_f32 = [wcol[0] as f32 / 255.0, wcol[1] as f32 / 255.0, wcol[2] as f32 / 255.0, wcol[3] as f32 / 255.0];
                ui_gpu::draw_ui_gpu(
                    &mut gpu_renderer,
                    game_state.width_i32,
                    game_state.height_i32,
                    &visible,
                    visible.wood,
                    pop_show,
                    game_state.selected_building,
                    game_state.fps_ema,
                    game_state.speed_mult,
                    game_state.paused,
                    config.ui_scale_base,
                    game_state.ui_category,
                    day_progress,
                    idle,
                    working,
                    sleeping,
                    hauling,
                    fetching,
                    avg_hap,
                    game_state.tax_rate,
                    game_state.ui_tab,
                    game_state.food_policy,
                    wlabel,
                    wcol_f32,
                    game_state.weather_system.current(), // Текущая погода для тултипа
                    &mut game_state.world,
                    &game_state.buildings,
                    camera.pos.x,
                    camera.pos.y,
                    MINIMAP_CELL_PX.load(Ordering::Relaxed).max(1),
                    game_state.cursor_xy.x as f32,
                    game_state.cursor_xy.y as f32,
                    hovered_building,
                    hovered_button,
                    hovered_resource,
                    game_state.console.open,
                    &game_state.console.input,
                    &game_state.console.log,
                    game_state.biome_debug_mode,
                    game_state.show_deposits,
                    camera.zoom,
                    game_state.atlas.half_w,
                    game_state.atlas.half_h,
                    min_tx,
                    min_ty,
                    max_tx,
                    max_ty,
                    &game_state.research_system,
                );
            } else {
                gpu_renderer.clear_ui();
            }
                
                // Рендеринг окна исследований (если открыто)
                if game_state.show_research_tree {
                    let visible = types::total_resources(&game_state.warehouses, &game_state.resources);
                    ui_gpu::draw_research_tree_gpu(
                        &mut gpu_renderer,
                        game_state.width_i32,
                        game_state.height_i32,
                        &game_state.research_system,
                        &visible,
                        config.ui_scale_base,
                        game_state.cursor_xy.x,
                        game_state.cursor_xy.y,
                        game_state.research_tree_scroll,
                    );
                }
                
                // Рендеринг уведомлений
                ui_gpu::draw_notifications_gpu(
                    &mut gpu_renderer,
                    game_state.width_i32,
                    game_state.height_i32,
                    &game_state.notification_system.notifications,
                    config.ui_scale_base,
                );
                
                let t = (game_state.world_clock_ms / game_loop::DAY_LENGTH_MS).clamp(0.0, 1.0);
                let angle = t * std::f32::consts::TAU;
                let daylight = 0.5 - 0.5 * angle.cos();
                let darkness = (1.0 - daylight).max(0.0);
                let night_strength = (darkness.powf(1.4) * 180.0).min(200.0) as u8;
                let night_alpha = if night_strength > 0 {
                    night_strength as f32 / 255.0
                } else {
                    0.0
                };
                gpu_renderer.update_night_overlay(night_alpha);
                
                // Рендеринг меню паузы поверх игры (если игра на паузе)
                if game_state.app_state == game_state::AppState::Paused {
                    menu::draw_pause_menu(
                        &mut gpu_renderer,
                        game_state.width_i32,
                        game_state.height_i32,
                        &pause_menu,
                        config.ui_scale_base,
                    );
                }
                    
                if let Err(err) = gpu_renderer.render() {
                    eprintln!("gpu_renderer.render() failed: {err}");
                    elwt.exit();
                }
                }
                _ => {}
            },
            Event::AboutToWait => {
                let now = Instant::now();
                let frame_ms = (now - game_state.last_frame).as_secs_f32() * 1000.0;
                game_state.last_frame = now;
                let frame_ms = frame_ms.min(250.0);
                
                // Обновляем состояние игры только если мы в игре (не в меню и не на паузе)
                if game_state.app_state == game_state::AppState::Playing {
                    game_loop::update_game_state(&mut game_state, frame_ms, &config);
                }
                
                // Обновляем таймер сообщения в меню паузы
                if game_state.app_state == game_state::AppState::Paused {
                    pause_menu.update(frame_ms);
                }

                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
