import { JSXElement, Show, type Component } from "solid-js";

const RenderItem: Component<{
  className: string;
  icon: string;
  title: JSXElement | string;
  value: JSXElement;
  extra_value?: JSXElement;
}> = (props) => {
  return (
    <div class={props.className}>
      <img src={props.icon} />
      <div class="title">{props.title}</div>
      <div class="wrapper">
        <div class="value">
          {props.value}
          <Show when={props.extra_value}>
            <div class="extra-value">{props.extra_value}</div>
          </Show>
        </div>
      </div>
    </div>
  );
};

export const RoundStats: Component<{
  qualifiedCount?: number;
  playCount?: number;
  goldCount?: number;
  personalBest?: string;
  personalBestDiff?: number;

  /// Total rounds played of this round mode.
  total_rounds?: number;
}> = (props) => {
  const percentageString = (value?: number, from?: number) => {
    if (!value || !from || from == 0) return "-";
    return ((value / from) * 100).toFixed(1) + "%";
  };
  return (
    <div class="common-games-stats-round-stats">
      <Show when={props.qualifiedCount}>
        <RenderItem
          className="qualified"
          title="Qualified"
          icon="/icons/lucide-icons/stats/qualified.svg"
          value={
            <p>
              {props.qualifiedCount}
              <Show when={props.total_rounds}>
                {" / " + props.total_rounds}
              </Show>
            </p>
          }
          extra_value={percentageString(
            props.qualifiedCount,
            props.total_rounds,
          )}
        />
      </Show>
      <Show when={props.goldCount}>
        <RenderItem
          className="gold"
          title="Gold"
          icon="/icons/lucide-icons/stats/gold.svg"
          value={
            <p>
              {props.goldCount}
              <Show when={props.total_rounds}>
                {" / " + props.total_rounds}
              </Show>
            </p>
          }
          extra_value={percentageString(props.goldCount, props.total_rounds)}
        />
      </Show>
      <Show when={props.personalBest}>
        <RenderItem
          className="personal-best"
          title="Personal Best"
          icon="/icons/lucide-icons/stats/personal-best.svg"
          value={props.personalBest}
          extra_value={
            !props.personalBestDiff
              ? undefined
              : (props.personalBestDiff > 0 ? "+" : "-") +
                `(${props.personalBestDiff.toFixed(1)}s)`
          }
        />
      </Show>
    </div>
  );
};

export const GameStats: Component<{
  played: number;
  wins: {
    count: number;
    streak?: number;
    streak_best?: number;
  };
  finals: {
    count: number;
    streak?: number;
    streak_best?: number;
  };
}> = (props) => {
  const extraValueWithStreaks = (
    count: number,
    played: number,
    streak?: number,
    streak_best?: number,
  ) => {
    if (played == 0) return undefined;
    const percentage = ((count / played) * 100).toFixed(1);
    const streakText =
      streak == streak_best || streak_best == undefined
        ? streak
        : `${streak}/${streak_best}`;
    return (
      <p class="extra-value">
        {percentage}%
        <Show when={streak}>
          <span class="streak">
            <img src="/icons/lucide-icons/streaks.svg" />
            {streakText}
          </span>
        </Show>
      </p>
    );
  };

  return (
    <div class="common-games-stats-game-stats">
      <RenderItem
        className="wins"
        title="Wins"
        icon="/icons/lucide-icons/stats/wins.svg"
        value={
          <p>
            {props.wins.count} / {props.played}
          </p>
        }
        extra_value={extraValueWithStreaks(
          props.wins.count,
          props.played,
          props.wins.streak,
          props.wins.streak_best,
        )}
      />
      <RenderItem
        className="finals"
        title="Finals"
        icon="/icons/lucide-icons/stats/finals.svg"
        value={
          <p>
            {props.finals.count} / {props.played}
          </p>
        }
        extra_value={extraValueWithStreaks(
          props.finals.count,
          props.played,
          props.finals.streak,
          props.finals.streak_best,
        )}
      />
    </div>
  );
};
