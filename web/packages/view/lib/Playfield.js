"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const Grid_1 = __importDefault(require("./Grid"));
exports.Playfield = props => {
    return (react_1.default.createElement(Grid_1.default, { width: props.game.width(), height: props.game.visibleHeight(), cellGetter: (x, y) => props.game.getCell(x, y) }));
};
exports.default = exports.Playfield;
//# sourceMappingURL=Playfield.js.map