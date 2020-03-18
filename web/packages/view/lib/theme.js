"use strict";
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const core = __importStar(require("@deep-trinity/core-wasm"));
const theming = __importStar(require("emotion-theming"));
exports.DEFAULT_THEME = {
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
exports.useTheme = () => theming.useTheme();
//# sourceMappingURL=theme.js.map