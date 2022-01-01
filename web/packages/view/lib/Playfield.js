"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Playfield = void 0;
const react_1 = __importDefault(require("react"));
const Grid_1 = __importDefault(require("./Grid"));
const Playfield = props => {
    return (react_1.default.createElement(Grid_1.default, { width: props.game.width, height: props.game.visibleHeight, cells: props.game.cells }));
};
exports.Playfield = Playfield;
exports.default = exports.Playfield;
//# sourceMappingURL=Playfield.js.map