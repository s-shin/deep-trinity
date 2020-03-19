import * as core from "@deep-trinity/web-core";
import * as theming from "emotion-theming";

export type Theme = {
  gridGap: string,
  cellSize: string,
  cellBorderStyle: string,
  nonEmptyCellBorderStyle?: string,
  pieceCellBorderStyle?: string,
  cellColors: { [cell: number]: string },
  pieceContainerMargin: string,
  pieceContainerSize: { width: string, height: string },
};

export const DEFAULT_THEME: Theme = {
  gridGap: "2px",
  cellSize: "4vmin",
  cellBorderStyle: "1px solid #CCC",
  nonEmptyCellBorderStyle: "none",
  pieceCellBorderStyle: "none",
  cellColors: {
    [core.Cell.EMPTY]: "rgba(0, 0, 0, 0)",
    [core.Cell.ANY]: "darkslategray",
    [core.Cell.S]: "darkolivegreen",
    [core.Cell.Z]: "sienna",
    [core.Cell.L]: "darkorange",
    [core.Cell.J]: "darkblue",
    [core.Cell.I]: "turquoise",
    [core.Cell.T]: "purple",
    [core.Cell.O]: "gold",
    [core.Cell.GARBAGE]: "gray"
  },
  pieceContainerMargin: "2.5vmin 2vmin",
  pieceContainerSize: { width: "17vmin", height: "9vmin" },
};

export type StyledProps = {
  theme: Theme,
};

export const useTheme: () => Theme = () => theming.useTheme<Theme>();
