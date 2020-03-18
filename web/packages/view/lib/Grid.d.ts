import React from "react";
import * as core from "@deep-trinity/core-wasm";
export declare type GridProps = {
    width: number;
    height: number;
    cellGetter: (x: number, y: number) => core.Cell;
    borderStyle?: string;
};
export declare const Grid: React.FC<GridProps>;
export default Grid;
