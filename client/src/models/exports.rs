use std::collections::HashMap;

use super::{common::{FGGameMode, FGPlatform, FGRoundBadge}, messages::FGMatchmakingMessage, state::{FGGameSessionState, FGGameState}};

/// The initial data to send, just in case previous state is not found.
pub struct FGExportsCurrent {
    pub state: Option<FGGameState>,
    pub game_session_state: Option<FGGameSessionState>,
    pub gamemode: Option<FGGameMode>,
    pub round_info: Option<FGExportsRoundInfo>,
    pub round_result: Option<FGExportsRoundResult>,
    pub lobby_info: Option<FGExportsLobbyInfo>,
}

pub struct FGExportsMatchmaking(Option<FGMatchmakingMessage>);

pub struct FGExportsRoundInfo {
    pub round_order: isize,
    pub round_id_str: String,
    pub round_display_name: String,
    pub duration: Option<isize>,
}

pub struct FGExportsRoundResult {
    pub badge: Option<FGRoundBadge>,
    pub qualified: bool,
    /// Unix timestamp. Not none when rounds ends with
    /// RoundOver message.
    pub round_over_time: Option<usize>,
}

pub struct FGExportsLobbyInfo {
    pub platforms: HashMap<FGPlatform, isize>,
    pub latency: Option<isize>,
    pub latency_from_time: Option<isize>,
}