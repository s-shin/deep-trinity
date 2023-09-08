<script lang="ts">
  import { Piece, Cell } from "@deep-trinity/model";
  import Grid from "./Grid.svelte";
  export let piece: Piece;

  const [E, S, Z, L, J, I, T, O] = [
    Cell.Empty,
    Cell.S,
    Cell.Z,
    Cell.L,
    Cell.J,
    Cell.I,
    Cell.T,
    Cell.O,
  ];
  const PIECE_GRIDS = {
    [Piece.S]: {
      numCols: 3,
      numRows: 2,
      cells: [S, S, E, E, S, S],
    },
    [Piece.Z]: {
      numCols: 3,
      numRows: 2,
      cells: [E, Z, Z, Z, Z, E],
    },
    [Piece.L]: {
      numCols: 3,
      numRows: 2,
      cells: [L, L, L, E, E, L],
    },
    [Piece.J]: {
      numCols: 3,
      numRows: 2,
      cells: [J, J, J, J, E, E],
    },
    [Piece.I]: {
      numCols: 4,
      numRows: 1,
      cells: [I, I, I, I],
    },
    [Piece.T]: {
      numCols: 3,
      numRows: 2,
      cells: [T, T, T, E, T, E],
    },
    [Piece.O]: {
      numCols: 2,
      numRows: 2,
      cells: [O, O, O, O],
    },
  };

  $: g = PIECE_GRIDS[piece];
</script>

<div class="piece">
  <Grid cells={g.cells} numCols={g.numCols} numRows={g.numRows}></Grid>
</div>

<style>
  .piece {
    --grid-border: var(--piece-grid-border, none);
    --grid-cell-border: var(--piece-cell-border);
    --grid-empty-cell-border: var(--piece-empty-cell-border, 1px solid #fff);
  }
</style>
