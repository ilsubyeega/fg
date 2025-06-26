import { Show, type Component } from "solid-js";
import "./ingame.scss";

export type TitleProps = {
  title: string;
  isCreative: boolean;
  creativeCode?: string;
};

export const defaultTitleProps: TitleProps = {
  title: "Round Title",
  isCreative: true,
  creativeCode: "1234-5678-ABCD-EFGH",
};

export type AttributesProps = {
  isAvailable: boolean;
  playersCount: number;
  timeText: string;
  timeMaxText?: string;
  timeLeftText?: string;
  qualifiedPlayers?: number;
  qualifiedMaxPlayers?: number;
  ping?: number;
};

export const defaultAttributesProps: AttributesProps = {
  isAvailable: true,
  playersCount: 10,
  timeText: "00:30",
  timeMaxText: "01:00",
  timeLeftText: "00:30",
  qualifiedPlayers: 5,
  qualifiedMaxPlayers: 10,
  ping: 4,
};

export const Header: Component<{
  titleProps: TitleProps;
  attributesProps: AttributesProps;
}> = (props) => {
  return (
    <div class="common-games-ingame-header">
      <Title {...props.titleProps} />
      <Attributes {...props.attributesProps} />
    </div>
  );
};

export const Title: Component<TitleProps> = (props) => {
  return (
    <div
      class="common-games-ingame-title"
      classList={{
        "is-creative": props.isCreative,
      }}
    >
      <div class="creative-wrapped">
        <div class="creative">CREATIVE</div>
      </div>
      <div class="title">{props.title}</div>
      <div class="creative-code">{props.creativeCode ?? ""}</div>
    </div>
  );
};

export const Attributes: Component<AttributesProps> = (props) => {
  return (
    <div
      class="common-games-ingame-attributes"
      classList={{
        "is-available": props.isAvailable,
      }}
    >
      <div class="players">
        <img src="/icons/lucide-icons/ingame/players.svg" />
        {props.playersCount}
      </div>
      <div class="times">
        <img src="/icons/lucide-icons/ingame/time.svg" />
        <div class="time">
          {!props.timeMaxText
            ? props.timeText
            : `${props.timeText} / ${props.timeMaxText}`}
        </div>
        <Show when={props.timeLeftText}>
          <div class="time-left">-{props.timeLeftText}</div>
        </Show>
      </div>
      <Show when={props.qualifiedPlayers}>
        <div class="qualified">
          <img src="/icons/lucide-icons/ingame/qualified.svg" />
          {!props.qualifiedMaxPlayers
            ? props.qualifiedPlayers
            : `${props.qualifiedPlayers} / ${props.qualifiedMaxPlayers}`}
        </div>
      </Show>
      <Show when={props.ping}>
        <div class="ping">
          <img src="/icons/lucide-icons/ingame/ping.svg" />
          {props.ping} ms
        </div>
      </Show>
    </div>
  );
};
