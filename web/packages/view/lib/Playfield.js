import { jsx as _jsx } from "react/jsx-runtime";
import Grid from "./Grid";
export function Playfield(props) {
    return (_jsx(Grid, { width: props.game.width, height: props.game.visibleHeight, cells: props.game.cells }));
}
export default Playfield;
//# sourceMappingURL=Playfield.js.map