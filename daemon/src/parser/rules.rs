//! Fall Guys log parsing rules.
//!
//! This module contains all the parsing rules for extracting game events
//! from Fall Guys log files. Each rule function parses a specific type
//! of log message and returns a [`ParseResult`].
//!
//! # Architecture
//!
//! Rules are organized by their purpose:
//!
//! - **State changes**: Game state machine transitions
//! - **Matchmaking**: Queue status and server connection
//! - **Player events**: Spawn, unspawn, score updates
//! - **Round events**: Loading, completion, rewards
//!
//! # Adding New Rules
//!
//! To add a new parsing rule:
//!
//! 1. Create a function with signature `fn rule_name(input: &str) -> ParseResult<FGGameMessage>`
//! 2. Use early return with `ParseResult::None` if the log line doesn't match
//! 3. Use combinators from [`super::combinators`] for parsing
//! 4. Add the rule to the [`rules()`] function
//!
//! # Example
//!
//! ```rust
//! fn my_new_rule(input: &str) -> ParseResult<FGGameMessage> {
//!     // Early return if this isn't the right log line
//!     if !input.contains("[MyTag] Some pattern") {
//!         return ParseResult::None;
//!     }
//!     
//!     // Parse the relevant data
//!     fn parse_data(input: &str) -> IResult<&str, MyData> {
//!         // ... parsing logic using nom combinators
//!     }
//!     
//!     match parse_data(input) {
//!         Ok((_, data)) => ParseResult::Parsed(FGGameMessage::MyEvent(data)),
//!         Err(_) => ParseResult::None,
//!     }
//! }
//! ```

use std::{str::FromStr, sync::Arc};

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::char,
    combinator::opt,
    sequence::{delimited, preceded},
};
use tracing::warn;

use crate::{
    extra_data::localized_string_round_id,
    models::{
        common::{FGPlatform, FGRoundInfo},
        dto::{
            FGCompletedEpisodeDto, FGCompletedEpisodeDtoRound,
            generate_fg_completed_episode_dto_round,
        },
        messages::{FGGameMessage, FGMatchmakingMessage},
        state::{FGClientReadinessState, FGGameSessionState},
    },
    parser::ParseResult,
};

// Re-import commonly used combinators for cleaner code
use super::combinators::{identifier, ip_address, parse_isize, port_number, skip_to_tag};

// ============================================================================
// Rule parsers
// ============================================================================

fn game_state_changed(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[GameStateMachine] Replacing ") {
        return ParseResult::None;
    }

    fn parse_state_change(input: &str) -> IResult<&str, (&str, &str)> {
        let (input, _) = skip_to_tag("Replacing ")(input)?;
        let (input, before) = identifier(input)?;
        let (input, _) = tag(" with ")(input)?;
        let (input, after) = identifier(input)?;
        Ok((input, (before, after)))
    }

    match parse_state_change(input) {
        Ok((_, (before, after))) => {
            let before_state = before.parse().ok();
            let after_state = after.parse().unwrap();
            ParseResult::Parsed(FGGameMessage::GameStateChanged {
                before: before_state,
                after: after_state,
            })
        }
        Err(_) => ParseResult::None,
    }
}

fn begin_matchmaking(input: &str) -> ParseResult<FGGameMessage> {
    if input.contains("[Matchmaking] Begin matchmaking") {
        ParseResult::Parsed(FGGameMessage::BeginMatchmaking)
    } else {
        ParseResult::None
    }
}

