import * as model from "@deep-trinity/model";
export type GridProps = {
    width: number;
    height: number;
    cells: ArrayLike<model.Cell>;
    borderStyle?: string;
};
export declare function Grid(props: GridProps): import("react/jsx-runtime").JSX.Element;
export default Grid;
