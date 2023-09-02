import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import styled from "@emotion/styled";
import NextPieces from "./NextPieces";
import Playfield from "./Playfield";
import PieceContainer from "./PieceContainer";
const GameRoot = styled.div `
  display: grid;
  align-items: center;
  place-items: center;
  height: 100%;
`;
const GameRootInner = styled.div `
  display: grid;
  grid-template-columns: min-content min-content min-content;
`;
export function Game(props) {
    return (_jsx(GameRoot, { children: _jsxs(GameRootInner, { children: [_jsx(PieceContainer, { piece: props.game.holdPiece }), _jsx(Playfield, { game: props.game }), _jsx(NextPieces, { pieces: props.game.nextPieces })] }) }));
}
export default Game;
//# sourceMappingURL=Game.js.map