"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const Piece_1 = __importDefault(require("./Piece"));
const NextPiecesRoot = styled_1.default.div ``;
exports.NextPieces = props => {
    const elPieces = [];
    for (let i = 0; i < props.pieces.length; i++) {
        elPieces.push(react_1.default.createElement(Piece_1.default, { key: i, piece: props.pieces[i] }));
    }
    return (react_1.default.createElement(NextPiecesRoot, null, elPieces));
};
exports.default = exports.NextPieces;
//# sourceMappingURL=NextPieces.js.map