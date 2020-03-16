"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const emotion_theming_1 = require("emotion-theming");
const CellElem = styled_1.default.div `
  background-color: ${(props) => props.color};
`;
exports.Cell = props => {
    const theme = emotion_theming_1.useTheme();
    return (react_1.default.createElement(CellElem, { color: theme.cellColors[props.cell] }, props.cell));
};
exports.default = exports.Cell;
//# sourceMappingURL=Cell.js.map