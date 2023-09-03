import { jsx as _jsx } from "react/jsx-runtime";
import styled from "@emotion/styled";
import { useTheme } from "./theme";
import Piece from "./Piece";
const PieceContainerRoot = styled.div `
  margin: ${(props) => props.margin};
  width: ${(props) => props.size.width};
  height: ${(props) => props.size.height};
  display: grid;
  place-items: center;
`;
export function PieceContainer(props) {
    const theme = useTheme();
    return (_jsx(PieceContainerRoot, { margin: theme.pieceContainerMargin, size: theme.pieceContainerSize, children: props.piece != undefined && _jsx(Piece, { piece: props.piece }) }));
}
export default PieceContainer;
//# sourceMappingURL=PieceContainer.js.map