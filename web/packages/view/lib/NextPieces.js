"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.NextPieces = void 0;
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const PieceContainer_1 = __importDefault(require("./PieceContainer"));
const NextPiecesRoot = styled_1.default.div ``;
const NextPieces = props => {
    const elPieces = [];
    for (let i = 0; i < props.pieces.length; i++) {
        elPieces.push(react_1.default.createElement(PieceContainer_1.default, { key: i, piece: props.pieces[i] }));
    }
    if (elPieces.length == 0) {
        elPieces.push(react_1.default.createElement(PieceContainer_1.default, { key: 0 })); // dummy
    }
    return (react_1.default.createElement(NextPiecesRoot, null, elPieces));
};
exports.NextPieces = NextPieces;
exports.default = exports.NextPieces;
//# sourceMappingURL=NextPieces.js.map