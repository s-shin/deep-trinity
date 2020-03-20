import React from "react";
import * as model from "@deep-trinity/model";
export declare type GridProps = {
    width: number;
    height: number;
    cells: ArrayLike<model.Cell>;
    borderStyle?: string;
};
export declare const Grid: React.FC<GridProps>;
export default Grid;
