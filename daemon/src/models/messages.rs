use crate::models::{common::FGGameMode, state::FGGameState};
use crate::models::{
    common::{FGCreativeShareCode, FGNetPlayerId, FGPlatform, FGPlayerId, FGRoundInfo},
    dto::FGCompletedEpisodeDto,
    state::{FGClientReadinessState, FGGameSessionState},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FGGameMessage {
    // [GameStateMachine]
    // Replacing FGClient.StateConnectionAuthentication with FGClient.StateGameLoading
    /// Game changed the state via `GameStateMachine`.
    GameStateChanged {
        before: Option<FGGameState>,
        after: FGGameState,
    },

    // ​[Matchmaking]
    // Begin matchmaking solo
    /// Started a matchmaking with the mode (not the game mode).
    BeginMatchmaking,

    // ​[FNMMSClientRemoteService]
    // Status message received: {JSON}
    /// Received a message while matchmaking.
    MatchmakingMessageReceived(FGMatchmakingMessage),

    // ​[StateConnectToGame]
    // InitiateNetworkConnectRequest with server IP:127.0.0.1:9999
    /// Connected to the game lobby.
    ServerConnected {
        ip_addr: String,
        port: Option<String>,
    },
    // [CreateLocalPlayerInstances]
    // Added new player as Participant, player ID = 0
    /// Created a local player with given player id.
    CreateLocalPlayer(FGPlayerId),

    // [HandleSuccessfulLogin]
    // Selected show is classic_solo_main_show
    //  IsUltimatePartyEpisode: False
    // Session: 12345678-server-1234567890
    /// Successfully joined a game lobby.
    SuccessfullyJoined {
        game_mode: FGGameMode,
        session_text: Option<String>,
    },

    // ​[FG_UnityInternetNetworkManager]
    // Networking Metrics after 123.4567 s of operation:
    /// Parsing networking metrics to read network latency.
    /// Currently only parses latency only.
    NetworkMetrics {
        latency: isize,
    },

    // [ClientGameManager]
    // Setting this client as readiness state 'ReceivedLevelDetails'.
    /// Seems like changing the states then saving to the server.
    SetClientReadiness(FGClientReadinessState),

    // [StateGameLoading]
    // Finished loading game level,
    // assumed to be round_tunnel_40. Duration: .99s
    /// Loaded a round.
    LoadedRound(FGRoundInfo),

    // Requesting spawn of local player, ID=0
    /// Requesting spawn of local player with given player ID.
    RequestLocalPlayer(FGPlayerId),

    // [ClientGameManager]
    // Handling bootstrap for remote player FallGuy
    // [17] (FG.Common.MPGNetObject),
    // playerID = 0, squadID = 0
    /// Handling remote player data.
    HandleRemotePlayer {
        player_id: FGPlayerId,
        net_player_id: FGNetPlayerId,
        squad_id: Option<isize>,
    },
    // Handling bootstrap for local player FallGuy
    // [35] (FG.Common.MPGNetObject),
    // playerID = 0, squadID = 0
    /// Handling local player data.
    HandleLocalPlayer {
        player_id: FGPlayerId,
        net_player_id: FGNetPlayerId,
        squad_id: Option<isize>,
    },
    // [CameraDirector]
    // Adding Spectator target ...
    // (pc_egs) with
    // Party ID:
    // Squad ID: 0 and
    // playerID: 6
    /// Intende to retrive extra data.
    AppendSpectatorTarget {
        player_id: FGPlayerId,
        squad_id: Option<isize>,
        party_id: Option<isize>,
        platform: FGPlatform,
    },

    // [StateGameLoading]
    // OnPlayerSpawned - NetID=6 ID=6 was spawned
    /// Handling player spawn event.
    /// This sometimes call multiple times. Should use other message for collecting player lists.
    PlayerSpawned {
        player_id: FGPlayerId,
        net_player_id: FGNetPlayerId,
    },

    // [CameraDirector] Set Local Squad ID: 0
    /// Setting the current local squad ID.
    SetLocalSquadId(Option<isize>),

    // [PartyStateManager] Local player party ID:
    // [CameraDirector] Set Local Party ID:
    /// Setting the current local party ID.
    SetLocalPartyId(Option<isize>),

    // [GameSession] Changing state from Precountdown to Countdown
    /// Changes about Game Session State.
    GameSessionState {
        after: FGGameSessionState,
        before: Option<FGGameSessionState>,
    },

    SetNumPlayersAchievingObjective(isize),

    // Player 00 score =
    /// The server message that counting own/other player score.
    SetPlayerScore {
        net_player_id: FGNetPlayerId,
        score: isize,
    },

    // [ClientGameManager] Handling unspawn for player 1
    /// Handling player unspawn event.
    /// Could happen in the player died or round ends.
    HandleUnspawn(FGNetPlayerId),

    /// Setting player's succeeded status.
    SetPlayerProgress {
        player_id: FGPlayerId,
        is_succeeded: bool,
    },

    // == [CompletedEpisodeDto] ==
    // .....
    /// The lobby rewards after eliminated/won.
    GameLobbyRewards(FGCompletedEpisodeDto),

    // ​[RoundLoader]
    // Load UGC via share code: 0275-8967-0239:572
    /// Game is loading creative round.
    CreativeRoundLoader(FGCreativeShareCode),

    // [LeaveMatchPopupManager] Calling CloseScreen()
    /// This is called when game just want to clear the 3D session, I guess?
    /// Not only called for leaving the match. So don't use this, but use GameSession one.
    LeaveMatch,

    // [ClientGameManager]
    // Server notifying that the round is over.
    /// Server notifying that the round is over.
    /// Happens after `ServerMessageEndRound` received.
    RoundOver,

    ServerMessageStartLoadingLevel,
    ServerMessageReadyRoundResponse,
    ServerMessageRoundResults,
    ServerMessageEndRound,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FGMatchmakingMessage {
    /// Connecting to matchmaking server.
    Connecting,
    /// throws this message when joined a matchmaking server.
    QueueFull,
    /// waiting a server?
    Waiting,
    /// Queued
    Queued { queued_players: isize },
    /// Session assigned. Not sure what is this.
    /// After received this message, Play message also gets received.
    SessionAssignment,
    /// Join the game, I guess?
    Play,
}
