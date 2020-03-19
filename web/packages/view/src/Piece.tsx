import React from "react";
import * as core from "@deep-trinity/web-core";
import { useTheme } from "./theme";
import Grid, { GridProps } from "./Grid";

const pieceSpecs: {
  [piece: number]: GridProps,
} = {
  [core.Piece.S]: {
    width: 3,
    height: 2,
    cellGetter: (x, y) =>
      ((x == 0 && y == 0) || (x == 1 && y == 0) || (x == 1 && y == 1) || (x == 2 && y == 1))
        ? core.Cell.S : core.Cell.EMPTY,
  },
  [core.Piece.Z]: {
    width: 3,
    height: 2,
    cellGetter: (x, y) =>
      ((x == 0 && y == 1) || (x == 1 && y == 0) || (x == 1 && y == 1) || (x == 2 && y == 0))
        ? core.Cell.Z : core.Cell.EMPTY,
  },
  [core.Piece.L]: {
    width: 3,
    height: 2,
    cellGetter: (x, y) =>
      ((x == 0 && y == 0) || (x == 1 && y == 0) || (x == 2 && y == 0) || (x == 2 && y == 1))
        ? core.Cell.L : core.Cell.EMPTY,
  },
  [core.Piece.J]: {
    width: 3,
    height: 2,
    cellGetter: (x, y) =>
      ((x == 0 && y == 0) || (x == 1 && y == 0) || (x == 2 && y == 0) || (x == 0 && y == 1))
        ? core.Cell.J : core.Cell.EMPTY,
  },
  [core.Piece.I]: {
    width: 4,
    height: 1,
    cellGetter: () => core.Cell.I,
  },
  [core.Piece.T]: {
    width: 3,
    height: 2,
    cellGetter: (x, y) =>
      ((x == 0 && y == 0) || (x == 1 && y == 0) || (x == 2 && y == 0) || (x == 1 && y == 1))
        ? core.Cell.T : core.Cell.EMPTY,
  },
  [core.Piece.O]: {
    width: 2,
    height: 2,
    cellGetter: () => core.Cell.O,
  },
};

export type PieceProps = {
  piece: core.Piece,
};

export const Piece: React.FC<PieceProps> = props => {
  const theme = useTheme();
  return (
    <Grid {...pieceSpecs[props.piece]} borderStyle={theme.pieceCellBorderStyle}/>
  );
};

export default Piece;
