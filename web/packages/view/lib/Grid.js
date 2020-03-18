"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const theme_1 = require("./theme");
const Cell_1 = __importDefault(require("./Cell"));
const GridRoot = styled_1.default.div `
  display: grid;
  grid-template-columns: ${(props) => `repeat(${props.width}, ${props.cellSize})`};
  grid-template-rows: ${(props) => `repeat(${props.height}, ${props.cellSize})`};
  grid-gap: ${(props) => props.gap};
`;
exports.Grid = props => {
    const theme = theme_1.useTheme();
    const cells = [];
    for (let y = props.height - 1; y >= 0; y--) {
        for (let x = 0; x < props.width; x++) {
            cells.push(react_1.default.createElement(Cell_1.default, { key: `${x}-${y}`, cell: props.cellGetter(x, y), borderStyle: props.borderStyle }));
        }
    }
    return (react_1.default.createElement(GridRoot, { width: props.width, height: props.height, cellSize: theme.cellSize, gap: theme.gridGap }, cells));
};
exports.default = exports.Grid;
//# sourceMappingURL=Grid.js.map