use std::{str::FromStr, sync::Arc};

use crate::{
    extra_data::localized_string_round_id,
    models::{
        common::{FGPlatform, FGRoundInfo},
        dto::{
            FGCompletedEpisodeDto, FGCompletedEpisodeDtoRound,
            generate_fg_completed_episode_dto_round,
        },
        state::{FGClientReadinessState, FGGameSessionState},
    },
};
use crate::{
    models::messages::{FGGameMessage, FGMatchmakingMessage},
    parser::ParseResult,
};
use super::create_regex;

fn game_state_changed(input: &str) -> ParseResult<FGGameMessage> {
    if !(input.contains("[GameStateMachine] Replacing ") && input.contains(" with ")) {
        return ParseResult::None;
    }

    let re =
        create_regex(r"Replacing (?<before>[a-zA-Z0-9_\.-]+) with (?<after>[a-zA-Z0-9_\.-]+)?");
    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let before = caps.name("before").unwrap().as_str().parse().ok();
    let after = caps.name("after").unwrap().as_str().parse().unwrap();

    ParseResult::Parsed(FGGameMessage::GameStateChanged { before, after })
}
fn begin_matchmaking(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[Matchmaking] Begin matchmaking") {
        return ParseResult::None;
    }
    ParseResult::Parsed(FGGameMessage::BeginMatchmaking)
}
fn matchmaking_message_received(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[FNMMSClientRemoteService] Status message received:") {
        return ParseResult::None;
    } else if !input.contains("\"state\": ") {
        return ParseResult::NeedMoreLines;
    }

    let re = create_regex(
        r#"\"(?<key>[a-zA-Z0-9_\.-]+)\": (\"?)(?<value>(null|([a-zA-Z0-9_\.-]+)))(\"?)"#,
    );
    for line in input.lines() {
        let Some(caps) = re.captures(line) else {
            continue;
        };
        let key = caps.name("key").unwrap().as_str();
        let value = caps.name("value").unwrap().as_str();
        match key {
            "name" => match value {
                "Play" => {
                    return ParseResult::Parsed(FGGameMessage::MatchmakingMessageReceived(
                        FGMatchmakingMessage::Play,
                    ));
                }
                "Error" => {
                    return ParseResult::None;
                }
                _ => {}
            },
            "queuedPlayers" => match value {
                "null" => {}
                _ => {
                    if let Ok(queued_players) = value.parse() {
                        return ParseResult::Parsed(FGGameMessage::MatchmakingMessageReceived(
                            FGMatchmakingMessage::Queued { queued_players },
                        ));
                    }
                }
            },
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
    unreachable!()
}
fn server_connected(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[StateConnectToGame] InitiateNetworkConnectRequest with server IP: ") {
        return ParseResult::None;
    }
    let re = create_regex(
        r"InitiateNetworkConnectRequest with server IP: (?<ip>[0-9.]+):(?<port>[0-9]+)?",
    );
    let captures = re.captures(input).unwrap();
    let ip_addr = captures.name("ip").unwrap().as_str().to_owned();
    let port = captures.name("port").map(|m| m.as_str().to_owned());
    ParseResult::Parsed(FGGameMessage::ServerConnected { ip_addr, port })
}

fn create_local_player(input: &str) -> ParseResult<FGGameMessage> {
    if input.contains("[CreateLocalPlayerInstances] Added new player as Participant") {
        let re = create_regex(r"Added new player as Participant, player ID = (?<player_id>\d+)");
        let Some(caps) = re.captures(input) else {
            return ParseResult::None;
        };

        let player_id = caps.name("player_id").unwrap().as_str();

        return ParseResult::Parsed(FGGameMessage::CreateLocalPlayer(player_id.parse().unwrap()));
    }
    ParseResult::None
}

fn successfully_joined(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[HandleSuccessfulLogin] Selected show is ") {
        return ParseResult::None;
    } else if !input.contains("[HandleSuccessfulLogin] Session: ") {
        return ParseResult::NeedMoreLines;
    }

    let Some(selected_show) = create_regex(r"Selected show is (?<selected_show>[a-zA-Z0-9_]+)")
        .captures(input)
        .and_then(|caps| caps.name("selected_show"))
    else {
        return ParseResult::None;
    };
    let session = create_regex(r"Session: (?<session>[a-zA-Z0-9_-]+)")
        .captures(input)
        .and_then(|caps| caps.name("session"))
        .map(|matched| matched.as_str().to_owned());

    ParseResult::Parsed(FGGameMessage::SuccessfullyJoined {
        game_mode: selected_show.as_str().parse().unwrap(),
        session_text: session,
    })
}