fn matchmaking_message_received(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[FNMMSClientRemoteService] Status message received:") {
        return ParseResult::None;
    }
    if !input.contains("\"state\": ") {
        return ParseResult::NeedMoreLines;
    }

    /// Parse a JSON-like key-value pair: "key": "value" or "key": value
    fn parse_json_kv(input: &str) -> IResult<&str, (&str, &str)> {
        let (input, _) = take_until("\"")(input)?;
        let (input, key) = delimited(char('"'), take_while1(|c: char| c != '"'), char('"')).parse(input)?;
        let (input, _) = tag(": ")(input)?;
        // Value can be quoted or unquoted
        let (input, value) = alt((
            delimited(char('"'), take_while(|c: char| c != '"'), char('"')),
            take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '.'),
        )).parse(input)?;
        Ok((input, (key, value)))
    }

    for line in input.lines() {
        if let Ok((_, (key, value))) = parse_json_kv(line) {
            match key {
                "name" => match value {
                    "Play" => {
                        return ParseResult::Parsed(FGGameMessage::MatchmakingMessageReceived(
                            FGMatchmakingMessage::Play,
                        ));
                    }
                    "Error" => return ParseResult::None,
                    _ => {}
                },
                "queuedPlayers" => {
                    if value != "null" {
                        if let Ok(queued_players) = value.parse() {
                            return ParseResult::Parsed(FGGameMessage::MatchmakingMessageReceived(
                                FGMatchmakingMessage::Queued { queued_players },
                            ));
                        }
                    }
                }
                "state" => match value {
                    "Connecting" => {
                        return ParseResult::Parsed(FGGameMessage::MatchmakingMessageReceived(
                            FGMatchmakingMessage::Connecting,
                        ));
                    }
                    "QueueFull" => {
                        return ParseResult::Parsed(FGGameMessage::MatchmakingMessageReceived(
                            FGMatchmakingMessage::QueueFull,
                        ));
                    }
                    "Waiting" => {
                        return ParseResult::Parsed(FGGameMessage::MatchmakingMessageReceived(
                            FGMatchmakingMessage::Waiting,
                        ));
                    }
                    "SessionAssignment" => {
                        return ParseResult::Parsed(FGGameMessage::MatchmakingMessageReceived(
                            FGMatchmakingMessage::SessionAssignment,
                        ));
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    unreachable!()
}

fn server_connected(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[StateConnectToGame] InitiateNetworkConnectRequest with server IP: ") {
        return ParseResult::None;
    }

    fn parse_server_ip(input: &str) -> IResult<&str, (&str, Option<&str>)> {
        let (input, _) = skip_to_tag("server IP: ")(input)?;
        let (input, ip) = ip_address(input)?;
        let (input, port) = opt(preceded(char(':'), port_number)).parse(input)?;
        Ok((input, (ip, port)))
    }

    match parse_server_ip(input) {
        Ok((_, (ip, port))) => ParseResult::Parsed(FGGameMessage::ServerConnected {
            ip_addr: ip.to_owned(),
            port: port.map(|p| p.to_owned()),
        }),
        Err(_) => ParseResult::None,
    }
}

fn create_local_player(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[CreateLocalPlayerInstances] Added new player as Participant") {
        return ParseResult::None;
    }

    fn parse_player_id(input: &str) -> IResult<&str, isize> {
        let (input, _) = skip_to_tag("player ID = ")(input)?;
        parse_isize(input)
    }

    match parse_player_id(input) {
        Ok((_, player_id)) => ParseResult::Parsed(FGGameMessage::CreateLocalPlayer(player_id)),
        Err(_) => ParseResult::None,
    }
}

fn successfully_joined(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[HandleSuccessfulLogin] Selected show is ") {
        return ParseResult::None;
    }
    if !input.contains("[HandleSuccessfulLogin] Session: ") {
        return ParseResult::NeedMoreLines;
    }

    fn parse_show(input: &str) -> IResult<&str, &str> {
        let (input, _) = skip_to_tag("Selected show is ")(input)?;
        take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
    }

    fn parse_session(input: &str) -> IResult<&str, &str> {
        let (input, _) = skip_to_tag("Session: ")(input)?;
        take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-')(input)
    }

    let show = match parse_show(input) {
        Ok((_, show)) => show,
        Err(_) => return ParseResult::None,
    };

    let session = parse_session(input).ok().map(|(_, s)| s.to_owned());

    ParseResult::Parsed(FGGameMessage::SuccessfullyJoined {
        game_mode: show.parse().unwrap(),
        session_text: session,
    })
}

fn network_metrics(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[FG_UnityInternetNetworkManager] Networking Metrics after") {
        return ParseResult::None;
    }
    if !input.contains("Network - RTT: ") {
        return ParseResult::NeedMoreLines;
    }

    fn parse_latency(input: &str) -> IResult<&str, isize> {
        let (input, _) = skip_to_tag("Network - RTT: ")(input)?;
        // Parse digits possibly with commas (e.g., "1,234")
        let (input, digits_str) = take_while1(|c: char| c.is_ascii_digit() || c == ',')(input)?;
        let clean: String = digits_str.chars().filter(|c| *c != ',').collect();
        let latency = clean.parse().unwrap_or(-1);
        Ok((input, latency))
    }

    match parse_latency(input) {
        Ok((_, latency)) => ParseResult::Parsed(FGGameMessage::NetworkMetrics { latency }),
        Err(_) => ParseResult::None,
    }
}

