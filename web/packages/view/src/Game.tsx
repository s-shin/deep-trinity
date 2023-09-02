import styled from "@emotion/styled";
import * as model from "@deep-trinity/model";
import NextPieces from "./NextPieces";
import Playfield from "./Playfield";
import PieceContainer from "./PieceContainer";

const GameRoot = styled.div`
  display: grid;
  align-items: center;
  place-items: center;
  height: 100%;
`;

const GameRootInner = styled.div`
  display: grid;
  grid-template-columns: min-content min-content min-content;
`;

export type GameProps = {
  game: model.Game,
};

export function Game(props: GameProps) {
  return (
    <GameRoot>
      <GameRootInner>
        <PieceContainer piece={props.game.holdPiece}/>
        <Playfield game={props.game}/>
        <NextPieces pieces={props.game.nextPieces}/>
      </GameRootInner>
    </GameRoot>
  )
}

export default Game;
