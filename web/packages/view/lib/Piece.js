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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Piece = void 0;
const react_1 = __importDefault(require("react"));
const model = __importStar(require("@deep-trinity/model"));
const theme_1 = require("./theme");
const Grid_1 = __importDefault(require("./Grid"));
const define = (piece, width, height, nonEmptyPositions) => {
    const cells = Array(width * height).fill(model.Cell.Empty);
    for (const pos of nonEmptyPositions) {
        cells[model.getIndex(width, pos[0], pos[1])] = model.pieceToCell(piece);
    }
    return { width, height, cells };
};
const pieceSpecs = {
    [model.Piece.S]: define(model.Piece.S, 3, 2, [[0, 0], [1, 0], [1, 1], [2, 1]]),
    [model.Piece.Z]: define(model.Piece.Z, 3, 2, [[0, 1], [1, 0], [1, 1], [2, 0]]),
    [model.Piece.L]: define(model.Piece.L, 3, 2, [[0, 0], [1, 0], [2, 0], [2, 1]]),
    [model.Piece.J]: define(model.Piece.J, 3, 2, [[0, 0], [1, 0], [2, 0], [0, 1]]),
    [model.Piece.I]: define(model.Piece.I, 4, 1, [[0, 0], [1, 0], [2, 0], [3, 0]]),
    [model.Piece.T]: define(model.Piece.T, 3, 2, [[0, 0], [1, 0], [2, 0], [1, 1]]),
    [model.Piece.O]: define(model.Piece.O, 2, 2, [[0, 0], [1, 0], [0, 1], [1, 1]]),
};
const Piece = props => {
    const theme = (0, theme_1.useTheme)();
    return (react_1.default.createElement(Grid_1.default, Object.assign({}, pieceSpecs[props.piece], { borderStyle: theme.pieceCellBorderStyle })));
};
exports.Piece = Piece;
exports.default = exports.Piece;
//# sourceMappingURL=Piece.js.map