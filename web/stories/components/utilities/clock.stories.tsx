import type { Meta } from "@kachurun/storybook-solid-vite";
import { Clock } from "~/components/utilities/clock";

export default {
  title: "Components/Utilities/Clock",
  component: Clock,
} satisfies Meta<typeof Clock>;

export const NowUTC9 = {
  name: "Now with UTC+9",
  args: {
    now: {
      timezone: 9 * 60,
    },
  },
};

export const Now12HUTC9 = {
  name: "Now (12h) with UTC+9",
  args: {
    now: {
      timezone: 9 * 60,
    },
    format: {
      format12h: true,
    },
  },
};

export const DeltaAfter10Seconds = {
  name: "Delta after 10 seconds",
  args: {
    delta: {
      time: new Date().getTime() + 1000 * 10,
    },
  },
};

export const DeltaAfter10SecondsPlus = {
  name: "Delta after 10 seconds (Plus Formatted)",
  args: {
    delta: {
      time: new Date().getTime() + 1000 * 10,
    },
    format: {
      formatPlus: true,
    },
  },
};

export const DeltaAfter1Minute = {
  name: "Delta after 1 minute",
  args: {
    delta: {
      time: new Date().getTime() + 1000 * 60,
    },
  },
};

export const DeltaAfter1Hour = {
  name: "Delta after 1 hour",
  args: {
    delta: {
      time: new Date().getTime() + 1000 * 60 * 60,
    },
  },
};