fn network_metrics(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[FG_UnityInternetNetworkManager] Networking Metrics after") {
        return ParseResult::None;
    } else if !input.contains("Network - RTT: ") {
        return ParseResult::NeedMoreLines;
    }

    let re = create_regex(r"Network - RTT: (?<latency>[0-9,]+)ms");
    let Some(latency_str) = re
        .captures(input)
        .and_then(|caps| caps.name("latency"))
        .map(|matched| matched.as_str().to_owned())
    else {
        return ParseResult::Unreachable;
    };

    let latency = latency_str.replace(",", "").parse().unwrap_or(-1);

    ParseResult::Parsed(FGGameMessage::NetworkMetrics { latency })
}

fn set_client_readiness(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameManager] Setting this client as readiness state") {
        return ParseResult::None;
    }

    let re = create_regex(r"Setting this client as readiness state '(?<state>[a-zA-Z0-9]+)'");
    let Some(state_str) = re
        .captures(input)
        .and_then(|caps| caps.name("state"))
        .map(|matched| matched.as_str().to_owned())
    else {
        return ParseResult::Unreachable;
    };

    let state_parsed = state_str
        .parse::<FGClientReadinessState>()
        .unwrap_or(FGClientReadinessState::Unknown(state_str));

    ParseResult::Parsed(FGGameMessage::SetClientReadiness(state_parsed))
}

fn loaded_round(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[StateGameLoading] Finished loading game level,") {
        return ParseResult::None;
    }

    let re = create_regex(r"Finished loading game level, assumed to be (?<level>[a-zA-Z0-9_-]+)\.");
    let Some(level_str_id) = re
        .captures(input)
        .and_then(|caps| caps.name("level"))
        .map(|matched| matched.as_str().to_owned())
    else {
        return ParseResult::None;
    };

    ParseResult::Parsed(FGGameMessage::LoadedRound(FGRoundInfo::from_str_id(
        &level_str_id,
    )))
}

fn request_local_player(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("Requesting spawn of local player, ID=") {
        return ParseResult::None;
    }

    let re = create_regex(r"Requesting spawn of local player, ID=(?<id>[0-9]+)");
    let Some(id_str) = re
        .captures(input)
        .and_then(|caps| caps.name("id"))
        .map(|matched| matched.as_str().to_owned())
    else {
        return ParseResult::None;
    };

    ParseResult::Parsed(FGGameMessage::RequestLocalPlayer(id_str.parse().unwrap()))
}

fn handle_remote_player(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameManager] Handling bootstrap for remote player ") {
        return ParseResult::None;
    }

    let re = create_regex(
        r"Handling bootstrap for remote player (?<player_name>[\s\S]+) \[(?<net_id>[0-9]+)\] \((?<class>[\s\S]+)\), playerID = (?<player_id>[0-9]+), squadID = (?<squad_id>[0-9]+)",
    );

    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let player_id = caps.name("player_id").unwrap().as_str().parse().unwrap();
    let net_player_id = caps.name("net_id").unwrap().as_str().parse().unwrap();
    let squad_id = caps.name("squad_id").unwrap().as_str().parse().ok();

    ParseResult::Parsed(FGGameMessage::HandleRemotePlayer {
        player_id,
        net_player_id,
        squad_id,
    })
}

fn handle_local_player(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameManager] Requesting local player") {
        return ParseResult::None;
    }

    let re = create_regex(
        r"[ClientGameManager] Handling bootstrap for local player (?<player_name>[\s\S]+) \[(?<net_id>[0-9]+)\] \((?<class>[\s\S]+)\), playerID = (?<player_id>[0-9]+), squadID = (?<squad_id>[0-9]+)",
    );

    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let player_id = caps.name("player_id").unwrap().as_str().parse().unwrap();
    let net_player_id = caps.name("net_id").unwrap().as_str().parse().unwrap();
    let squad_id = caps.name("squad_id").unwrap().as_str().parse().ok();

    ParseResult::Parsed(FGGameMessage::HandleLocalPlayer {
        player_id,
        net_player_id,
        squad_id,
    })
}

fn append_spectator_target(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[CameraDirector] Adding Spectator target") {
        return ParseResult::None;
    }

    let re = create_regex(
        r"Adding Spectator target ([\s\S]+) \((?<platform>[\s\S]+)\) with Party ID: (?<party_id>[0-9 ]+) Squad ID: (?<squad_id>[0-9]+) and playerID: (?<player_id>[0-9]+)",
    );

    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let player_id = caps.name("player_id").unwrap().as_str().parse().unwrap();
    let platform = FGPlatform::from_str(caps.name("platform").unwrap().as_str()).unwrap();

    fn wrap_nowhitespace(value: &str) -> Option<String> {
        let value = value.replace(" ", "");
        if !value.is_empty() { Some(value) } else { None }
    }

    let squad_id =
        wrap_nowhitespace(caps.name("squad_id").unwrap().as_str()).map(|str| str.parse().unwrap());
    let party_id =
        wrap_nowhitespace(caps.name("party_id").unwrap().as_str()).map(|str| str.parse().unwrap());

    ParseResult::Parsed(FGGameMessage::AppendSpectatorTarget {
        player_id,
        squad_id,
        party_id,
        platform,
    })
}

