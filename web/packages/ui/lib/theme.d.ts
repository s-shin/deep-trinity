export declare type Theme = {
    gridGap: string;
    cellSize: string;
    cellBorderStyle: string;
    nonEmptyCellBorderStyle?: string;
    pieceCellBorderStyle?: string;
    cellColors: {
        [cell: number]: string;
    };
};
export declare const DEFAULT_THEME: Theme;
export declare type StyledProps = {
    theme: Theme;
};
