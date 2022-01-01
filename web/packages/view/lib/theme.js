"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.useTheme = exports.DEFAULT_THEME = void 0;
const model = __importStar(require("@deep-trinity/model"));
const theming = __importStar(require("@emotion/react"));
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
const useTheme = () => theming.useTheme();
exports.useTheme = useTheme;
//# sourceMappingURL=theme.js.map