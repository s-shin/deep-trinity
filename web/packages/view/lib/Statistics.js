"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Statistics = void 0;
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const Key = styled_1.default.div `
  padding: 0 0.5em;
`;
const Value = styled_1.default.div `
  padding: 0 0.5em;
  width: 5em;
`;
const STRS = [
    "Single",
    "Double",
    "Triple",
    "Tetris",
    "TST",
    "TSD",
    "TSS",
    "TSMD",
    "TSMS",
    "Max Combos",
    "Max BTBs",
    "Perfect Clear",
    "Hold",
    "Lock",
];
const Statistics = props => {
    const rows = props.types.map(t => (react_1.default.createElement("tr", { key: t },
        react_1.default.createElement("td", null,
            react_1.default.createElement(Key, null, STRS[t])),
        react_1.default.createElement("td", null,
            react_1.default.createElement(Value, null, props.game.stats[t])))));
    return (react_1.default.createElement("div", null,
        react_1.default.createElement("table", null,
            react_1.default.createElement("tbody", null, rows))));
};
exports.Statistics = Statistics;
exports.default = exports.Statistics;
//# sourceMappingURL=Statistics.js.map