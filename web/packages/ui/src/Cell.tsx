import React from "react";
import styled from "@emotion/styled";
import { useTheme } from "emotion-theming";
import * as core from "@deep-trinity/core-wasm";
import { Theme } from "./theme";

const CellElem = styled.div`
  background-color: ${(props: { color: string }) => props.color};
`;

export type CellProps = {
  cell: core.Cell,
};

export const Cell: React.FC<CellProps> = props => {
  const theme = useTheme<Theme>();
  return (
    <CellElem color={theme.cellColors[props.cell]}>{props.cell}</CellElem>
  );
};

export default Cell;
