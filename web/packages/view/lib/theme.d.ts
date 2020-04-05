export declare type Theme = {
    gridGap: string;
    cellSize: string;
    cellBorderStyle: string;
    nonEmptyCellBorderStyle?: string;
    pieceCellBorderStyle?: string;
    cellColors: {
        [cell: number]: string;
    };
    pieceContainerMargin: string;
    pieceContainerSize: {
        width: string;
        height: string;
    };
};
export declare const DEFAULT_THEME: Theme;
export declare type StyledProps = {
    theme: Theme;
};
export declare const useTheme: () => Theme;
