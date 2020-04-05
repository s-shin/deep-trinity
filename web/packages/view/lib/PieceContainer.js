"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const theme_1 = require("./theme");
const Piece_1 = __importDefault(require("./Piece"));
const PieceContainerRoot = styled_1.default.div `
  margin: ${(props) => props.margin};
  width: ${(props) => props.size.width};
  height: ${(props) => props.size.height};
  display: grid;
  place-items: center;
`;
exports.PieceContainer = props => {
    const theme = theme_1.useTheme();
    return (react_1.default.createElement(PieceContainerRoot, { margin: theme.pieceContainerMargin, size: theme.pieceContainerSize }, props.piece != undefined && react_1.default.createElement(Piece_1.default, { piece: props.piece })));
};
exports.default = exports.PieceContainer;
//# sourceMappingURL=PieceContainer.js.map