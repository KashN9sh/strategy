use winit::keyboard::{KeyCode, PhysicalKey};
use rand::rngs::StdRng;
use crate::types::{Resources, WeatherKind};
use crate::world::World;
use crate::weather::WeatherSystem;

/// Консоль разработчика для отладки и управления игрой
pub struct DeveloperConsole {
    pub open: bool,
    pub input: String,
    pub log: Vec<String>,
}

impl DeveloperConsole {
    /// Создать новую консоль
    pub fn new() -> Self {
        Self {
            open: false,
            input: String::new(),
            log: vec!["Console: type 'help' for commands".to_string()],
        }
    }

    /// Переключить состояние консоли (открыть/закрыть)
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Обработать нажатие клавиши в консоли
    /// Возвращает true, если консоль должна перехватить ввод
    pub fn handle_key(&mut self, key: PhysicalKey) -> bool {
        if !self.open {
            // Проверка на тоггл консоли
            if let PhysicalKey::Code(KeyCode::Slash) = key {
                self.toggle();
                return true;
            }
            return false;
        }

        // В консоли обрабатываем ввод
        match key {
            PhysicalKey::Code(KeyCode::Enter) => {
                // Выполнение команды обрабатывается отдельно
                return true;
            }
            PhysicalKey::Code(KeyCode::Backspace) => {
                self.input.pop();
                return true;
            }
            PhysicalKey::Code(KeyCode::Escape) => {
                self.open = false;
                return true;
            }
            // Небольшой набор ASCII: буквы/цифры/символы — добавляем в строку ввода
            PhysicalKey::Code(KeyCode::Space) => { self.input.push(' '); return true; }
            PhysicalKey::Code(KeyCode::Digit0) => { self.input.push('0'); return true; }
            PhysicalKey::Code(KeyCode::Digit1) => { self.input.push('1'); return true; }
            PhysicalKey::Code(KeyCode::Digit2) => { self.input.push('2'); return true; }
            PhysicalKey::Code(KeyCode::Digit3) => { self.input.push('3'); return true; }
            PhysicalKey::Code(KeyCode::Digit4) => { self.input.push('4'); return true; }
            PhysicalKey::Code(KeyCode::Digit5) => { self.input.push('5'); return true; }
            PhysicalKey::Code(KeyCode::Digit6) => { self.input.push('6'); return true; }
            PhysicalKey::Code(KeyCode::Digit7) => { self.input.push('7'); return true; }
            PhysicalKey::Code(KeyCode::Digit8) => { self.input.push('8'); return true; }
            PhysicalKey::Code(KeyCode::Digit9) => { self.input.push('9'); return true; }
            PhysicalKey::Code(KeyCode::Minus) => { self.input.push('-'); return true; }
            PhysicalKey::Code(KeyCode::Equal) => { self.input.push('='); return true; }
            PhysicalKey::Code(KeyCode::Comma) => { self.input.push(','); return true; }
            PhysicalKey::Code(KeyCode::Period) => { self.input.push('.'); return true; }
            PhysicalKey::Code(KeyCode::Slash) => { self.input.push('/'); return true; }
            PhysicalKey::Code(KeyCode::Backslash) => { self.input.push('\\'); return true; }
            PhysicalKey::Code(KeyCode::KeyA) => { self.input.push('a'); return true; }
            PhysicalKey::Code(KeyCode::KeyB) => { self.input.push('b'); return true; }
            PhysicalKey::Code(KeyCode::KeyC) => { self.input.push('c'); return true; }
            PhysicalKey::Code(KeyCode::KeyD) => { self.input.push('d'); return true; }
            PhysicalKey::Code(KeyCode::KeyE) => { self.input.push('e'); return true; }
            PhysicalKey::Code(KeyCode::KeyF) => { self.input.push('f'); return true; }
            PhysicalKey::Code(KeyCode::KeyG) => { self.input.push('g'); return true; }
            PhysicalKey::Code(KeyCode::KeyH) => { self.input.push('h'); return true; }
            PhysicalKey::Code(KeyCode::KeyI) => { self.input.push('i'); return true; }
            PhysicalKey::Code(KeyCode::KeyJ) => { self.input.push('j'); return true; }
            PhysicalKey::Code(KeyCode::KeyK) => { self.input.push('k'); return true; }
            PhysicalKey::Code(KeyCode::KeyL) => { self.input.push('l'); return true; }
            PhysicalKey::Code(KeyCode::KeyM) => { self.input.push('m'); return true; }
            PhysicalKey::Code(KeyCode::KeyN) => { self.input.push('n'); return true; }
            PhysicalKey::Code(KeyCode::KeyO) => { self.input.push('o'); return true; }
            PhysicalKey::Code(KeyCode::KeyP) => { self.input.push('p'); return true; }
            PhysicalKey::Code(KeyCode::KeyQ) => { self.input.push('q'); return true; }
            PhysicalKey::Code(KeyCode::KeyR) => { self.input.push('r'); return true; }
            PhysicalKey::Code(KeyCode::KeyS) => { self.input.push('s'); return true; }
            PhysicalKey::Code(KeyCode::KeyT) => { self.input.push('t'); return true; }
            PhysicalKey::Code(KeyCode::KeyU) => { self.input.push('u'); return true; }
            PhysicalKey::Code(KeyCode::KeyV) => { self.input.push('v'); return true; }
            PhysicalKey::Code(KeyCode::KeyW) => { self.input.push('w'); return true; }
            PhysicalKey::Code(KeyCode::KeyX) => { self.input.push('x'); return true; }
            PhysicalKey::Code(KeyCode::KeyY) => { self.input.push('y'); return true; }
            PhysicalKey::Code(KeyCode::KeyZ) => { self.input.push('z'); return true; }
            _ => {}
        }
        false
    }

