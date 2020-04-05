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
exports.Cell = props => {
    const theme = theme_1.useTheme();
    return (react_1.default.createElement(CellElement, { color: theme.cellColors[props.cell], borderStyle: props.borderStyle
            || props.cell != model.Cell.Empty && theme.nonEmptyCellBorderStyle
            || theme.cellBorderStyle }));
};
exports.default = exports.Cell;
//# sourceMappingURL=Cell.js.map