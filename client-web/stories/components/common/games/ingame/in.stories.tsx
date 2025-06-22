import { defaultAttributesProps, defaultTitleProps, Header, Title, Attributes } from "~/components/common/games/ingame";
import type { Meta } from "@kachurun/storybook-solid-vite";

export default {
  title: "Components/Common/Games/In-Game/in",
  component: Header,
  subcomponents: { Title, Attributes },
  args: {
    titleProps: defaultTitleProps,
    attributesProps: defaultAttributesProps
  }
} satisfies Meta<typeof Header>;

export const Default = {};
