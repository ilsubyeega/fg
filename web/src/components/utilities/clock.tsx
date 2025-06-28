import {
  Component,
  createSignal,
  For,
  Index,
  onCleanup,
  onMount,
} from "solid-js";

import "./clock.scss";
export interface ClockProps {
  /// The Current time
  now?: {
    /// Timezone in minutes.
    // Is undefined when auto.
    timezone?: number;
  };
  /// The delta time; such like '-0.01s'
  delta?: {
    // Unix timestamp, in milliseconds.
    time: number;
    // Unix timestamp, in milliseconds.
    end_time?: number;
  };
  format?: {
    format12h?: boolean;
    formatPlus?: boolean;
  };
  style?: {
    numWidth: string;
    decWidth: string;
  };
}

export const Clock: Component<ClockProps> = (props) => {
  const [content, setContent] = createSignal(["-"]);
  let interval: ReturnType<typeof setInterval>;

  onMount(() => {
    interval = setInterval(() => {
      setContent(processFormat(props).split(""));
    }, 50);
  });
  onCleanup(() => interval && clearInterval(interval));

  return (
    <p
      class="utilities-clock"
      style={{
        "--clock-num-width": props.style?.numWidth,
        "--clock-dec-width": props.style?.decWidth,
      }}
    >
      <For each={content()}>
        {(item) => {
          let className;
          if (!Number.isNaN(parseInt(item))) {
            className = "num";
          } else if (item.toUpperCase() != item.toLowerCase()) {
            className = "char";
          } else {
            className = "dec";
          }

          return <span class={className}>{item}</span>;
        }}
      </For>
    </p>
  );
};

const processFormat = (clockprops: ClockProps) => {
  if (clockprops.now) {
    const date = new Date();
    // If not set, detect automatically.
    const timezone = clockprops.now.timezone ?? date.getTimezoneOffset();

    let timezoned_time = date.getTime() + timezone * 1000 * 60;
    const extracted = extractFromMillis(timezoned_time);
    let suffix = "";
    if (clockprops.format?.format12h) {
      if (extracted.hours < 12) {
        suffix = " AM";
      } else {
        suffix = " PM";
        extracted.hours -= 12;
      }
    }

    return formatExtracted(extracted) + suffix;
  }
  if (clockprops.delta) {
    const time = clockprops.delta.end_time ?? new Date().getTime();
    const delta = time - clockprops.delta.time;
    const prefix = delta < 0 ? "-" : clockprops.format?.formatPlus ? "+" : "";

    return prefix + formatMilliseconds(delta);
  }

  return "Invalid Clock";
};

const formatMilliseconds = (duration_ms: number) =>
  formatExtracted(extractFromMillis(duration_ms));

const formatExtracted = (extracted: ReturnType<typeof extractFromMillis>) => {
  const { hours, minutes, seconds, millis } = extracted;
  let text = "";
  if (hours != 0) {
    text += hours.toString().padStart(2, "0") + ":";
  }
  if (hours != 0 || minutes != 0) {
    text += minutes.toString().padStart(2, "0") + ":";
  }
  text += seconds.toString().padStart(2, "0") + ".";
  text += Math.abs(Math.trunc(millis / 100)).toString();

  return text;
};

const extractFromMillis = (millis: number) => {
  return {
    hours: Math.abs(Math.trunc((millis / 1000 / 60 / 60) % 24)),
    minutes: Math.abs(Math.trunc((millis / 1000 / 60) % 60)),
    seconds: Math.abs(Math.trunc((millis / 1000) % 60)),
    millis: millis % 1000,
  };
};
