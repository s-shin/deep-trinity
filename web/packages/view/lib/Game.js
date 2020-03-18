"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const NextPieces_1 = __importDefault(require("./NextPieces"));
const Playfield_1 = __importDefault(require("./Playfield"));
const PieceContainer_1 = __importDefault(require("./PieceContainer"));
const GameRoot = styled_1.default.div `
  display: grid;
  align-items: center;
  place-items: center;
  height: 100%;
`;
const GameRootInner = styled_1.default.div `
  display: grid;
  grid-template-columns: min-content min-content min-content;
`;
exports.Game = props => {
    return (react_1.default.createElement(GameRoot, null,
        react_1.default.createElement(GameRootInner, null,
            react_1.default.createElement(PieceContainer_1.default, { piece: props.game.getHoldPiece() }),
            react_1.default.createElement(Playfield_1.default, { game: props.game }),
            react_1.default.createElement(NextPieces_1.default, { pieces: [...props.game.getNextPieces()] }))));
};
exports.default = exports.Game;
//# sourceMappingURL=Game.js.map