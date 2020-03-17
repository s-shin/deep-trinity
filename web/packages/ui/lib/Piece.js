"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const emotion_theming_1 = require("emotion-theming");
const core = __importStar(require("@deep-trinity/core-wasm"));
const Grid_1 = __importDefault(require("./Grid"));
const pieceSpecs = {
    [core.Piece.S]: {
        width: 3,
        height: 2,
        cellGetter: (x, y) => ((x == 0 && y == 0) || (x == 1 && y == 0) || (x == 1 && y == 1) || (x == 2 && y == 1))
            ? core.Cell.S : core.Cell.EMPTY,
    },
    [core.Piece.Z]: {
        width: 3,
        height: 2,
        cellGetter: (x, y) => ((x == 0 && y == 1) || (x == 1 && y == 0) || (x == 1 && y == 1) || (x == 2 && y == 0))
            ? core.Cell.Z : core.Cell.EMPTY,
    },
    [core.Piece.L]: {
        width: 3,
        height: 2,
        cellGetter: (x, y) => ((x == 0 && y == 0) || (x == 1 && y == 0) || (x == 2 && y == 0) || (x == 2 && y == 1))
            ? core.Cell.L : core.Cell.EMPTY,
    },
    [core.Piece.J]: {
        width: 3,
        height: 2,
        cellGetter: (x, y) => ((x == 0 && y == 0) || (x == 1 && y == 0) || (x == 2 && y == 0) || (x == 0 && y == 1))
            ? core.Cell.J : core.Cell.EMPTY,
    },
    [core.Piece.I]: {
        width: 4,
        height: 1,
        cellGetter: () => core.Cell.I,
    },
    [core.Piece.T]: {
        width: 3,
        height: 2,
        cellGetter: (x, y) => ((x == 0 && y == 0) || (x == 1 && y == 0) || (x == 2 && y == 0) || (x == 1 && y == 1))
            ? core.Cell.T : core.Cell.EMPTY,
    },
    [core.Piece.O]: {
        width: 2,
        height: 2,
        cellGetter: () => core.Cell.O,
    },
};
exports.Piece = props => {
    const theme = emotion_theming_1.useTheme();
    return (react_1.default.createElement(Grid_1.default, Object.assign({}, pieceSpecs[props.piece], { borderStyle: theme.pieceCellBorderStyle })));
};
exports.default = exports.Piece;
//# sourceMappingURL=Piece.js.map