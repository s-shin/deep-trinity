import * as core from "@deep-trinity/core-wasm";

export const DEFAULT_THEME = {
  cellColors: {
    [core.Cell.EMPTY]: "rgba(0, 0, 0, 0)",
    [core.Cell.ANY]: "darkslategray",
    [core.Cell.S]: "green",
    [core.Cell.Z]: "red",
    [core.Cell.L]: "orange",
    [core.Cell.J]: "blue",
    [core.Cell.I]: "cyan",
    [core.Cell.T]: "purple",
    [core.Cell.O]: "yellow",
    [core.Cell.GARBAGE]: "gray"
  },
};

export type Theme = typeof DEFAULT_THEME;

export type StyledProps = {
  theme: Theme,
};
