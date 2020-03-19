import React from "react";
import * as core from "@deep-trinity/web-core";
import Grid from "./Grid";

export type PlayfieldProps = {
  game: core.Game,
};

export const Playfield: React.FC<PlayfieldProps> = props => {
  return (
    <Grid
      width={props.game.width()}
      height={props.game.visibleHeight()}
      cellGetter={(x, y) => props.game.getCell(x, y)}
    />
  );
};

export default Playfield;