fn player_spawned(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[StateGameLoading] OnPlayerSpawned - NetID") {
        return ParseResult::None;
    }

    let re = create_regex(
        r"OnPlayerSpawned - NetID=(?<net_player_id>[0-9]+) ID=(?<player_id>[0-9]+) was spawned",
    );

    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let player_id = caps.name("player_id").unwrap().as_str().parse().unwrap();
    let net_player_id = caps
        .name("net_player_id")
        .unwrap()
        .as_str()
        .parse()
        .unwrap();

    ParseResult::Parsed(FGGameMessage::PlayerSpawned {
        player_id,
        net_player_id,
    })
}

fn set_local_squad_id(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[CameraDirector] Set Local Squad ID: ") {
        return ParseResult::None;
    }

    let re = create_regex(r"Set Local Squad ID: (?<player_id>[0-9]+)?");

    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let player_id = caps.name("player_id").map(|s| s.as_str().parse().unwrap());

    ParseResult::Parsed(FGGameMessage::SetLocalSquadId(player_id))
}

fn set_local_party_id(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[CameraDirector] Set Local Party ID: ") {
        return ParseResult::None;
    }

    let re = create_regex(r"Set Local Party ID: (?<party_id>[0-9]+)?");

    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let party_id = caps.name("party_id").map(|s| s.as_str().parse().unwrap());

    ParseResult::Parsed(FGGameMessage::SetLocalPartyId(party_id))
}

fn game_session_state(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[GameSession] Changing state from ") {
        return ParseResult::None;
    }

    let re =
        create_regex(r"Changing state from (?<before>[a-zA-Z0-9_-]+) to (?<after>[a-zA-Z0-9_-]+)?");

    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let before = caps
        .name("before")
        .map(|s| s.as_str().to_owned())
        .and_then(|str| FGGameSessionState::from_str(&str).ok());

    let after = caps
        .name("after")
        .map(|s| s.as_str().to_owned())
        .and_then(|str| FGGameSessionState::from_str(&str).ok())
        .unwrap();

    ParseResult::Parsed(FGGameMessage::GameSessionState { after, before })
}

fn set_num_players_achieving_objectsive(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameSession] NumPlayersAchievingObjective=") {
        return ParseResult::None;
    }

    let re = create_regex(r" NumPlayersAchievingObjective=(?<num_players>[0-9]+)");
    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let num_players = caps
        .name("num_players")
        .map(|s| s.as_str().to_owned())
        .and_then(|str| str.parse().ok())
        .unwrap();

    ParseResult::Parsed(FGGameMessage::SetNumPlayersAchievingObjective(num_players))
}

fn set_player_score(input: &str) -> ParseResult<FGGameMessage> {
    if !(input.contains("Player ") && input.contains(" score = ")) {
        return ParseResult::None;
    }

    let re = create_regex(r"Player (?<net_player_id>[0-9]+)? score = (?<score>[0-9]+)?");

    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let net_player_id = caps
        .name("net_player_id")
        .map(|s| s.as_str().to_owned())
        .and_then(|str| str.parse().ok())
        .unwrap();

    let score = caps
        .name("score")
        .map(|s| s.as_str().to_owned())
        .and_then(|str| str.parse().ok())
        .unwrap();

    ParseResult::Parsed(FGGameMessage::SetPlayerScore {
        net_player_id,
        score,
    })
}

fn handle_unspawn(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("[ClientGameManager] Handling unspawn for player ") {
        return ParseResult::None;
    }

    let re = create_regex(r"Handling unspawn for player (?<net_player_id>[0-9]+)");

    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let net_player_id = caps
        .name("net_player_id")
        .map(|s| s.as_str().to_owned())
        .and_then(|str| str.parse().ok())
        .unwrap();

    ParseResult::Parsed(FGGameMessage::HandleUnspawn(net_player_id))
}

fn set_player_progress(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains("ClientGameManager::HandleServerPlayerProgress PlayerId=") {
        return ParseResult::None;
    }

    let re = create_regex(
        r"HandleServerPlayerProgress PlayerId=(?<player_id>[0-9]+) is succeeded=(?<is_succeeded>True|False)",
    );

    let Some(caps) = re.captures(input) else {
        return ParseResult::None;
    };

    let player_id = caps
        .name("player_id")
        .map(|s| s.as_str())
        .and_then(|str| str.parse().ok())
        .unwrap();

    let is_succeeded = caps
        .name("is_succeeded")
        .map(|s| s.as_str().to_owned())
        .unwrap();

    ParseResult::Parsed(FGGameMessage::SetPlayerProgress {
        player_id,
        is_succeeded: is_succeeded == "True",
    })
}

