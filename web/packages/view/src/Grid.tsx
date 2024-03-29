import styled from "@emotion/styled";
import * as model from "@deep-trinity/model";
import { useTheme } from "./theme";
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
  cells: ArrayLike<model.Cell>,
  borderStyle?: string,
};

export function Grid(props: GridProps) {
  const theme = useTheme();
  const cells = [];
  for (let y = props.height - 1; y >= 0; y--) {
    for (let x = 0; x < props.width; x++) {
      cells.push(
        <Cell
          key={`${x}-${y}`}
          cell={props.cells[model.getIndex(props.width, x, y)] || model.Cell.Empty}
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
}

export default Grid;
