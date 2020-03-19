import React from "react";
import styled from "@emotion/styled";
import * as core from "@deep-trinity/web-core";
import PieceContainer from "./PieceContainer";

const NextPiecesRoot = styled.div``;

export type NextPiecesProps = {
  pieces: core.Piece[],
};

export const NextPieces: React.FC<NextPiecesProps> = props => {
  const elPieces = [];
  for (let i = 0; i < props.pieces.length; i++) {
    elPieces.push(
      <PieceContainer key={i} piece={props.pieces[i]}/>
    );
  }
  return (
    <NextPiecesRoot>
      {elPieces}
    </NextPiecesRoot>
  );
};

export default NextPieces;