    /// Выполнить команду консоли
    pub fn execute_command(
        &mut self,
        cmd: &str,
        resources: &mut Resources,
        weather_system: &mut WeatherSystem,
        world_clock_ms: &mut f32,
        world: &mut World,
        biome_overlay_debug: &mut bool,
        biome_debug_mode: &mut bool,
        show_deposits: &mut bool,
        rng: &mut StdRng,
    ) {
        if cmd.trim().is_empty() {
            return;
        }

        self.log.push(format!("> {}", cmd));

        let trimmed = cmd.trim();
        let mut parts = trimmed.split_whitespace();
        let Some(head) = parts.next() else { return; };

        match head.to_ascii_lowercase().as_str() {
            "help" => {
                self.log.push("Commands: help, weather <clear|rain|fog|snow>, gold <±N>, set gold <N>, time <day|night|dawn|dusk|<0..1>>, biome <swamp_thr rocky_thr|overlay>, biome-overlay, debug, deposits".to_string());
            }
            "debug" => {
                *biome_debug_mode = !*biome_debug_mode;
                self.log.push(format!("Debug mode: {}", if *biome_debug_mode { "ON" } else { "OFF" }));
                self.log.push(format!("Current weather: {:?}", weather_system.current()));
                self.log.push(format!("Weather intensity: {}", weather_system.intensity()));
            }
            "deposits" => {
                *show_deposits = !*show_deposits;
                self.log.push(format!("Resource deposits: {}", if *show_deposits { "ON" } else { "OFF" }));
            }
            "weather" => {
                if let Some(arg) = parts.next() {
                    let nw = match arg.to_ascii_lowercase().as_str() {
                        "clear" => Some(WeatherKind::Clear),
                        "rain" => Some(WeatherKind::Rain),
                        "fog" => Some(WeatherKind::Fog),
                        "snow" => Some(WeatherKind::Snow),
                        _ => None,
                    };
                    if let Some(w) = nw {
                        weather_system.set(w, rng);
                        self.log.push(format!("OK: weather set to {}", arg));
                    } else {
                        self.log.push("ERR: usage weather <clear|rain|fog|snow>".to_string());
                    }
                } else {
                    self.log.push("ERR: usage weather <clear|rain|fog|snow>".to_string());
                }
            }
            "gold" => {
                if let Some(arg) = parts.next() {
                    if let Ok(delta) = arg.parse::<i32>() {
                        resources.gold = resources.gold.saturating_add(delta);
                        self.log.push(format!("OK: gold += {} -> {}", delta, resources.gold));
                    } else {
                        self.log.push("ERR: usage gold <±N>".to_string());
                    }
                } else {
                    self.log.push("ERR: usage gold <±N>".to_string());
                }
            }
            "set" => {
                let Some(what) = parts.next() else {
                    self.log.push("ERR: usage set gold <N>".to_string());
                    return;
                };
                match what.to_ascii_lowercase().as_str() {
                    "gold" => {
                        if let Some(arg) = parts.next() {
                            if let Ok(val) = arg.parse::<i32>() {
                                resources.gold = val;
                                self.log.push(format!("OK: gold = {}", resources.gold));
                            } else {
                                self.log.push("ERR: usage set gold <N>".to_string());
                            }
                        } else {
                            self.log.push("ERR: usage set gold <N>".to_string());
                        }
                    }
                    _ => self.log.push("ERR: unknown 'set' target".to_string()),
                }
            }
            "time" => {
                if let Some(arg) = parts.next() {
                    // day=~0.5, night=~0.0, dawn≈0.25, dusk≈0.75; также принимаем число 0..1
                    let t_opt: Option<f32> = match arg.to_ascii_lowercase().as_str() {
                        "day" => Some(0.5),
                        "night" => Some(0.0),
                        "dawn" => Some(0.25),
                        "dusk" => Some(0.75),
                        other => other.parse::<f32>().ok().map(|v| v.clamp(0.0, 1.0)),
                    };
                    if let Some(v) = t_opt {
                        *world_clock_ms = v * 120_000.0;
                        self.log.push(format!("OK: time set to {:.2}", v));
                    } else {
                        self.log.push("ERR: usage time <day|night|dawn|dusk|0..1>".to_string());
                    }
                } else {
                    self.log.push("ERR: usage time <day|night|dawn|dusk|0..1>".to_string());
                }
            }
            "biome" => {
                if let Some(arg) = parts.next() {
                    if arg.eq_ignore_ascii_case("overlay") {
                        *biome_overlay_debug = !*biome_overlay_debug;
                        self.log.push(format!("OK: biome overlay {}", if *biome_overlay_debug {"ON"} else {"OFF"}));
                    } else if let Some(arg2) = parts.next() {
                        if let (Ok(sw), Ok(rk)) = (arg.parse::<f32>(), arg2.parse::<f32>()) {
                            world.biome_swamp_thr = sw;
                            world.biome_rocky_thr = rk;
                            world.biomes.clear(); // сбросим кэш
                            self.log.push(format!("OK: biome thresholds set swamp_thr={:.2} rocky_thr={:.2}", sw, rk));
                        } else {
                            self.log.push("ERR: usage biome <swamp_thr rocky_thr|overlay>".to_string());
                        }
                    } else {
                        self.log.push("ERR: usage biome <swamp_thr rocky_thr|overlay>".to_string());
                    }
                } else {
                    self.log.push("ERR: usage biome <swamp_thr rocky_thr|overlay>".to_string());
                }
            }
            "biome_overlay" | "biomeoverlay" | "biome-overlay" => {
                *biome_overlay_debug = !*biome_overlay_debug;
                self.log.push(format!("OK: biome overlay {}", if *biome_overlay_debug {"ON"} else {"OFF"}));
            }
            _ => {
                self.log.push("ERR: unknown command. Type 'help'".to_string());
            }
        }
    }
}

