use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::input;

#[inline]
pub fn defaults() -> (input::Config, input::InputConfig) {
    let config = input::Config {
        base_step_ms: 33.0,
        ui_scale_base: 1.6,
        tax_min: 0.0,
        tax_max: 20.0,
        tax_step: 1.0,
        happy_feed_bonus: 15,
        happy_variety_bonus: 5,
        happy_house_bonus: 10,
        happy_starving_penalty: -25,
        migration_join_threshold: 65.0,
        migration_leave_threshold: 35.0,
        tax_income_base: 0.5,
        tax_income_happy_scale: 0.5,
        tax_income_per_capita: 10.0,
        upkeep_house: 0,
        upkeep_warehouse: 0,
        upkeep_lumberjack: 1,
        upkeep_forester: 1,
        upkeep_stone_quarry: 1,
        upkeep_clay_pit: 1,
        upkeep_iron_mine: 1,
        upkeep_wheat_field: 1,
        upkeep_mill: 1,
        upkeep_bakery: 1,
        upkeep_kiln: 1,
        upkeep_fishery: 1,
        upkeep_smelter: 2,
        biome_swamp_thr: 0.10,
        biome_rocky_thr: 0.10,
        biome_swamp_lumberjack_wmul: 1.10,
        biome_rocky_stone_wmul: 0.90,
        biome_swamp_tree_growth_wmul: 0.85,
        biome_rocky_tree_growth_wmul: 1.20,
        biome_meadow_wheat_wmul: 0.95,
        biome_swamp_wheat_wmul: 1.15,
    };
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
        save_game: "F5".into(),
        load_game: "F9".into(),
        tax_up: "]".into(),
        tax_down: "[".into(),
    };
    (config, input)
}