fn set_client_readiness(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameManager] Setting this client as readiness state") {
        return ParseResult::None;
    }

    fn parse_state(input: &str) -> IResult<&str, &str> {
        let (input, _) = skip_to_tag("readiness state '")(input)?;
        take_while1(|c: char| c.is_alphanumeric())(input)
    }

    match parse_state(input) {
        Ok((_, state_str)) => {
            let state = state_str
                .parse::<FGClientReadinessState>()
                .unwrap_or(FGClientReadinessState::Unknown(state_str.to_owned()));
            ParseResult::Parsed(FGGameMessage::SetClientReadiness(state))
        }
        Err(_) => ParseResult::None,
    }
}

fn loaded_round(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[StateGameLoading] Finished loading game level,") {
        return ParseResult::None;
    }

    fn parse_level(input: &str) -> IResult<&str, &str> {
        let (input, _) = skip_to_tag("assumed to be ")(input)?;
        take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-')(input)
    }

    match parse_level(input) {
        Ok((_, level_str_id)) => {
            ParseResult::Parsed(FGGameMessage::LoadedRound(FGRoundInfo::from_str_id(level_str_id)))
        }
        Err(_) => ParseResult::None,
    }
}

fn request_local_player(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("Requesting spawn of local player, ID=") {
        return ParseResult::None;
    }

    fn parse_id(input: &str) -> IResult<&str, isize> {
        let (input, _) = skip_to_tag("ID=")(input)?;
        parse_isize(input)
    }

    match parse_id(input) {
        Ok((_, id)) => ParseResult::Parsed(FGGameMessage::RequestLocalPlayer(id)),
        Err(_) => ParseResult::None,
    }
}

fn handle_remote_player(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameManager] Handling bootstrap for remote player ") {
        return ParseResult::None;
    }

    fn parse_remote_player(input: &str) -> IResult<&str, (isize, isize, Option<isize>)> {
        let (input, _) = skip_to_tag("Handling bootstrap for remote player ")(input)?;
        // Skip player name until [
        let (input, _) = take_until("[")(input)?;
        let (input, _) = char('[')(input)?;
        let (input, net_id) = parse_isize(input)?;
        let (input, _) = skip_to_tag("playerID = ")(input)?;
        let (input, player_id) = parse_isize(input)?;
        let (input, _) = tag(", squadID = ")(input)?;
        let (input, squad_id) = parse_isize(input)?;
        Ok((input, (player_id, net_id, Some(squad_id))))
    }

    match parse_remote_player(input) {
        Ok((_, (player_id, net_player_id, squad_id))) => {
            ParseResult::Parsed(FGGameMessage::HandleRemotePlayer {
                player_id,
                net_player_id,
                squad_id,
            })
        }
        Err(_) => ParseResult::None,
    }
}

fn handle_local_player(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameManager] Handling bootstrap for local player ") {
        return ParseResult::None;
    }

    fn parse_local_player(input: &str) -> IResult<&str, (isize, isize, Option<isize>)> {
        let (input, _) = skip_to_tag("Handling bootstrap for local player ")(input)?;
        // Skip player name until [
        let (input, _) = take_until("[")(input)?;
        let (input, _) = char('[')(input)?;
        let (input, net_id) = parse_isize(input)?;
        let (input, _) = skip_to_tag("playerID = ")(input)?;
        let (input, player_id) = parse_isize(input)?;
        let (input, _) = tag(", squadID = ")(input)?;
        let (input, squad_id) = parse_isize(input)?;
        Ok((input, (player_id, net_id, Some(squad_id))))
    }

    match parse_local_player(input) {
        Ok((_, (player_id, net_player_id, squad_id))) => {
            ParseResult::Parsed(FGGameMessage::HandleLocalPlayer {
                player_id,
                net_player_id,
                squad_id,
            })
        }
        Err(_) => ParseResult::None,
    }
}

