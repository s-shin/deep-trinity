import * as model from "@deep-trinity/model";
import * as theming from '@emotion/react';
export const DEFAULT_THEME = {
    gridGap: "2px",
    cellSize: "4vmin",
    cellBorderStyle: "1px solid #CCC",
    nonEmptyCellBorderStyle: "none",
    pieceCellBorderStyle: "none",
    cellColors: {
        [model.Cell.Empty]: "rgba(0, 0, 0, 0)",
        [model.Cell.Any]: "darkslategray",
        [model.Cell.S]: "darkolivegreen",
        [model.Cell.Z]: "sienna",
        [model.Cell.L]: "darkorange",
        [model.Cell.J]: "darkblue",
        [model.Cell.I]: "turquoise",
        [model.Cell.T]: "purple",
        [model.Cell.O]: "gold",
        [model.Cell.Garbage]: "gray"
    },
    pieceContainerMargin: "2.5vmin 2vmin",
    pieceContainerSize: { width: "17vmin", height: "9vmin" },
};
export const useTheme = () => theming.useTheme();
//# sourceMappingURL=theme.js.map