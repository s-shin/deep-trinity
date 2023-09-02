import { jsx as _jsx } from "react/jsx-runtime";
import styled from "@emotion/styled";
import * as model from "@deep-trinity/model";
import { useTheme } from "./theme";
import Cell from "./Cell";
const GridRoot = styled.div `
  display: grid;
  grid-template-columns: ${(props) => `repeat(${props.width}, ${props.cellSize})`};
  grid-template-rows: ${(props) => `repeat(${props.height}, ${props.cellSize})`};
  grid-gap: ${(props) => props.gap};
`;
export function Grid(props) {
    const theme = useTheme();
    const cells = [];
    for (let y = props.height - 1; y >= 0; y--) {
        for (let x = 0; x < props.width; x++) {
            cells.push(_jsx(Cell, { cell: props.cells[model.getIndex(props.width, x, y)] || model.Cell.Empty, borderStyle: props.borderStyle }, `${x}-${y}`));
        }
    }
    return (_jsx(GridRoot, { width: props.width, height: props.height, cellSize: theme.cellSize, gap: theme.gridGap, children: cells }));
}
export default Grid;
//# sourceMappingURL=Grid.js.map