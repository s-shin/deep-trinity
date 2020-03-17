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
exports.DEFAULT_THEME = {
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
//# sourceMappingURL=theme.js.map