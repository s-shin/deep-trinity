import React from "react";
import * as model from "@deep-trinity/model";
export declare type CellProps = {
    cell: model.Cell;
    borderStyle?: string;
};
export declare const Cell: React.FC<CellProps>;
export default Cell;