fn append_spectator_target(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[CameraDirector] Adding Spectator target") {
        return ParseResult::None;
    }

    fn parse_spectator(
        input: &str,
    ) -> IResult<&str, (isize, FGPlatform, Option<isize>, Option<isize>)> {
        let (input, _) = skip_to_tag("Adding Spectator target ")(input)?;
        // Skip to (platform)
        let (input, _) = take_until("(")(input)?;
        let (input, platform_str) =
            delimited(char('('), take_while1(|c: char| c != ')'), char(')')).parse(input)?;
        let (input, _) = skip_to_tag("Party ID: ")(input)?;
        // Party ID may be empty or have spaces
        let (input, party_str) = take_while(|c: char| c.is_ascii_digit() || c == ' ')(input)?;
        let (input, _) = skip_to_tag("Squad ID: ")(input)?;
        let (input, squad_str) = take_while(|c: char| c.is_ascii_digit() || c == ' ')(input)?;
        let (input, _) = skip_to_tag("playerID: ")(input)?;
        let (input, player_id) = parse_isize(input)?;

        let platform = FGPlatform::from_str(platform_str).unwrap();

        fn parse_optional_id(s: &str) -> Option<isize> {
            let trimmed: String = s.chars().filter(|c| !c.is_whitespace()).collect();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse().ok()
            }
        }

        let party_id = parse_optional_id(party_str);
        let squad_id = parse_optional_id(squad_str);

        Ok((input, (player_id, platform, squad_id, party_id)))
    }

    match parse_spectator(input) {
        Ok((_, (player_id, platform, squad_id, party_id))) => {
            ParseResult::Parsed(FGGameMessage::AppendSpectatorTarget {
                player_id,
                squad_id,
                party_id,
                platform,
            })
        }
        Err(_) => ParseResult::None,
    }
}

fn player_spawned(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[StateGameLoading] OnPlayerSpawned - NetID") {
        return ParseResult::None;
    }

    fn parse_spawned(input: &str) -> IResult<&str, (isize, isize)> {
        let (input, _) = skip_to_tag("OnPlayerSpawned - NetID=")(input)?;
        let (input, net_player_id) = parse_isize(input)?;
        let (input, _) = tag(" ID=")(input)?;
        let (input, player_id) = parse_isize(input)?;
        Ok((input, (player_id, net_player_id)))
    }

    match parse_spawned(input) {
        Ok((_, (player_id, net_player_id))) => ParseResult::Parsed(FGGameMessage::PlayerSpawned {
            player_id,
            net_player_id,
        }),
        Err(_) => ParseResult::None,
    }
}

fn set_local_squad_id(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[CameraDirector] Set Local Squad ID: ") {
        return ParseResult::None;
    }

    fn parse_squad_id(input: &str) -> IResult<&str, Option<isize>> {
        let (input, _) = skip_to_tag("Set Local Squad ID: ")(input)?;
        opt(parse_isize).parse(input)
    }

    match parse_squad_id(input) {
        Ok((_, squad_id)) => ParseResult::Parsed(FGGameMessage::SetLocalSquadId(squad_id)),
        Err(_) => ParseResult::None,
    }
}

fn set_local_party_id(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[CameraDirector] Set Local Party ID: ") {
        return ParseResult::None;
    }

    fn parse_party_id(input: &str) -> IResult<&str, Option<isize>> {
        let (input, _) = skip_to_tag("Set Local Party ID: ")(input)?;
        opt(parse_isize).parse(input)
    }

    match parse_party_id(input) {
        Ok((_, party_id)) => ParseResult::Parsed(FGGameMessage::SetLocalPartyId(party_id)),
        Err(_) => ParseResult::None,
    }
}

