"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const Cell_1 = __importDefault(require("./Cell"));
const PlayfieldContainer = styled_1.default.div `
  display: grid;
  grid-template-columns: fit-content(100%);
  grid-template-rows: fit-content(100%);
`;
exports.Playfield = props => {
    const W = props.game.width();
    const H = props.game.visible_height();
    const cells = [];
    for (let y = 0; y < H; y++) {
        for (let x = 0; x < W; x++) {
            cells.push(react_1.default.createElement(Cell_1.default, { cell: props.game.getCell(x, y), key: `${x}-${y}` }));
        }
    }
    return (react_1.default.createElement(PlayfieldContainer, null, cells));
};
exports.default = exports.Playfield;
//# sourceMappingURL=Playfield.js.map