import { jsx as _jsx } from "react/jsx-runtime";
import * as model from "@deep-trinity/model";
import { useTheme } from "./theme";
import Grid from "./Grid";
const define = (piece, width, height, nonEmptyPositions) => {
    const cells = Array(width * height).fill(model.Cell.Empty);
    for (const pos of nonEmptyPositions) {
        cells[model.getIndex(width, pos[0], pos[1])] = model.pieceToCell(piece);
    }
    return { width, height, cells };
};
const pieceSpecs = {
    [model.Piece.S]: define(model.Piece.S, 3, 2, [[0, 0], [1, 0], [1, 1], [2, 1]]),
    [model.Piece.Z]: define(model.Piece.Z, 3, 2, [[0, 1], [1, 0], [1, 1], [2, 0]]),
    [model.Piece.L]: define(model.Piece.L, 3, 2, [[0, 0], [1, 0], [2, 0], [2, 1]]),
    [model.Piece.J]: define(model.Piece.J, 3, 2, [[0, 0], [1, 0], [2, 0], [0, 1]]),
    [model.Piece.I]: define(model.Piece.I, 4, 1, [[0, 0], [1, 0], [2, 0], [3, 0]]),
    [model.Piece.T]: define(model.Piece.T, 3, 2, [[0, 0], [1, 0], [2, 0], [1, 1]]),
    [model.Piece.O]: define(model.Piece.O, 2, 2, [[0, 0], [1, 0], [0, 1], [1, 1]]),
};
export function Piece(props) {
    const theme = useTheme();
    return (_jsx(Grid, Object.assign({}, pieceSpecs[props.piece], { borderStyle: theme.pieceCellBorderStyle })));
}
export default Piece;
//# sourceMappingURL=Piece.js.map