fn game_session_state(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[GameSession] Changing state from ") {
        return ParseResult::None;
    }

    fn parse_session_state(input: &str) -> IResult<&str, (&str, &str)> {
        let (input, _) = skip_to_tag("Changing state from ")(input)?;
        let (input, before) = identifier(input)?;
        let (input, _) = tag(" to ")(input)?;
        let (input, after) = identifier(input)?;
        Ok((input, (before, after)))
    }

    match parse_session_state(input) {
        Ok((_, (before_str, after_str))) => {
            let before = FGGameSessionState::from_str(before_str).ok();
            let after = FGGameSessionState::from_str(after_str).unwrap();
            ParseResult::Parsed(FGGameMessage::GameSessionState { after, before })
        }
        Err(_) => ParseResult::None,
    }
}

fn set_num_players_achieving_objective(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameSession] NumPlayersAchievingObjective=") {
        return ParseResult::None;
    }

    fn parse_num_players(input: &str) -> IResult<&str, isize> {
        let (input, _) = skip_to_tag("NumPlayersAchievingObjective=")(input)?;
        parse_isize(input)
    }

    match parse_num_players(input) {
        Ok((_, num)) => ParseResult::Parsed(FGGameMessage::SetNumPlayersAchievingObjective(num)),
        Err(_) => ParseResult::None,
    }
}

fn set_player_score(input: &str) -> ParseResult<FGGameMessage> {
    if !(input.contains("Player ") && input.contains(" score = ")) {
        return ParseResult::None;
    }

    fn parse_score(input: &str) -> IResult<&str, (isize, isize)> {
        let (input, _) = skip_to_tag("Player ")(input)?;
        let (input, net_player_id) = parse_isize(input)?;
        let (input, _) = tag(" score = ")(input)?;
        let (input, score) = parse_isize(input)?;
        Ok((input, (net_player_id, score)))
    }

    match parse_score(input) {
        Ok((_, (net_player_id, score))) => {
            ParseResult::Parsed(FGGameMessage::SetPlayerScore { net_player_id, score })
        }
        Err(_) => ParseResult::None,
    }
}

fn handle_unspawn(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameManager] Handling unspawn for player ") {
        return ParseResult::None;
    }

    fn parse_unspawn(input: &str) -> IResult<&str, isize> {
        let (input, _) = skip_to_tag("Handling unspawn for player ")(input)?;
        parse_isize(input)
    }

    match parse_unspawn(input) {
        Ok((_, net_player_id)) => ParseResult::Parsed(FGGameMessage::HandleUnspawn(net_player_id)),
        Err(_) => ParseResult::None,
    }
}

fn set_player_progress(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("ClientGameManager::HandleServerPlayerProgress PlayerId=") {
        return ParseResult::None;
    }

    fn parse_progress(input: &str) -> IResult<&str, (isize, bool)> {
        let (input, _) = skip_to_tag("HandleServerPlayerProgress PlayerId=")(input)?;
        let (input, player_id) = parse_isize(input)?;
        let (input, _) = tag(" is succeeded=")(input)?;
        let (input, succeeded_str) = alt((tag("True"), tag("False"))).parse(input)?;
        Ok((input, (player_id, succeeded_str == "True")))
    }

    match parse_progress(input) {
        Ok((_, (player_id, is_succeeded))) => {
            ParseResult::Parsed(FGGameMessage::SetPlayerProgress {
                player_id,
                is_succeeded,
            })
        }
        Err(_) => ParseResult::None,
    }
}

