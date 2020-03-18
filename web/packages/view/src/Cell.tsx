import React from "react";
import styled from "@emotion/styled";
import * as core from "@deep-trinity/core-wasm";
import { useTheme } from "./theme";

type CellElementProps = {
  color: string,
  borderStyle: string,
};

const CellElement = styled.div`
  width: 100%;
  height: 100%;
  background-color: ${(props: CellElementProps) => props.color};
  box-sizing: border-box;
  border: ${(props: CellElementProps) => props.borderStyle};
`;

export type CellProps = {
  cell: core.Cell,
  borderStyle?: string,
};

export const Cell: React.FC<CellProps> = props => {
  const theme = useTheme();
  return (
    <CellElement
      color={theme.cellColors[props.cell]}
      borderStyle={
        props.borderStyle
        || props.cell != core.Cell.EMPTY && theme.nonEmptyCellBorderStyle
        || theme.cellBorderStyle
      }
    />
  );
};

export default Cell;
