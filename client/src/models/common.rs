use std::{fmt::Display, str::FromStr};

use crate::extra_data::{EXTRA_DATA_ASSETS, localized_string_round_id, localized_string_show_id};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

/// The Game Mode enum.
pub enum FGGameMode {
    Knockout,
    RankedKnockout,

    ClassicSolo,
    ClassicDuo,
    ClassicSquads,

    Explore,
    CreatorSpotlight,

    /// The extra modes.
    /// Can parsed with `shows` data.
    Extra {
        name: String,
        id: String,
    },
    /// We don't know this gamemode, but try to fallback parse then use it if possible.
    UnknownAssumed(Box<FGGameMode>, String),
    /// If UnknownAssumed fails, this will throw.
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

/// The Player ID seems to be number, but for sure, let's use String as a fallback.
pub type FGPlayerId = isize;
/// When loading a player, NetPlayerID is incremented by 1, but PlayerID is not.
pub type FGNetPlayerId = isize;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]

/// The round data.
pub struct FGRoundInfo {
    pub id: String,
    /// The display name. Should be retrived from the `levels_round` data.
    pub display_name: String,
}
impl FGRoundInfo {
    /// id: e.g `round_tunnel_40`
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The Fall Guys platform. Can retrived via `settings_matchmaking_multiplay` data.
pub enum FGPlatform {
    /// pc_egs
    PCEpicGamesStore,
    /// pc_steam
    PCSteam,
    /// pc_standalone
    PCStandalone,
    /// switch
    Switch,
    /// xb1
    XboxOne,
    /// xsx
    XboxSeriesX,
    /// ps4
    PlayStation4,
    /// ps5
    PlayStation5,
    /// android_standalone
    AndroidStandalone,
    /// android_ega
    AndroidEpicGamesAccount,
    /// ios_ega
    IOSEpicGamesAccount,

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

pub type FGCreativeShareCode = String;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub enum FGRoundBadge {
    Gold,
    Silver,
    Bronze,
    None,
    Fail,
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
