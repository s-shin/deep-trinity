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
exports.Cell = void 0;
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const model = __importStar(require("@deep-trinity/model"));
const theme_1 = require("./theme");
const CellElement = styled_1.default.div `
  width: 100%;
  height: 100%;
  background-color: ${(props) => props.color};
  box-sizing: border-box;
  border: ${(props) => props.borderStyle};
`;
const Cell = props => {
    const theme = (0, theme_1.useTheme)();
    return (react_1.default.createElement(CellElement, { color: theme.cellColors[props.cell], borderStyle: props.borderStyle
            || props.cell != model.Cell.Empty && theme.nonEmptyCellBorderStyle
            || theme.cellBorderStyle }));
};
exports.Cell = Cell;
exports.default = exports.Cell;
//# sourceMappingURL=Cell.js.map