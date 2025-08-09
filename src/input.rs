use serde::{Serialize, Deserialize};
use winit::keyboard::KeyCode;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config { pub base_step_ms: f32, pub ui_scale_base: f32 }

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct InputConfig {
    pub move_up: String,
    pub move_down: String,
    pub move_left: String,
    pub move_right: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub toggle_pause: String,
    pub speed_0_5x: String,
    pub speed_1x: String,
    pub speed_2x: String,
    pub speed_3x: String,
    pub build_lumberjack: String,
    pub build_house: String,
    pub toggle_road_mode: String,
    pub reset_new_seed: String,
    pub reset_same_seed: String,
    pub save_game: String,
    pub load_game: String,
}

pub struct ResolvedInput {
    pub move_up: KeyCode,
    pub move_down: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
    pub toggle_pause: KeyCode,
    pub speed_0_5x: KeyCode,
    pub speed_1x: KeyCode,
    pub speed_2x: KeyCode,
    pub speed_3x: KeyCode,
    pub build_lumberjack: KeyCode,
    pub build_house: KeyCode,
    pub toggle_road_mode: KeyCode,
    pub reset_new_seed: KeyCode,
    pub reset_same_seed: KeyCode,
    pub save_game: KeyCode,
    pub load_game: KeyCode,
}

fn code_from_str(s: &str) -> KeyCode {
    use KeyCode::*;
    match s.to_uppercase().as_str() {
        "W" => KeyW, "A" => KeyA, "S" => KeyS, "D" => KeyD,
        "Q" => KeyQ, "E" => KeyE, "SPACE" => Space,
        "DIGIT1" | "1" => Digit1, "DIGIT2" | "2" => Digit2, "DIGIT3" | "3" => Digit3, "DIGIT4" | "4" => Digit4,
        "Z" => KeyZ, "X" => KeyX, "R" => KeyR, "T" => KeyT, "N" => KeyN, "F5" => F5, "F9" => F9,
        _ => KeyCode::Escape,
    }
}

impl ResolvedInput {
    pub fn from(cfg: &InputConfig) -> Self {
        Self {
            move_up: code_from_str(&cfg.move_up),
            move_down: code_from_str(&cfg.move_down),
            move_left: code_from_str(&cfg.move_left),
            move_right: code_from_str(&cfg.move_right),
            zoom_in: code_from_str(&cfg.zoom_in),
            zoom_out: code_from_str(&cfg.zoom_out),
            toggle_pause: code_from_str(&cfg.toggle_pause),
            speed_0_5x: code_from_str(&cfg.speed_0_5x),
            speed_1x: code_from_str(&cfg.speed_1x),
            speed_2x: code_from_str(&cfg.speed_2x),
            speed_3x: code_from_str(&cfg.speed_3x),
            build_lumberjack: code_from_str(&cfg.build_lumberjack),
            build_house: code_from_str(&cfg.build_house),
            toggle_road_mode: code_from_str(&cfg.toggle_road_mode),
            reset_new_seed: code_from_str(&cfg.reset_new_seed),
            reset_same_seed: code_from_str(&cfg.reset_same_seed),
            save_game: code_from_str(&cfg.save_game),
            load_game: code_from_str(&cfg.load_game),
        }
    }
}


