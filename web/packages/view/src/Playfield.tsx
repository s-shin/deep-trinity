import React from "react";
import * as model from "@deep-trinity/model";
import Grid from "./Grid";

export type PlayfieldProps = {
  game: model.Game,
};

export const Playfield: React.FC<PlayfieldProps> = props => {
  return (
    <Grid
      width={props.game.width}
      height={props.game.visibleHeight}
      cells={props.game.cells}
    />
  );
};

export default Playfield;
