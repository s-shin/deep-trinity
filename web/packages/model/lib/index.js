export var Piece;
(function (Piece) {
    Piece[Piece["S"] = 0] = "S";
    Piece[Piece["Z"] = 1] = "Z";
    Piece[Piece["L"] = 2] = "L";
    Piece[Piece["J"] = 3] = "J";
    Piece[Piece["I"] = 4] = "I";
    Piece[Piece["T"] = 5] = "T";
    Piece[Piece["O"] = 6] = "O";
})(Piece || (Piece = {}));
export var Cell;
(function (Cell) {
    Cell[Cell["Empty"] = 0] = "Empty";
    Cell[Cell["Any"] = 1] = "Any";
    Cell[Cell["S"] = 2] = "S";
    Cell[Cell["Z"] = 3] = "Z";
    Cell[Cell["L"] = 4] = "L";
    Cell[Cell["J"] = 5] = "J";
    Cell[Cell["I"] = 6] = "I";
    Cell[Cell["T"] = 7] = "T";
    Cell[Cell["O"] = 8] = "O";
    Cell[Cell["Garbage"] = 9] = "Garbage";
})(Cell || (Cell = {}));
export const pieceToCell = (p) => p + 2;
export const cellToPiece = (c) => (c < 2 || 8 < c) ? undefined : c - 2;
export var StatisticsEntryType;
(function (StatisticsEntryType) {
    StatisticsEntryType[StatisticsEntryType["Single"] = 0] = "Single";
    StatisticsEntryType[StatisticsEntryType["Double"] = 1] = "Double";
    StatisticsEntryType[StatisticsEntryType["Triple"] = 2] = "Triple";
    StatisticsEntryType[StatisticsEntryType["Tetris"] = 3] = "Tetris";
    StatisticsEntryType[StatisticsEntryType["Tst"] = 4] = "Tst";
    StatisticsEntryType[StatisticsEntryType["Tsd"] = 5] = "Tsd";
    StatisticsEntryType[StatisticsEntryType["Tss"] = 6] = "Tss";
    StatisticsEntryType[StatisticsEntryType["Tsmd"] = 7] = "Tsmd";
    StatisticsEntryType[StatisticsEntryType["Tsms"] = 8] = "Tsms";
    StatisticsEntryType[StatisticsEntryType["MaxCombos"] = 9] = "MaxCombos";
    StatisticsEntryType[StatisticsEntryType["MaxBtbs"] = 10] = "MaxBtbs";
    StatisticsEntryType[StatisticsEntryType["PerfectClear"] = 11] = "PerfectClear";
    StatisticsEntryType[StatisticsEntryType["Hold"] = 12] = "Hold";
    StatisticsEntryType[StatisticsEntryType["Lock"] = 13] = "Lock";
})(StatisticsEntryType || (StatisticsEntryType = {}));
export const STATISTICS_ENTRY_TYPES = Object.values(StatisticsEntryType).filter(v => typeof v === "number");
export const getIndex = (w, x, y) => x + y * w;
//# sourceMappingURL=index.js.map