import { jsx as _jsx } from "react/jsx-runtime";
import styled from "@emotion/styled";
import PieceContainer from "./PieceContainer";
const NextPiecesRoot = styled.div ``;
export function NextPieces(props) {
    const elPieces = [];
    for (let i = 0; i < props.pieces.length; i++) {
        elPieces.push(_jsx(PieceContainer, { piece: props.pieces[i] }, i));
    }
    if (elPieces.length == 0) {
        elPieces.push(_jsx(PieceContainer, {}, 0)); // dummy
    }
    return (_jsx(NextPiecesRoot, { children: elPieces }));
}
export default NextPieces;
//# sourceMappingURL=NextPieces.js.map