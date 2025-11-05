use winit::keyboard::KeyCode;
use winit::event_loop::EventLoopWindowTarget;
use crate::camera::Camera;
use crate::game_state::GameState;
use crate::input::ResolvedInput;
use crate::input::Config;
use crate::save;
use crate::gpu_renderer::GpuRenderer;

/// Trait для команд - инкапсулирует действие, которое можно выполнить
pub trait Command {
    /// Выполнить команду
    /// Возвращает true, если событие обработано и нужно остановить дальнейшую обработку
    fn execute(
        &self,
        game_state: &mut GameState,
        camera: &mut Camera,
        elwt: &EventLoopWindowTarget<()>,
        input: &ResolvedInput,
        config: &Config,
        gpu_renderer: &mut GpuRenderer,
    ) -> bool;
    
    /// Проверить, можно ли выполнить команду
    /// Может быть полезно для валидации команд перед выполнением
    #[allow(dead_code)]
    fn can_execute(&self, game_state: &GameState, _input: &ResolvedInput) -> bool {
        let _ = game_state;
        true
    }
}

/// Команда выхода из игры
pub struct ExitCommand;

impl Command for ExitCommand {
    fn execute(
        &self,
        _game_state: &mut GameState,
        _camera: &mut Camera,
        elwt: &EventLoopWindowTarget<()>,
        _input: &ResolvedInput,
        _config: &Config,
        _gpu_renderer: &mut GpuRenderer,
    ) -> bool {
        elwt.exit();
        true
    }
}

/// Команда движения камеры
pub struct MoveCameraCommand {
    pub dx: f32,
    pub dy: f32,
}

impl MoveCameraCommand {
    pub fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }
}

impl Command for MoveCameraCommand {
    fn execute(
        &self,
        _game_state: &mut GameState,
        camera: &mut Camera,
        _elwt: &EventLoopWindowTarget<()>,
        _input: &ResolvedInput,
        _config: &Config,
        _gpu_renderer: &mut GpuRenderer,
    ) -> bool {
        camera.move_by(self.dx, self.dy);
        false // Не останавливаем обработку других событий
    }
}

/// Команда зума камеры
pub struct ZoomCameraCommand {
    pub factor: f32,
}

impl ZoomCameraCommand {
    pub fn new(factor: f32) -> Self {
        Self { factor }
    }
}

impl Command for ZoomCameraCommand {
    fn execute(
        &self,
        _game_state: &mut GameState,
        camera: &mut Camera,
        _elwt: &EventLoopWindowTarget<()>,
        _input: &ResolvedInput,
        _config: &Config,
        _gpu_renderer: &mut GpuRenderer,
    ) -> bool {
        camera.zoom_by_factor(self.factor, 0.5, 8.0);
        false
    }
}

/// Команда переключения паузы
pub struct TogglePauseCommand;

impl Command for TogglePauseCommand {
    fn execute(
        &self,
        game_state: &mut GameState,
        _camera: &mut Camera,
        _elwt: &EventLoopWindowTarget<()>,
        _input: &ResolvedInput,
        _config: &Config,
        _gpu_renderer: &mut GpuRenderer,
    ) -> bool {
        game_state.paused = !game_state.paused;
        false
    }
}

/// Команда изменения налогов
pub struct ChangeTaxCommand {
    pub delta: f32,
}

impl ChangeTaxCommand {
    pub fn new(delta: f32) -> Self {
        Self { delta }
    }
}

impl Command for ChangeTaxCommand {
    fn execute(
        &self,
        game_state: &mut GameState,
        _camera: &mut Camera,
        _elwt: &EventLoopWindowTarget<()>,
        _input: &ResolvedInput,
        config: &Config,
        _gpu_renderer: &mut GpuRenderer,
    ) -> bool {
        game_state.tax_rate = (game_state.tax_rate + self.delta * config.tax_step)
            .min(config.tax_max)
            .max(config.tax_min);
        false
    }
}

/// Команда сохранения игры
pub struct SaveGameCommand;

