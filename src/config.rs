use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::input;

#[inline]
pub fn defaults() -> (input::Config, input::InputConfig) {
    let config = input::Config { base_step_ms: 33.0, ui_scale_base: 1.6 };
    let input = input::InputConfig {
        move_up: "W".into(),
        move_down: "S".into(),
        move_left: "A".into(),
        move_right: "D".into(),
        zoom_in: "E".into(),
        zoom_out: "Q".into(),
        toggle_pause: "SPACE".into(),
        speed_0_5x: "Digit1".into(),
        speed_1x: "Digit2".into(),
        speed_2x: "Digit3".into(),
        speed_3x: "Digit4".into(),
        build_lumberjack: "Z".into(),
        build_house: "X".into(),
        toggle_road_mode: "R".into(),
        reset_new_seed: "T".into(),
        reset_same_seed: "N".into(),
        save_game: "F5".into(),
        load_game: "F9".into(),
    };
    (config, input)
}

pub fn load_or_create(path: &str) -> Result<(input::Config, input::InputConfig)> {
    if Path::new(path).exists() {
        let data = fs::read_to_string(path)?;
        #[derive(Deserialize)]
        struct FileCfg {
            config: input::Config,
            input: input::InputConfig,
        }
        let parsed: FileCfg = toml::from_str(&data)?;
        Ok((parsed.config, parsed.input))
    } else {
        let (config, input) = defaults();
        #[derive(Serialize)]
        struct FileCfg<'a> {
            config: &'a input::Config,
            input: &'a input::InputConfig,
        }
        let toml_text = toml::to_string_pretty(&FileCfg { config: &config, input: &input })?;
        fs::write(path, toml_text)?;
        Ok((config, input))
    }
}


