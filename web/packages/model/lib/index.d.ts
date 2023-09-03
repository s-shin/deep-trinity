export declare enum Piece {
    S = 0,
    Z = 1,
    L = 2,
    J = 3,
    I = 4,
    T = 5,
    O = 6
}
export declare enum Cell {
    Empty = 0,
    Any = 1,
    S = 2,
    Z = 3,
    L = 4,
    J = 5,
    I = 6,
    T = 7,
    O = 8,
    Garbage = 9
}
export declare const pieceToCell: (p: Piece) => Cell;
export declare const cellToPiece: (c: Cell) => Piece | undefined;
export declare enum StatisticsEntryType {
    Single = 0,
    Double = 1,
    Triple = 2,
    Tetris = 3,
    Tst = 4,
    Tsd = 5,
    Tss = 6,
    Tsmd = 7,
    Tsms = 8,
    MaxCombos = 9,
    MaxBtbs = 10,
    PerfectClear = 11,
    Hold = 12,
    Lock = 13
}
export declare const STATISTICS_ENTRY_TYPES: StatisticsEntryType[];
export type Statistics = {
    [entryType in StatisticsEntryType]: number;
};
export type Game = {
    width: number;
    height: number;
    visibleHeight: number;
    cells: ArrayLike<Cell>;
    holdPiece?: Piece;
    nextPieces: ArrayLike<Piece>;
    currentNumCombos?: number;
    currentNumBTBs?: number;
    stats: Statistics;
};
export declare const getIndex: (w: number, x: number, y: number) => number;
