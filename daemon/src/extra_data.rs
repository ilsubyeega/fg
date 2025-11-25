//! Extra game data assets for Fall Guys.
//!
//! This module loads and provides access to static game data that is
//! extracted from Fall Guys game files. This data includes:
//!
//! - **Game Rules**: Round configuration (duration, team mode, scoring)
//! - **Levels/Rounds**: Round metadata (display names, tags, archetypes)
//! - **Localized Strings**: Human-readable names for rounds and shows
//! - **Shows**: Show/playlist configuration
//!
//! # Data Source
//!
//! The JSON data files are embedded at compile time from the `extra_datas/`
//! directory. See `extra_datas/README.md` for information about updating
//! these files.
//!
//! # Usage
//!
//! The data is accessible through the [`EXTRA_DATA_ASSETS`] static, which
//! is lazily initialized on first access.
//!
//! ```rust
//! use fg_daemon::extra_data::{EXTRA_DATA_ASSETS, localized_string};
//!
//! // Get round info
//! if let Some(round) = EXTRA_DATA_ASSETS.levels_round.get("round_tunnel_40") {
//!     println!("Round: {:?}", round.display_name);
//! }
//!
//! // Get localized string
//! let name = localized_string("ranked_show_knockout");
//! println!("Show name: {}", name); // "Ranked Knockout"
//! ```

use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

// =============================================================================
// Static Data Assets
// =============================================================================

/// Global static containing all loaded extra data assets.
///
/// This is lazily initialized on first access and contains all game data
/// needed for parsing and displaying Fall Guys information.
pub static EXTRA_DATA_ASSETS: LazyLock<FGExtraDataAssets> =
    LazyLock::new(initialize_extra_data_assets);

// Embedded JSON data
const STR_GAME_RULES: &str = include_str!("../extra_datas/game_rules.json");
const STR_LEVELS_ROUND: &str = include_str!("../extra_datas/levels_round.json");
const STR_LOCALIZED_STRINGS: &str = include_str!("../extra_datas/localised_strings.json");
const STR_SHOWS: &str = include_str!("../extra_datas/shows.json");

// =============================================================================
// Data Types
// =============================================================================

/// Container for all Fall Guys extra data assets.
///
/// Each field is a [`HashMap`] keyed by the item's ID for O(1) lookups.
pub struct FGExtraDataAssets {
    /// Game rules defining round behavior (duration, scoring, teams).
    pub game_rules: HashMap<String, FGExtraDataGameRulesItem>,
    /// Level/round metadata (display names, tags, archetype).
    pub levels_round: HashMap<String, FGExtraDataLevelsRoundItem>,
    /// Localized string translations.
    pub localized_strings: HashMap<String, String>,
    /// Show/playlist configuration.
    pub shows: HashMap<String, FGExtraDataShowsItem>,
}

/// Game rules configuration for a round.
///
/// Defines how a round behaves: duration, team settings, scoring rules,
/// qualification requirements, etc.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FGExtraDataGameRulesItem {
    /// Unique identifier for this rule set.
    pub id: String,
    /// Minimum number of participants for public lobbies.
    pub min_participants: Option<isize>,
    /// Maximum number of participants for public lobbies.
    pub max_participants: Option<isize>,
    /// Minimum participants for private lobbies.
    pub min_participants_private_lobby: Option<isize>,
    /// Maximum participants for private lobbies.
    pub max_participants_private_lobby: Option<isize>,
    /// Percentage of players that qualify.
    pub qualification_percentage: Option<isize>,
    /// Qualification percentage for squad modes.
    pub squads_qualification_percentage: Option<isize>,
    /// Whether the round has a timer.
    pub has_timer: Option<bool>,
    /// Round duration in seconds.
    pub duration: isize,
    /// Time threshold for red alert warning.
    pub time_left_red_alert_threshold: isize,
    /// How the game manager spawns players.
    pub game_manager_spawn_type: String,
    /// Condition for round to end.
    pub round_end_condition: String,
    /// Overtime mode configuration.
    pub overtime_mode: String,
    /// Overtime amount if applicable.
    pub overtime_amount: Option<isize>,
    /// Team mode (solo, squads, etc.).
    pub team_mode: String,
    /// Number of teams.
    pub team_count: Option<isize>,
    /// Whether teams must have equal sizes.
    pub require_same_team_sizes: Option<bool>,
    /// Number of teams that must be eliminated.
    pub required_team_eliminations: Option<isize>,
    /// Whether this is a scoring-based game.
    pub is_scoring_game: Option<bool>,
    /// How scores are displayed.
    pub score_display_mode: String,
    /// Whether to use creator-defined score target.
    pub use_creator_score_target: Option<bool>,
    /// Target score to win.
    pub score_target: Option<isize>,
    /// Score target for 2-player squads.
    pub score_target_squad_2players: Option<isize>,
    /// Score target for 3-player squads.
    pub score_target_squad_3players: Option<isize>,
    /// Score target for 4-player squads.
    pub score_target_squad_4players: Option<isize>,
    /// Whether piggyback grabbing is enabled.
    pub is_piggyback_enabled: Option<bool>,
    /// TOM (The One Monster) round rules.
    pub tom_round_rules: String,
}

