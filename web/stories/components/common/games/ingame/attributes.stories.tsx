import { Attributes, defaultAttributesProps } from "~/components/common/games/ingame";
import type { Meta } from "@kachurun/storybook-solid-vite";

export default {
  title: "Components/Common/Games/In-Game/Attributes",
  component: Attributes,
  args: defaultAttributesProps,
} satisfies Meta<typeof Attributes>;

export const Default = {};
