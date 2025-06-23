use std::collections::HashMap;

use super::{
    common::{FGGameMode, FGPlatform, FGRoundBadge},
    messages::FGMatchmakingMessage,
    state::{FGGameSessionState, FGGameState},
};

/// The initial data to send, just in case previous state is not found.
pub struct FGExportsCurrent {
    pub state: Option<FGGameState>,
    pub game_session_state: Option<FGGameSessionState>,
    pub gamemode: Option<FGGameMode>,
    pub round_info: Option<FGExportsRoundInfo>,
    pub round_result: Option<FGExportsRoundResult>,
    pub lobby_info: Option<FGExportsLobbyInfo>,

    pub my_data: Option<FGExportsMyData>,
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
    pub qualification_time: Option<f32>,
    /// Unix timestamp. Not none when rounds ends with
    /// RoundOver message.
    pub round_over_time: Option<usize>,
}

pub struct FGExportsLobbyInfo {
    pub platforms: HashMap<FGPlatform, isize>,
    /// Resets every next round.
    pub platforms_delta: HashMap<FGPlatform, isize>,
    pub platforms_myteam: Option<HashMap<FGPlatform, isize>>,
    pub latency: Option<isize>,
    pub latency_from_time: Option<isize>,
}

/// The data fetched from database and current states.
pub struct FGExportsMyData {
    pub round_qualifiation_count: Option<isize>,
    pub round_played_count: Option<isize>,
    pub round_gold_count: Option<isize>,
    pub round_personal_best_time: Option<f32>,

    pub wins: Option<isize>,
    pub played: Option<isize>,
    pub wins_streak: Option<isize>,
    pub wins_streak_best: Option<isize>,

    pub finals: Option<isize>,
    pub finals_streak: Option<isize>,
    pub finals_streak_best: Option<isize>,
}
