import React from "react";
import styled from "@emotion/styled";
import * as core from "@deep-trinity/core-wasm";
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
  game: core.Game,
};

export const Game: React.FC<GameProps> = props => {
  return (
    <GameRoot>
      <GameRootInner>
        <PieceContainer piece={props.game.getHoldPiece()}/>
        <Playfield game={props.game}/>
        <NextPieces pieces={[...props.game.getNextPieces()]}/>
      </GameRootInner>
    </GameRoot>
  )
};

export default Game;