pub fn load_or_create(path: &str) -> Result<(input::Config, input::InputConfig)> {
    use crate::resource_path;
    // Сначала пробуем загрузить из пользовательской директории
    let user_config_path = resource_path::user_data_dir().join(path);
    let config_path = if user_config_path.exists() {
        user_config_path
    } else {
        // Если нет в пользовательской директории, пробуем из ресурсов (для первого запуска)
        resource_path::resource_path(path)
    };
    
    if config_path.exists() {
        let data = fs::read_to_string(&config_path)?;
        #[derive(Deserialize)]
        struct FileCfg {
            config: input::Config,
            input: input::InputConfig,
        }
        let parsed: FileCfg = toml::from_str(&data)?;
        // Мягкая миграция старых конфигов: дополним отсутствующие поля значениями по умолчанию
        let (def_cfg, _def_input) = defaults();
        let mut cfg = parsed.config.clone();
        // если новые поля остались нулевыми — подставим дефолты
        if cfg.tax_max <= 0.0 { cfg.tax_max = def_cfg.tax_max; }
        if cfg.tax_step <= 0.0 { cfg.tax_step = def_cfg.tax_step; }
        if cfg.happy_feed_bonus == 0 { cfg.happy_feed_bonus = def_cfg.happy_feed_bonus; }
        if cfg.happy_variety_bonus == 0 { cfg.happy_variety_bonus = def_cfg.happy_variety_bonus; }
        if cfg.happy_house_bonus == 0 { cfg.happy_house_bonus = def_cfg.happy_house_bonus; }
        if cfg.happy_starving_penalty == 0 { cfg.happy_starving_penalty = def_cfg.happy_starving_penalty; }
        if cfg.migration_join_threshold == 0.0 { cfg.migration_join_threshold = def_cfg.migration_join_threshold; }
        if cfg.migration_leave_threshold == 0.0 { cfg.migration_leave_threshold = def_cfg.migration_leave_threshold; }
        if cfg.tax_income_base == 0.0 { cfg.tax_income_base = def_cfg.tax_income_base; }
        if cfg.tax_income_happy_scale == 0.0 { cfg.tax_income_happy_scale = def_cfg.tax_income_happy_scale; }
        if cfg.tax_income_per_capita == 0.0 { cfg.tax_income_per_capita = def_cfg.tax_income_per_capita; }
        // апкипы — заполняем индивидуально, если нули
        if cfg.upkeep_house == 0 { cfg.upkeep_house = def_cfg.upkeep_house; }
        if cfg.upkeep_warehouse == 0 { cfg.upkeep_warehouse = def_cfg.upkeep_warehouse; }
        if cfg.upkeep_lumberjack == 0 { cfg.upkeep_lumberjack = def_cfg.upkeep_lumberjack; }
        if cfg.upkeep_forester == 0 { cfg.upkeep_forester = def_cfg.upkeep_forester; }
        if cfg.upkeep_stone_quarry == 0 { cfg.upkeep_stone_quarry = def_cfg.upkeep_stone_quarry; }
        if cfg.upkeep_clay_pit == 0 { cfg.upkeep_clay_pit = def_cfg.upkeep_clay_pit; }
        if cfg.upkeep_iron_mine == 0 { cfg.upkeep_iron_mine = def_cfg.upkeep_iron_mine; }
        if cfg.upkeep_wheat_field == 0 { cfg.upkeep_wheat_field = def_cfg.upkeep_wheat_field; }
        if cfg.upkeep_mill == 0 { cfg.upkeep_mill = def_cfg.upkeep_mill; }
        if cfg.upkeep_bakery == 0 { cfg.upkeep_bakery = def_cfg.upkeep_bakery; }
        if cfg.upkeep_kiln == 0 { cfg.upkeep_kiln = def_cfg.upkeep_kiln; }
        if cfg.upkeep_fishery == 0 { cfg.upkeep_fishery = def_cfg.upkeep_fishery; }
        if cfg.upkeep_smelter == 0 { cfg.upkeep_smelter = def_cfg.upkeep_smelter; }
        // биомы — мягкие дефолты
        if cfg.biome_swamp_thr == 0.0 { cfg.biome_swamp_thr = def_cfg.biome_swamp_thr; }
        if cfg.biome_rocky_thr == 0.0 { cfg.biome_rocky_thr = def_cfg.biome_rocky_thr; }
        if cfg.biome_swamp_lumberjack_wmul == 0.0 { cfg.biome_swamp_lumberjack_wmul = def_cfg.biome_swamp_lumberjack_wmul; }
        if cfg.biome_rocky_stone_wmul == 0.0 { cfg.biome_rocky_stone_wmul = def_cfg.biome_rocky_stone_wmul; }
        if cfg.biome_swamp_tree_growth_wmul == 0.0 { cfg.biome_swamp_tree_growth_wmul = def_cfg.biome_swamp_tree_growth_wmul; }
        if cfg.biome_rocky_tree_growth_wmul == 0.0 { cfg.biome_rocky_tree_growth_wmul = def_cfg.biome_rocky_tree_growth_wmul; }
        if cfg.biome_meadow_wheat_wmul == 0.0 { cfg.biome_meadow_wheat_wmul = def_cfg.biome_meadow_wheat_wmul; }
        if cfg.biome_swamp_wheat_wmul == 0.0 { cfg.biome_swamp_wheat_wmul = def_cfg.biome_swamp_wheat_wmul; }
        Ok((cfg, parsed.input))
    } else {
        let (config, input) = defaults();
        #[derive(Serialize)]
        struct FileCfg<'a> {
            config: &'a input::Config,
            input: &'a input::InputConfig,
        }
        let toml_text = toml::to_string_pretty(&FileCfg { config: &config, input: &input })?;
        // Сохраняем в пользовательскую директорию, а не в bundle
        use crate::resource_path;
        let user_config_path = resource_path::user_data_dir().join("config.toml");
        fs::write(&user_config_path, toml_text)?;
        Ok((config, input))
    }
}