fn game_lobby_rewards(input: &str) -> ParseResult<FGGameMessage> {
    if !input.contains(" [CompletedEpisodeDto] ") {
        return ParseResult::None;
    }

    // Parse before "Processing claimed rewards"
    let is_out_of_scope = |text: &str| {
        text.contains("[RewardService] Processing claimed rewards")
            || text.contains(".TryUseSpectatingPlayersShot")
            || text.contains("Exception")
    };

    let is_valid_log = |text: &str| {
        (text.contains("> ") && text.contains(":"))
            || (text.contains("[Round") || text.contains("]"))
            || text.contains("CompletedEpisodeDto")
    };
    if !is_out_of_scope(input) {
        // We assume that the first blank line(not valid log)'s length are same thus, we can use it as a reference.
        let last_line = input.lines().last().unwrap();
        if is_valid_log(last_line)
            // Parse first 6 line (which is always shown.) then extra 2 for round #0.
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

    let mut kudos: Option<isize> = None;
    let mut fame: Option<isize> = None;
    let mut crowns: Option<isize> = None;
    let mut current_crown_shards: Option<isize> = None;
    let mut rounds: Vec<FGCompletedEpisodeDtoRound> = Vec::new();

    let mut round_order = -1;
    let mut temp_round = generate_fg_completed_episode_dto_round();

    let title_regex =
        create_regex(r"\[Round (?<order>[0-9]+) \| (?<round_id_str>[a-zA-Z0-9_-]+)\]");
    let prop_regex = create_regex(r"> (?<key>[a-zA-Z0-9 _-]+): (?<value>[a-zA-Z0-9_-]?+)");
    for line in input.lines() {
        // For getting sure that it is out of scope.
        if is_out_of_scope(line) {
            break;
        }
        if line.contains("[") && line.contains("Round ") && line.contains("]") {
            if round_order != -1 {
                rounds.push(temp_round);
                temp_round = generate_fg_completed_episode_dto_round();
            }
            let Some(caps) = title_regex.captures(line) else {
                println!("line: {}", line);
                return ParseResult::Unreachable;
            };
            let order = caps.name("order").unwrap().as_str().parse().unwrap();
            let round_id_str = caps.name("round_id_str").unwrap().as_str().to_string();
            round_order = order;
            temp_round.round_order = order;
            temp_round.round_id_str = round_id_str.clone();
            temp_round.round_display_name = localized_string_round_id(&round_id_str);
        } else if line.contains("> ") && line.contains(": ") {
            let Some(caps) = prop_regex.captures(line) else {
                println!("line: {}", line);
                return ParseResult::Unreachable;
            };
            let key = caps.name("key").unwrap().as_str();
            let value = caps.name("value").unwrap().as_str();

            if round_order == -1 {
                match key {
                    "Kudos" => kudos = Some(value.parse().unwrap()),
                    "Fame" => fame = Some(value.parse().unwrap()),
                    "Crowns" => crowns = Some(value.parse().unwrap()),
                    "CurrentCrownShards" => current_crown_shards = Some(value.parse().unwrap()),
                    _ => {}
                }
            } else {
                if value.is_empty() {
                    println!("DTO: {key} is empty value.");
                    continue;
                }
                match key {
                    "Qualified" => {
                        temp_round.qualified = value.parse::<String>().unwrap() == "True"
                    }
                    "Position" => temp_round.position = value.parse().unwrap(),
                    "Team Score" => temp_round.team_score = value.parse().unwrap(),
                    "Kudos" => temp_round.kudos = value.parse().unwrap(),
                    "Fame" => temp_round.fame = value.parse().unwrap(),
                    "Bonus Tier" => temp_round.bonus_tier = value.parse().unwrap(),
                    "Bonus Kudos" => temp_round.bonus_kudos = value.parse().unwrap(),
                    "Bonus Fame" => temp_round.bonus_fame = value.parse().unwrap(),
                    "BadgeId" => temp_round.badge_id = value.parse().unwrap(),
                    _ => unreachable!("Unknown key value {key}"),
                }
            }
        }
    }

    // Push the last rounds if not same.
    if !rounds.is_empty() && rounds.last().unwrap().round_order != temp_round.round_order {
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

    let re = create_regex(r" Load UGC via share code: (?<code>[0-9-]+):(?<version>[0-9]+)");
    if let Some(captures) = re.captures(input) {
        let code = captures["code"].parse().unwrap();
        ParseResult::Parsed(FGGameMessage::CreativeRoundLoader(code))
    } else {
        ParseResult::None
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
        Arc::new(set_num_players_achieving_objectsive),
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
