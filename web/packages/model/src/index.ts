export enum Piece {
  S,
  Z,
  L,
  J,
  I,
  T,
  O,
}

export enum Cell {
  Empty,
  Any,
  S,
  Z,
  L,
  J,
  I,
  T,
  O,
  Garbage,
}

export const pieceToCell = (p: Piece): Cell => p + 2;

export const cellToPiece = (c: Cell): Piece | undefined => (c < 2 || 8 < c) ? undefined : c - 2;

export enum StatisticsEntryType {
  Single,
  Double,
  Triple,
  Tetris,
  Tst,
  Tsd,
  Tss,
  Tsmd,
  Tsms,
  MaxCombos,
  MaxBtbs,
  PerfectClear,
  Hold,
  Lock,
}

export const STATISTICS_ENTRY_TYPES =
  Object.values(StatisticsEntryType).filter(v => typeof v === "number") as StatisticsEntryType[];

export type Statistics = {
  [entryType in StatisticsEntryType]: number;
};

export type Game = {
  width: number,
  height: number,
  visibleHeight: number,
  cells: ArrayLike<Cell>,
  holdPiece?: Piece,
  nextPieces: ArrayLike<Piece>,
  currentNumCombos?: number,
  currentNumBTBs?: number,
  stats: Statistics,
};

export const getIndex = (w: number, x: number, y: number): number => x + y * w;
