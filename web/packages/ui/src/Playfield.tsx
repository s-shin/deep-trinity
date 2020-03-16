import React from "react";
import styled from "@emotion/styled";
import * as core from "@deep-trinity/core-wasm";
import Cell from "./Cell";

const PlayfieldContainer = styled.div`
  display: grid;
  grid-template-columns: fit-content(100%);
  grid-template-rows: fit-content(100%);
`;

export type PlayfieldProps = {
  game: core.Game,
};

export const Playfield: React.FC<PlayfieldProps> = props => {
  const W = props.game.width();
  const H = props.game.visible_height();
  const cells = [];
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      cells.push(<Cell cell={props.game.getCell(x, y)} key={`${x}-${y}`}/>);
    }
  }
  return (
    <PlayfieldContainer>
      {cells}
    </PlayfieldContainer>
  );
};

export default Playfield;
