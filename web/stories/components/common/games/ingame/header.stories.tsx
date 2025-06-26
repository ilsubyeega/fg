import { defaultAttributesProps, defaultTitleProps, Header } from "~/components/common/games/ingame";
import type { Meta } from "@kachurun/storybook-solid-vite";

export default {
  title: "Components/Common/Games/In-Game/Header",
  component: Header,
  args: {
    titleProps: defaultTitleProps,
    attributesProps: defaultAttributesProps
  }
} satisfies Meta<typeof Header>;

export const Default = {};
