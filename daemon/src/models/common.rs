//! Common types used throughout the Fall Guys daemon.
//!
//! This module contains shared data types that are used across multiple
//! components of the log parser.

use std::{fmt::Display, str::FromStr};

use crate::extra_data::{EXTRA_DATA_ASSETS, localized_string_round_id, localized_string_show_id};

// =============================================================================
// Game Mode
// =============================================================================

/// The game mode/show type.
///
/// Represents the different playlists/modes available in Fall Guys.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FGGameMode {
    /// Standard elimination mode.
    Knockout,
    /// Competitive ranked mode.
    RankedKnockout,
    /// Classic solo show.
    ClassicSolo,
    /// Classic duos show (2-player squads).
    ClassicDuo,
    /// Classic squads show (4-player squads).
    ClassicSquads,
    /// Casual exploration mode.
    Explore,
    /// Community-created content spotlight.
    CreatorSpotlight,
    /// Extra modes from `shows.json`.
    ///
    /// These are limited-time or special event modes.
    Extra {
        /// Localized display name.
        name: String,
        /// Internal identifier.
        id: String,
    },
    /// Unknown mode that was assumed based on name patterns.
    UnknownAssumed(Box<FGGameMode>, String),
    /// Completely unknown mode.
    Unknown(String),
}

impl FromStr for FGGameMode {
    type Err = String;
    /// Parsed from id.
    fn from_str(text: &str) -> Result<FGGameMode, String> {
        let val = match text {
            "ranked_show_knockout" => FGGameMode::RankedKnockout,
            "knockout_mode" => FGGameMode::Knockout,
            "classic_solo_main_show" => FGGameMode::ClassicSolo,
            "classic_duos_show" => FGGameMode::ClassicDuo,
            "classic_squads_show" => FGGameMode::ClassicSquads,
            "spotlight_mode" => FGGameMode::CreatorSpotlight,
            "casual_show" => FGGameMode::Explore,
            _ => {
                // Get extra shows from `shows.json`.
                let game_id = text.to_owned();
                if EXTRA_DATA_ASSETS.shows.contains_key(&game_id) {
                    return Ok(FGGameMode::Extra {
                        name: localized_string_show_id(&game_id),
                        id: game_id,
                    });
                }

                // Fallback: Assuming.
                let fallback = if text.contains("solo") {
                    FGGameMode::ClassicSolo
                } else if text.contains("duo") {
                    FGGameMode::ClassicDuo
                } else if text.contains("squads") {
                    FGGameMode::ClassicSquads
                } else if text.contains("ranked") {
                    FGGameMode::RankedKnockout
                } else if text.contains("knockout") {
                    FGGameMode::Knockout
                } else if text.contains("explore") {
                    FGGameMode::Explore
                } else {
                    FGGameMode::Unknown(text.to_owned())
                };

                match fallback {
                    FGGameMode::Unknown(_) => fallback,
                    _ => FGGameMode::UnknownAssumed(Box::new(fallback), text.to_owned()),
                }
            }
        };

        Ok(val)
    }
}

impl Display for FGGameMode {
    fn fmt(self: &FGGameMode, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FGGameMode::Knockout => write!(f, "Knockout"),
            FGGameMode::RankedKnockout => write!(f, "Ranked Knockout"),
            FGGameMode::ClassicSolo => write!(f, "Classic Solo"),
            FGGameMode::ClassicDuo => write!(f, "Classic Duo"),
            FGGameMode::ClassicSquads => write!(f, "Classic Squads"),
            FGGameMode::Explore => write!(f, "Explore"),
            FGGameMode::CreatorSpotlight => write!(f, "Creator Spotlight"),
            FGGameMode::Extra { name, id } => write!(f, "{} ({})", name, id),
            FGGameMode::UnknownAssumed(mode, value) => {
                write!(f, "Unknown: {} (Assumed {})", value, mode)
            }
            FGGameMode::Unknown(_) => write!(f, "Unknown"),
        }
    }
}

// =============================================================================
// Player IDs
// =============================================================================

