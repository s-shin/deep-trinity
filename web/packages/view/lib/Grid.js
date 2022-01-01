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
exports.Grid = void 0;
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const model = __importStar(require("@deep-trinity/model"));
const theme_1 = require("./theme");
const Cell_1 = __importDefault(require("./Cell"));
const GridRoot = styled_1.default.div `
  display: grid;
  grid-template-columns: ${(props) => `repeat(${props.width}, ${props.cellSize})`};
  grid-template-rows: ${(props) => `repeat(${props.height}, ${props.cellSize})`};
  grid-gap: ${(props) => props.gap};
`;
const Grid = props => {
    const theme = (0, theme_1.useTheme)();
    const cells = [];
    for (let y = props.height - 1; y >= 0; y--) {
        for (let x = 0; x < props.width; x++) {
            cells.push(react_1.default.createElement(Cell_1.default, { key: `${x}-${y}`, cell: props.cells[model.getIndex(props.width, x, y)] || model.Cell.Empty, borderStyle: props.borderStyle }));
        }
    }
    return (react_1.default.createElement(GridRoot, { width: props.width, height: props.height, cellSize: theme.cellSize, gap: theme.gridGap }, cells));
};
exports.Grid = Grid;
exports.default = exports.Grid;
//# sourceMappingURL=Grid.js.map