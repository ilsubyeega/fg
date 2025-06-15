use std::str::FromStr;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub enum FGGameState {
    StateMainMenu,
    StateMatchmaking,
    StateConnectToGame,
    StateConnectionAuthentication,
    StateGameLoading,
    StateWaitingForUser,
    StateGameInProgress,
    StateQualificationScreen,
    StateRoundReadyUp,
    /// Used for getting rewards on Explore etc.
    StateUltimatePartyRewardFlow,
    StateVictoryScreen,
    StateWaitingForRewards,
    StateDisconnectingFromServer,
    StateRewardScreen,
    StateReloadingToMainMenu,
    Unknown(String),
}

impl FromStr for FGGameState {
    type Err = String;

    fn from_str(text: &str) -> Result<FGGameState, String> {
        let mut text = text.to_owned();
        if text.starts_with("FGClient.") {
            text = text.replace("FGClient.", "");
        }
        let val = match text.as_str() {
            "StateMainMenu" => FGGameState::StateMainMenu,
            "StateMatchmaking" => FGGameState::StateMatchmaking,
            "StateConnectToGame" => FGGameState::StateConnectToGame,
            "StateConnectionAuthentication" => FGGameState::StateConnectionAuthentication,
            "StateGameLoading" => FGGameState::StateGameLoading,
            "StateWaitingForUser" => FGGameState::StateWaitingForUser,
            "StateGameInProgress" => FGGameState::StateGameInProgress,
            "StateQualificationScreen" => FGGameState::StateQualificationScreen,
            "StateRoundReadyUp" => FGGameState::StateRoundReadyUp,
            "StateUltimatePartyRewardFlow" => FGGameState::StateUltimatePartyRewardFlow,
            "StateVictoryScreen" => FGGameState::StateVictoryScreen,
            "StateWaitingForRewards" => FGGameState::StateWaitingForRewards,
            "StateDisconnectingFromServer" => FGGameState::StateDisconnectingFromServer,
            "StateRewardScreen" => FGGameState::StateRewardScreen,
            "StateReloadingToMainMenu" => FGGameState::StateReloadingToMainMenu,
            _ => FGGameState::Unknown(text.to_string()),
        };
        Ok(val)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FGClientReadinessState {
    ReceivedLevelDetails,
    LevelLoaded,
    ObjectsSpawned,
    ReadyToPlay,
    Unknown(String),
}

impl FromStr for FGClientReadinessState {
    type Err = String;
    fn from_str(text: &str) -> Result<FGClientReadinessState, String> {
        let val = match text {
            "ReceivedLevelDetails" => FGClientReadinessState::ReceivedLevelDetails,
            "LevelLoaded" => FGClientReadinessState::LevelLoaded,
            "ObjectsSpawned" => FGClientReadinessState::ObjectsSpawned,
            "ReadyToPlay" => FGClientReadinessState::ReadyToPlay,
            _ => FGClientReadinessState::Unknown(text.to_string()),
        };
        Ok(val)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FGGameSessionState {
    Precountdown,
    Countdown,
    Playing,
    GameOver,
    Results,
    Unknown(String),
}

impl FromStr for FGGameSessionState {
    type Err = String;
    fn from_str(text: &str) -> Result<FGGameSessionState, String> {
        let val = match text {
            "Precountdown" => FGGameSessionState::Precountdown,
            "Countdown" => FGGameSessionState::Countdown,
            "Playing" => FGGameSessionState::Playing,
            "GameOver" => FGGameSessionState::GameOver,
            "Results" => FGGameSessionState::Results,
            _ => FGGameSessionState::Unknown(text.to_string()),
        };
        Ok(val)
    }
}