/// Player identifier assigned by the game.
pub type FGPlayerId = isize;

/// Network player identifier (increments as players join).
///
/// Note: `FGNetPlayerId` increments by 1 when loading a player,
/// but `FGPlayerId` does not.
pub type FGNetPlayerId = isize;

// =============================================================================
// Round Info
// =============================================================================

/// Information about a game round.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FGRoundInfo {
    /// Internal round identifier (e.g., "round_tunnel_40").
    pub id: String,
    /// Human-readable display name.
    pub display_name: String,
}

impl FGRoundInfo {
    /// Creates round info from a string ID.
    ///
    /// For UGC (user-generated content) rounds, creates a placeholder name.
    /// For standard rounds, looks up the localized name.
    ///
    /// # Arguments
    ///
    /// * `id` - The round identifier (e.g., "round_tunnel_40" or "ugc-12345")
    pub fn from_str_id(id: &str) -> Self {
        if id.starts_with("ugc-") {
            FGRoundInfo {
                id: id.to_owned(),
                display_name: format!("Creative: {id}"),
            }
        } else {
            FGRoundInfo {
                id: id.to_owned(),
                display_name: localized_string_round_id(id),
            }
        }
    }
}

// =============================================================================
// Platform
// =============================================================================

/// Player's gaming platform.
///
/// Identifies which platform a player is using to play Fall Guys.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FGPlatform {
    /// Epic Games Store (PC).
    PCEpicGamesStore,
    /// Steam (PC).
    PCSteam,
    /// Standalone PC client.
    PCStandalone,
    /// Nintendo Switch.
    Switch,
    /// Xbox One.
    XboxOne,
    /// Xbox Series X|S.
    XboxSeriesX,
    /// PlayStation 4.
    PlayStation4,
    /// PlayStation 5.
    PlayStation5,
    /// Android standalone.
    AndroidStandalone,
    /// Android with Epic Games account.
    AndroidEpicGamesAccount,
    /// iOS with Epic Games account.
    IOSEpicGamesAccount,
    /// Unknown platform.
    Unknown(Option<String>),
}

impl FromStr for FGPlatform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pc_egs" => Ok(FGPlatform::PCEpicGamesStore),
            "pc_steam" => Ok(FGPlatform::PCSteam),
            "pc_standalone" => Ok(FGPlatform::PCStandalone),
            "switch" => Ok(FGPlatform::Switch),
            "xb1" => Ok(FGPlatform::XboxOne),
            "xsx" => Ok(FGPlatform::XboxSeriesX),
            "ps4" => Ok(FGPlatform::PlayStation4),
            "ps5" => Ok(FGPlatform::PlayStation5),
            "android_standalone" => Ok(FGPlatform::AndroidStandalone),
            "android_ega" => Ok(FGPlatform::AndroidEpicGamesAccount),
            "ios_ega" => Ok(FGPlatform::IOSEpicGamesAccount),
            _ => Ok(FGPlatform::Unknown(Some(s.to_owned()))),
        }
    }
}

// =============================================================================
// Creative Share Code
// =============================================================================

/// Share code for user-generated creative content.
pub type FGCreativeShareCode = String;

// =============================================================================
// Round Badge
// =============================================================================

/// Badge awarded at the end of a round.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FGRoundBadge {
    /// First place / top tier.
    Gold,
    /// Second tier.
    Silver,
    /// Third tier.
    Bronze,
    /// No badge (qualified but no special placement).
    None,
    /// Failed / eliminated.
    Fail,
    /// Unknown badge type.
    Unknown(Option<String>),
}

impl FromStr for FGRoundBadge {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gold" => Ok(FGRoundBadge::Gold),
            "silver" => Ok(FGRoundBadge::Silver),
            "bronze" => Ok(FGRoundBadge::Bronze),
            "none" => Ok(FGRoundBadge::None),
            "fail" => Ok(FGRoundBadge::Fail),
            _ => Ok(FGRoundBadge::Unknown(Some(s.to_string()))),
        }
    }
}
