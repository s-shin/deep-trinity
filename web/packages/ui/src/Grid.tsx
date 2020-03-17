import React from "react";
import styled from "@emotion/styled";
import { useTheme } from "emotion-theming";
import * as core from "@deep-trinity/core-wasm";
import { Theme } from "./theme";
import Cell from "./Cell";

type GridRootProps = {
  width: number,
  height: number,
  cellSize: string,
  gap: string,
};

const GridRoot = styled.div`
  display: grid;
  grid-template-columns: ${(props: GridRootProps) => `repeat(${props.width}, ${props.cellSize})`};
  grid-template-rows: ${(props: GridRootProps) => `repeat(${props.height}, ${props.cellSize})`};
  grid-gap: ${(props: GridRootProps) => props.gap};
`;

export type GridProps = {
  width: number,
  height: number,
  cellGetter: (x: number, y: number) => core.Cell,
  borderStyle?: string,
};

export const Grid: React.FC<GridProps> = props => {
  const theme = useTheme<Theme>();
  const cells = [];
  for (let y = props.height - 1; y >= 0; y--) {
    for (let x = 0; x < props.width; x++) {
      cells.push(
        <Cell
          key={`${x}-${y}`}
          cell={props.cellGetter(x, y)}
          borderStyle={props.borderStyle}
        />,
      );
    }
  }
  return (
    <GridRoot
      width={props.width}
      height={props.height}
      cellSize={theme.cellSize}
      gap={theme.gridGap}
    >
      {cells}
    </GridRoot>
  );
};

export default Grid;
