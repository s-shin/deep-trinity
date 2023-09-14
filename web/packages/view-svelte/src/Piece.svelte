<script lang="ts">
  import { Piece, Cell } from "@deep-trinity/model";
  import Grid from "./Grid.svelte";

  export let piece: Piece | undefined = void 0;

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
  const EMPTY_GRID = {
    numCols: 4,
    numRows: 2,
    cells: [E, E, E, E, E, E],
  };

  $: g = piece !== void 0 ? PIECE_GRIDS[piece] : EMPTY_GRID;
</script>

<div class="piece piece-{piece !== void 0 ? `${piece}` : 'empty'}">
  <Grid cells={g.cells} numCols={g.numCols} numRows={g.numRows}></Grid>
</div>

<style>
  @property --piece-grid-border {
    syntax: "*";
    inherits: true;
    initial-value: 0;
  }
  @property --piece-cell-border {
    syntax: "*";
    inherits: true;
    initial-value: 1px solid #fff;
  }
  @property --piece-empty-cell-border {
    syntax: "*";
    inherits: true;
    initial-value: 0;
  }
  @property --piece-padding {
    syntax: "*";
    inherits: true;
    initial-value: 10px;
  }
  @property --piece-i-padding {
    syntax: "*";
    inherits: true;
    initial-value: 22.5;
  }

  .piece {
    padding: var(--piece-padding) 0;
    --grid-border: var(--piece-grid-border);
    --grid-cell-border: var(--piece-cell-border);
    --grid-empty-cell-border: var(--piece-empty-cell-border);
  }
  .piece-4 {
    padding: var(--piece-i-padding, var(--piece-padding)) 0;
  }
</style>
