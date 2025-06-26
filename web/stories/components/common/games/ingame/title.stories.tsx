import { defaultTitleProps, Title } from "~/components/common/games/ingame";
import type { Meta } from "@kachurun/storybook-solid-vite";

export default {
  title: "Components/Common/Games/In-Game/Title",
  component: Title,
  args: defaultTitleProps,
} satisfies Meta<typeof Title>;

export const Default = {};