impl Command for SaveGameCommand {
    fn execute(
        &self,
        game_state: &mut GameState,
        camera: &mut Camera,
        _elwt: &EventLoopWindowTarget<()>,
        _input: &ResolvedInput,
        _config: &Config,
        _gpu_renderer: &mut GpuRenderer,
    ) -> bool {
        let _ = save::save_game(&save::SaveData::from_runtime(
            game_state.seed,
            &game_state.resources,
            &game_state.buildings,
            camera.pos,
            camera.zoom,
            &game_state.world,
        ));
        false
    }
}

/// Команда загрузки игры
pub struct LoadGameCommand;

impl Command for LoadGameCommand {
    fn execute(
        &self,
        game_state: &mut GameState,
        camera: &mut Camera,
        _elwt: &EventLoopWindowTarget<()>,
        _input: &ResolvedInput,
        _config: &Config,
        _gpu_renderer: &mut GpuRenderer,
    ) -> bool {
        if let Ok(save) = save::load_game() {
            game_state.seed = save.seed;
            game_state.world.reset_noise(game_state.seed);
            game_state.buildings = save.to_buildings();
            game_state.buildings_dirty = true;
            game_state.citizens.clear();
            game_state.population = 0;
            game_state.resources = save.resources;
            camera.pos = glam::Vec2::new(save.cam_x, save.cam_y);
            camera.zoom = save.zoom;
            // восстановим отметку occupied
            game_state.world.occupied.clear();
            for b in &game_state.buildings {
                game_state.world.occupy(b.pos);
            }
            // восстановим деревья
            game_state.world.trees.clear();
            game_state.world.removed_trees.clear();
            for t in &save.trees {
                game_state.world.trees.insert((t.x, t.y), crate::world::Tree {
                    stage: t.stage,
                    age_ms: t.age_ms,
                });
            }
        }
        false
    }
}

/// Менеджер команд - регистрирует команды по клавишам
pub struct CommandManager {
    commands: std::collections::HashMap<KeyCode, Box<dyn Command>>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: std::collections::HashMap::new(),
        }
    }
    
    /// Зарегистрировать команду для клавиши
    pub fn register<C: Command + 'static>(&mut self, key: KeyCode, command: C) {
        self.commands.insert(key, Box::new(command));
    }
    
    /// Выполнить команду для клавиши, если она зарегистрирована
    pub fn execute(
        &self,
        key: KeyCode,
        game_state: &mut GameState,
        camera: &mut Camera,
        elwt: &EventLoopWindowTarget<()>,
        input: &ResolvedInput,
        config: &Config,
        gpu_renderer: &mut GpuRenderer,
    ) -> Option<bool> {
        self.commands.get(&key).map(|cmd| {
            cmd.execute(game_state, camera, elwt, input, config, gpu_renderer)
        })
    }
    
    /// Создать менеджер команд с предустановленными командами
    pub fn create_default(input: &ResolvedInput) -> Self {
        let mut manager = Self::new();
        
        // Регистрируем команды по клавишам из input
        manager.register(input.move_up, MoveCameraCommand::new(0.0, -80.0));
        manager.register(input.move_down, MoveCameraCommand::new(0.0, 80.0));
        manager.register(input.move_left, MoveCameraCommand::new(-80.0, 0.0));
        manager.register(input.move_right, MoveCameraCommand::new(80.0, 0.0));
        
        manager.register(input.zoom_out, ZoomCameraCommand::new(0.9));
        manager.register(input.zoom_in, ZoomCameraCommand::new(1.1));
        
        manager.register(input.toggle_pause, TogglePauseCommand);
        
        manager.register(input.tax_up, ChangeTaxCommand::new(1.0));
        manager.register(input.tax_down, ChangeTaxCommand::new(-1.0));
        
        manager.register(input.save_game, SaveGameCommand);
        manager.register(input.load_game, LoadGameCommand);
        
        manager
    }
}

impl Default for CommandManager {
    fn default() -> Self {
        Self::new()
    }
}

