import * as model from "@deep-trinity/model";
import Grid from "./Grid";

export type PlayfieldProps = {
  game: model.Game,
};

export function Playfield(props: PlayfieldProps) {
  return (
    <Grid
      width={props.game.width}
      height={props.game.visibleHeight}
      cells={props.game.cells}
    />
  );
}

export default Playfield;
