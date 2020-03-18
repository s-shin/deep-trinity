import React from "react";
import styled from "@emotion/styled";
import * as core from "@deep-trinity/core-wasm";
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
`;

export type PieceContainerProps = {
  piece?: core.Piece,
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
