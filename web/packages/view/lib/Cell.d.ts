import React from "react";
import * as core from "@deep-trinity/core-wasm";
export declare type CellProps = {
    cell: core.Cell;
    borderStyle?: string;
};
export declare const Cell: React.FC<CellProps>;
export default Cell;