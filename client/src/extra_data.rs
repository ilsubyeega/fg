use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

pub static EXTRA_DATA_ASSETS: LazyLock<FGExtraDataAssets> =
    LazyLock::new(initialize_extra_data_assets);

const STR_GAME_RULES: &str = include_str!("../extra_datas/game_rules.json");
const STR_LEVELS_ROUND: &str = include_str!("../extra_datas/levels_round.json");
const STR_LOCALIZED_STRINGS: &str = include_str!("../extra_datas/localised_strings.json");
const STR_SHOWS: &str = include_str!("../extra_datas/shows.json");

pub struct FGExtraDataAssets {
    pub game_rules: HashMap<String, FGExtraDataGameRulesItem>,
    pub levels_round: HashMap<String, FGExtraDataLevelsRoundItem>,
    pub localized_strings: HashMap<String, String>,
    pub shows: HashMap<String, FGExtraDataShowsItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FGExtraDataGameRulesItem {
    pub id: String,
    pub min_participants: Option<isize>,
    pub max_participants: Option<isize>,
    pub min_participants_private_lobby: Option<isize>,
    pub max_participants_private_lobby: Option<isize>,
    pub qualification_percentage: Option<isize>,
    pub squads_qualification_percentage: Option<isize>,
    pub has_timer: Option<bool>,
    pub duration: isize,
    pub time_left_red_alert_threshold: isize,
    pub game_manager_spawn_type: String,
    pub round_end_condition: String,
    pub overtime_mode: String,
    pub overtime_amount: Option<isize>,
    pub team_mode: String,
    pub team_count: Option<isize>,
    pub require_same_team_sizes: Option<bool>,
    pub required_team_eliminations: Option<isize>,
    pub is_scoring_game: Option<bool>,
    pub score_display_mode: String,
    pub use_creator_score_target: Option<bool>,
    pub score_target: Option<isize>,
    pub score_target_squad_2players: Option<isize>,
    pub score_target_squad_3players: Option<isize>,
    pub score_target_squad_4players: Option<isize>,
    pub is_piggyback_enabled: Option<bool>,
    pub tom_round_rules: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FGExtraDataLevelsRoundItem {
    pub id: String,
    pub display_name: Option<String>,
    pub game_rules: String,
    pub round_info: Option<String>,
    pub main_ambience_state: String,
    pub loading_screen_name: String,
    pub level_badge_name: String,
    pub tags: Vec<String>,
    pub level_archetype: String,
    pub fall_feed: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FGExtraDataLocalizedStringsItem {
    pub id: String,
    pub text: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FGExtraDataShowsItem {
    pub id: String,
    pub show_name: Option<String>,
    pub show_description: Option<String>,
    pub content_label: String,
    pub min_party_size: u32,
    pub max_party_size: u32,
    pub show_type: FGExtraDataShowsItemShowType,
    pub episode_reward_settings_id: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FGExtraDataShowsItemShowType {
    pub showtype_switch: String,
    // Does not shown when showtype_switch is individual.
    pub squad_size: Option<u32>,
}

fn parse_json() -> (
    Vec<FGExtraDataGameRulesItem>,
    Vec<FGExtraDataLevelsRoundItem>,
    Vec<FGExtraDataLocalizedStringsItem>,
    Vec<FGExtraDataShowsItem>,
) {
    (
        serde_json::from_str(STR_GAME_RULES).unwrap(),
        serde_json::from_str(STR_LEVELS_ROUND).unwrap(),
        serde_json::from_str(STR_LOCALIZED_STRINGS).unwrap(),
        serde_json::from_str(STR_SHOWS).unwrap(),
    )
}

fn initialize_extra_data_assets() -> FGExtraDataAssets {
    let (game_rules_vec, round_item, localized_strings_list, show_item) = parse_json();

    let mut game_rules = HashMap::new();
    for item in game_rules_vec {
        game_rules.insert(item.id.clone(), item);
    }
    
    let mut levels_round = HashMap::new();
    for item in round_item {
        levels_round.insert(item.id.clone(), item);
    }

    let mut localized_strings = HashMap::new();
    for item in localized_strings_list {
        localized_strings.insert(item.id, item.text);
    }

    let mut shows = HashMap::new();
    for item in show_item {
        shows.insert(item.id.clone(), item);
    }

    FGExtraDataAssets {
        game_rules,
        levels_round,
        localized_strings,
        shows,
    }
}

#[test]
fn test_init_assets() {
    initialize_extra_data_assets();
}

pub fn localized_string(key: &str) -> String {
    let mut key = key.to_owned();
    if key.starts_with("localised_strings.") {
        key = key.replace("localised_strings.", "");
    }

    let res = EXTRA_DATA_ASSETS.localized_strings.get(&key);
    match res {
        Some(text) => text.clone(),
        None => format!("unknown_localized_key.{}", key),
    }
    .to_owned()
}

pub fn localized_string_round_id(round_id: &str) -> String {
    match EXTRA_DATA_ASSETS.levels_round.get(round_id) {
        Some(round) => {
            if let Some(display_name) = &round.display_name {
                localized_string(display_name)
            } else {
                format!("{} (no display name)", round_id)
            }
        }
        None => format!("{} (Unknown)", round_id),
    }
}

pub fn localized_string_show_id(show_id: &str) -> String {
    match EXTRA_DATA_ASSETS.shows.get(show_id) {
        Some(show) => {
            if let Some(show_name) = &show.show_name {
                localized_string(show_name)
            } else {
                format!("{} (no display name)", show_id)
            }
        }
        None => format!("{} (Unknown)", show_id),
    }
}

#[test]
fn get_localized_string_text() {
    assert_eq!(localized_string("ranked_show_knockout"), "Ranked Knockout");
    assert_eq!(
        localized_string("localised_strings.ranked_show_knockout"),
        "Ranked Knockout"
    );
}
