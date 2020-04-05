"use strict";
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const model = __importStar(require("@deep-trinity/model"));
const theming = __importStar(require("emotion-theming"));
exports.DEFAULT_THEME = {
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
exports.useTheme = () => theming.useTheme();
//# sourceMappingURL=theme.js.map