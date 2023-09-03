import styled from "@emotion/styled";
import * as model from "@deep-trinity/model";
import PieceContainer from "./PieceContainer";

const NextPiecesRoot = styled.div``;

export type NextPiecesProps = {
  pieces: ArrayLike<model.Piece>,
};

export function NextPieces(props: NextPiecesProps) {
  const elPieces = [];
  for (let i = 0; i < props.pieces.length; i++) {
    elPieces.push(
      <PieceContainer key={i} piece={props.pieces[i]}/>,
    );
  }
  if (elPieces.length == 0) {
    elPieces.push(<PieceContainer key={0}/>); // dummy
  }
  return (
    <NextPiecesRoot>
      {elPieces}
    </NextPiecesRoot>
  );
}

export default NextPieces;
