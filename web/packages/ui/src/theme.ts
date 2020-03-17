import * as core from "@deep-trinity/core-wasm";

export type Theme = {
  gridGap: string,
  cellSize: string,
  cellBorderStyle: string,
  nonEmptyCellBorderStyle?: string,
  pieceCellBorderStyle?: string,
  cellColors: { [cell: number]: string },
};

export const DEFAULT_THEME: Theme = {
  gridGap: "2px",
  cellSize: "4vh",
  cellBorderStyle: "1px solid #CCC",
  nonEmptyCellBorderStyle: "none",
  pieceCellBorderStyle: "none",
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

export type StyledProps = {
  theme: Theme,
};
