import React from "react";
import styled from "@emotion/styled";
import * as model from "@deep-trinity/model";
import { useTheme } from "./theme";
import Piece from "./Piece";

type PieceContainerRootProps = {
  margin: string,
  size: { width: string, height: string },
};

const PieceContainerRoot = styled.div`
  margin: ${(props: PieceContainerRootProps) => props.margin};
  width: ${(props: PieceContainerRootProps) => props.size.width};
  height: ${(props: PieceContainerRootProps) => props.size.height};
  display: grid;
  place-items: center;
`;

export type PieceContainerProps = {
  piece?: model.Piece,
};

export const PieceContainer: React.FC<PieceContainerProps> = props => {
  const theme = useTheme();
  return (
    <PieceContainerRoot margin={theme.pieceContainerMargin} size={theme.pieceContainerSize}>
      {props.piece != undefined && <Piece piece={props.piece}/>}
    </PieceContainerRoot>
  );
};

export default PieceContainer;