fn game_lobby_rewards(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains(" [CompletedEpisodeDto] ") {
        return ParseResult::None;
    }

    let is_out_of_scope = |text: &str| {
        text.contains("[RewardService] Processing claimed rewards")
            || text.contains(".TryUseSpectatingPlayersShot")
            || text.contains("Exception")
    };

    let is_valid_log = |text: &str| {
        (text.contains("> ") && text.contains(":"))
            || text.contains("[Round")
            || text.contains("]")
            || text.contains("CompletedEpisodeDto")
    };

    if !is_out_of_scope(input) {
        let last_line = input.lines().last().unwrap();
        if is_valid_log(last_line)
            || input.lines().count() < (6 + 2)
            || (!is_valid_log(last_line)
                && input
                    .lines()
                    .filter(|line| !is_valid_log(line))
                    .map(|line| line.len())
                    .min()
                    .unwrap()
                    .abs_diff(last_line.len())
                    < 5)
        {
            return ParseResult::NeedMoreLines;
        }
    }

    /// Parse round title: [Round N | round_id]
    fn parse_round_title(input: &str) -> IResult<&str, (isize, &str)> {
        let (input, _) = take_until("[Round ")(input)?;
        let (input, _) = tag("[Round ")(input)?;
        let (input, order) = parse_isize(input)?;
        let (input, _) = tag(" | ")(input)?;
        let (input, round_id) = take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-')(input)?;
        Ok((input, (order, round_id)))
    }

    /// Parse property: > Key: Value
    fn parse_property(input: &str) -> IResult<&str, (&str, &str)> {
        let (input, _) = take_until("> ")(input)?;
        let (input, _) = tag("> ")(input)?;
        let (input, key) = take_while1(|c: char| c.is_alphanumeric() || c == ' ' || c == '_' || c == '-')(input)?;
        let (input, _) = tag(": ")(input)?;
        let (input, value) = take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-')(input)?;
        Ok((input, (key.trim(), value)))
    }

    let mut kudos: Option<isize> = None;
    let mut fame: Option<isize> = None;
    let mut crowns: Option<isize> = None;
    let mut current_crown_shards: Option<isize> = None;
    let mut rounds: Vec<FGCompletedEpisodeDtoRound> = Vec::new();

    let mut round_order: isize = -1;
    let mut temp_round = generate_fg_completed_episode_dto_round();

    for line in input.lines() {
        if is_out_of_scope(line) {
            break;
        }

        if round_order > 0 && line.contains(" [CompletedEpisodeDto] ") {
            warn!("Duplicated log detected. ignoring next dto payload");
            break;
        }

        // Try to parse round title
        if line.contains("[Round ") && line.contains(" | ") {
            if round_order != -1 {
                rounds.push(temp_round);
                temp_round = generate_fg_completed_episode_dto_round();
            }
            if let Ok((_, (order, round_id_str))) = parse_round_title(line) {
                round_order = order;
                temp_round.round_order = order;
                temp_round.round_id_str = round_id_str.to_string();
                temp_round.round_display_name = localized_string_round_id(round_id_str);
            } else {
                warn!("line: {}", line);
                return ParseResult::None;
            }
        } else if line.contains("> ") && line.contains(": ") {
            if let Ok((_, (key, value))) = parse_property(line) {
                if round_order == -1 {
                    // Global properties
                    match key {
                        "Kudos" => kudos = value.parse().ok(),
                        "Fame" => fame = value.parse().ok(),
                        "Crowns" => crowns = value.parse().ok(),
                        "CurrentCrownShards" => current_crown_shards = value.parse().ok(),
                        _ => {}
                    }
                } else {
                    // Round properties
                    if value.is_empty() {
                        warn!("DTO: {key} is empty value.");
                        continue;
                    }
                    match key {
                        "Qualified" => temp_round.qualified = value == "True",
                        "Position" => {
                            if let Ok(v) = value.parse() {
                                temp_round.position = v;
                            }
                        }
                        "Team Score" => {
                            if let Ok(v) = value.parse() {
                                temp_round.team_score = v;
                            }
                        }
                        "Kudos" => {
                            if let Ok(v) = value.parse() {
                                temp_round.kudos = v;
                            }
                        }
                        "Fame" => {
                            if let Ok(v) = value.parse() {
                                temp_round.fame = v;
                            }
                        }
                        "Bonus Tier" => {
                            if let Ok(v) = value.parse() {
                                temp_round.bonus_tier = v;
                            }
                        }
                        "Bonus Kudos" => {
                            if let Ok(v) = value.parse() {
                                temp_round.bonus_kudos = v;
                            }
                        }
                        "Bonus Fame" => {
                            if let Ok(v) = value.parse() {
                                temp_round.bonus_fame = v;
                            }
                        }
                        "BadgeId" => {
                            if let Ok(v) = value.parse() {
                                temp_round.badge_id = v;
                            }
                        }
                        _ => {
                            warn!("Unknown key value {key}. value: {input}");
                            return ParseResult::None;
                        }
                    }
                }
            } else {
                warn!("out of scope: {}", line);
                break;
            }
        }
    }

    // Push the last round if not already pushed
    if !rounds.is_empty() && rounds.last().unwrap().round_order != temp_round.round_order {
        rounds.push(temp_round);
    } else if rounds.is_empty() && round_order != -1 {
        rounds.push(temp_round);
    }

    ParseResult::Parsed(FGGameMessage::GameLobbyRewards(FGCompletedEpisodeDto {
        kudos,
        fame,
        crowns,
        current_crown_shards,
        rounds,
    }))
}