/// Level/round metadata.
///
/// Contains display information and categorization for a round.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FGExtraDataLevelsRoundItem {
    /// Unique identifier (e.g., "round_tunnel_40").
    pub id: String,
    /// Localization key for the display name.
    pub display_name: Option<String>,
    /// Reference to the game rules for this round.
    pub game_rules: String,
    /// Additional round info reference.
    pub round_info: Option<String>,
    /// Ambient sound state.
    pub main_ambience_state: String,
    /// Loading screen identifier.
    pub loading_screen_name: String,
    /// Badge icon name.
    pub level_badge_name: String,
    /// Categorization tags.
    pub tags: Vec<String>,
    /// Round archetype (race, survival, team, etc.).
    pub level_archetype: String,
    /// Fall feed message reference.
    pub fall_feed: Option<String>,
}

/// Localized string entry (used for parsing).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FGExtraDataLocalizedStringsItem {
    /// String identifier.
    pub id: String,
    /// Translated text.
    pub text: String,
}

/// Show/playlist configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FGExtraDataShowsItem {
    /// Unique identifier.
    pub id: String,
    /// Localization key for show name.
    pub show_name: Option<String>,
    /// Localization key for show description.
    pub show_description: Option<String>,
    /// Content label for the show.
    pub content_label: String,
    /// Minimum party size to queue.
    pub min_party_size: u32,
    /// Maximum party size to queue.
    pub max_party_size: u32,
    /// Show type configuration.
    pub show_type: FGExtraDataShowsItemShowType,
    /// Episode reward settings reference.
    pub episode_reward_settings_id: String,
}

/// Show type configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FGExtraDataShowsItemShowType {
    /// Type switch (individual, squad, etc.).
    pub showtype_switch: String,
    /// Squad size (only present for squad modes).
    pub squad_size: Option<u32>,
}

// =============================================================================
// Initialization
// =============================================================================

/// Parses all JSON data files into their respective types.
fn parse_json() -> (
    Vec<FGExtraDataGameRulesItem>,
    Vec<FGExtraDataLevelsRoundItem>,
    Vec<FGExtraDataLocalizedStringsItem>,
    Vec<FGExtraDataShowsItem>,
) {
    (
        serde_json::from_str(STR_GAME_RULES)
            .expect("Failed to parse game_rules.json"),
        serde_json::from_str(STR_LEVELS_ROUND)
            .expect("Failed to parse levels_round.json"),
        serde_json::from_str(STR_LOCALIZED_STRINGS)
            .expect("Failed to parse localised_strings.json"),
        serde_json::from_str(STR_SHOWS)
            .expect("Failed to parse shows.json"),
    )
}

/// Initializes the extra data assets from embedded JSON.
///
/// This converts the vector-based JSON data into hashmaps for O(1) lookup.
fn initialize_extra_data_assets() -> FGExtraDataAssets {
    let (game_rules_vec, round_items, localized_strings_list, show_items) = parse_json();

    FGExtraDataAssets {
        game_rules: game_rules_vec
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect(),
        levels_round: round_items
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect(),
        localized_strings: localized_strings_list
            .into_iter()
            .map(|item| (item.id, item.text))
            .collect(),
        shows: show_items
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect(),
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Gets a localized string by key.
///
/// Handles both raw keys and keys prefixed with "localised_strings.".
///
/// # Arguments
///
/// * `key` - The localization key (e.g., "ranked_show_knockout")
///
/// # Returns
///
/// The localized string, or a placeholder if not found.
///
/// # Example
///
/// ```rust
/// use fg_daemon::extra_data::localized_string;
///
/// let name = localized_string("ranked_show_knockout");
/// assert_eq!(name, "Ranked Knockout");
/// ```
pub fn localized_string(key: &str) -> String {
    let key = key.strip_prefix("localised_strings.").unwrap_or(key);

    EXTRA_DATA_ASSETS
        .localized_strings
        .get(key)
        .cloned()
        .unwrap_or_else(|| format!("unknown_localized_key.{}", key))
}

/// Gets the localized display name for a round ID.
///
/// # Arguments
///
/// * `round_id` - The round identifier (e.g., "round_tunnel_40")
///
/// # Returns
///
/// The localized round name, or a placeholder if not found.
pub fn localized_string_round_id(round_id: &str) -> String {
    EXTRA_DATA_ASSETS
        .levels_round
        .get(round_id)
        .and_then(|round| round.display_name.as_ref())
        .map(|name| localized_string(name))
        .unwrap_or_else(|| format!("{} (Unknown)", round_id))
}

/// Gets the localized display name for a show ID.
///
/// # Arguments
///
/// * `show_id` - The show identifier (e.g., "show_wle_xtreme")
///
/// # Returns
///
/// The localized show name, or a placeholder if not found.
pub fn localized_string_show_id(show_id: &str) -> String {
    EXTRA_DATA_ASSETS
        .shows
        .get(show_id)
        .and_then(|show| show.show_name.as_ref())
        .map(|name| localized_string(name))
        .unwrap_or_else(|| format!("{} (Unknown)", show_id))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_assets() {
        // This will panic if JSON parsing fails
        let _ = &*EXTRA_DATA_ASSETS;
    }

    #[test]
    fn test_localized_string() {
        assert_eq!(localized_string("ranked_show_knockout"), "Ranked Knockout");
    }

    #[test]
    fn test_localized_string_with_prefix() {
        assert_eq!(
            localized_string("localised_strings.ranked_show_knockout"),
            "Ranked Knockout"
        );
    }

    #[test]
    fn test_localized_string_not_found() {
        let result = localized_string("nonexistent_key");
        assert!(result.starts_with("unknown_localized_key."));
    }
}
