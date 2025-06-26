import { Component } from "solid-js";

export const Mattchmaking: Component<{
  isActive: boolean;
  status?: MatchmakingStatus;
  playerCount?: number;
  timeText?: string;
}> = (props) => {
  const matchmakingMessage = () => {
    switch (props.status) {
      case MatchmakingStatus.Connecting:
        return "Connecting...";
      case MatchmakingStatus.Queuefull:
        return "Queue full.";
      case MatchmakingStatus.Waiting:
        return "Matchmaking...";
      case MatchmakingStatus.Queued:
        return `Waiting for players...`;
      case MatchmakingStatus.SessionAssignment:
        return "Connecting to server...";
      case MatchmakingStatus.Play:
        return "Connecting to game...";
      default:
        console.warn("Unknown matchmaking status:", props.status);
        return "Unknown matchmaking status.";
    }
  };
  return (
    <div
      classList={{
        "common-games-matchmaking": true,
        active: props.isActive,
        "has-player": props.playerCount !== undefined,
        "has-times": props.timeText !== undefined,
      }}
    >
      <div class="message">{matchmakingMessage()}</div>
      <div class="extra">
        <div class="players">
          <div class="inner">
            <img src="/icons/lucide-icons/ingame/players.svg" />
            {props.playerCount ?? "?"}
          </div>
        </div>
        <div class="times">
          <div class="inner">
            <img src="/icons/lucide-icons/ingame/time.svg" />
            {props.timeText ?? "?"}
          </div>
        </div>
      </div>
    </div>
  );
};

export enum MatchmakingStatus {
  Connecting,
  Queuefull,
  Waiting,
  Queued,
  SessionAssignment,
  Play,
}
