import { jsx as _jsx } from "react/jsx-runtime";
import styled from "@emotion/styled";
import * as model from "@deep-trinity/model";
import { useTheme } from "./theme";
const CellElement = styled.div `
  width: 100%;
  height: 100%;
  background-color: ${(props) => props.color};
  box-sizing: border-box;
  border: ${(props) => props.borderStyle};
`;
export function Cell(props) {
    const theme = useTheme();
    return (_jsx(CellElement, { color: theme.cellColors[props.cell], borderStyle: props.borderStyle
            || props.cell != model.Cell.Empty && theme.nonEmptyCellBorderStyle
            || theme.cellBorderStyle }));
}
export default Cell;
//# sourceMappingURL=Cell.js.map