fn creative_round_loader(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[RoundLoader] Load UGC via share code: ") {
        return ParseResult::None;
    }

    fn parse_share_code(input: &str) -> IResult<&str, &str> {
        let (input, _) = skip_to_tag("Load UGC via share code: ")(input)?;
        // Parse code like 0275-8967-0239:version
        take_while1(|c: char| c.is_ascii_digit() || c == '-')(input)
    }

    match parse_share_code(input) {
        Ok((_, code)) => {
            ParseResult::Parsed(FGGameMessage::CreativeRoundLoader(code.to_owned()))
        }
        Err(_) => ParseResult::None,
    }
}
fn leave_match(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[LeaveMatchPopupManager] Calling CloseScreen()") {
        return ParseResult::None;
    }
    ParseResult::Parsed(FGGameMessage::LeaveMatch)
}
fn round_over(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameManager] Server notifying that the round is over.") {
        return ParseResult::None;
    }
    ParseResult::Parsed(FGGameMessage::RoundOver)
}
fn server_message_start_loading_level(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("GameMessageServerStartLoadingLevel received") {
        return ParseResult::None;
    }
    ParseResult::Parsed(FGGameMessage::ServerMessageStartLoadingLevel)
}
fn server_message_ready_round_response(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("GameMessageServerReadyRoundResponse received") {
        return ParseResult::None;
    }
    ParseResult::Parsed(FGGameMessage::ServerMessageReadyRoundResponse)
}
fn server_message_round_results(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("GameMessageServerRoundResults received") {
        return ParseResult::None;
    }
    ParseResult::Parsed(FGGameMessage::ServerMessageRoundResults)
}
fn server_message_end_round(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("GameMessageServerEndRound received") {
        return ParseResult::None;
    }
    ParseResult::Parsed(FGGameMessage::ServerMessageEndRound)
}

pub type FGParserRule = Arc<dyn Fn(&str) -> ParseResult<FGGameMessage> + Send + Sync>;
pub fn rules() -> Vec<FGParserRule> {
    vec![
        Arc::new(game_state_changed),
        Arc::new(begin_matchmaking),
        Arc::new(matchmaking_message_received),
        Arc::new(server_connected),
        Arc::new(create_local_player),
        Arc::new(successfully_joined),
        Arc::new(network_metrics),
        Arc::new(set_client_readiness),
        Arc::new(loaded_round),
        Arc::new(request_local_player),
        Arc::new(handle_remote_player),
        Arc::new(handle_local_player),
        Arc::new(append_spectator_target),
        Arc::new(player_spawned),
        Arc::new(set_local_squad_id),
        Arc::new(set_local_party_id),
        Arc::new(game_session_state),
        Arc::new(set_num_players_achieving_objective),
        Arc::new(set_player_score),
        Arc::new(handle_unspawn),
        Arc::new(set_player_progress),
        Arc::new(game_lobby_rewards),
        Arc::new(creative_round_loader),
        Arc::new(leave_match),
        Arc::new(round_over),
        Arc::new(server_message_start_loading_level),
        Arc::new(server_message_ready_round_response),
        Arc::new(server_message_round_results),
        Arc::new(server_message_end_round),
    ]
}
