<script lang="ts" context="module">
  import { Cell } from "@deep-trinity/model";

  export const CellStyle = {
    NORMAL: 0,
    GHOST: 1,
  } as const;

  export type CellStyle = (typeof CellStyle)[keyof typeof CellStyle];

  export type CellInfo = {
    cell: Cell;
    style?: CellStyle;
  };
</script>

<script lang="ts">
  export let numCols: number;
  export let numRows: number;
  export let cells: Iterable<Cell | CellInfo>;

  function* iterate(
    cells: Iterable<Cell | CellInfo>,
  ): Iterable<CellInfo & { order: number }> {
    let it = cells[Symbol.iterator]();
    for (let y = numRows - 1; y >= 0; y--) {
      let orderBase = y * numCols;
      for (let x = 0; x < numCols; x++) {
        let order = orderBase + x;
        let cell = it.next();
        if (cell.done) {
          yield { cell: Cell.Empty, order };
        } else if (typeof cell.value === "number") {
          yield { cell: cell.value, order };
        } else {
          yield { ...cell.value, order };
        }
      }
    }
  }

  $: it = iterate(cells);
</script>

<div class="grid-wrapper">
  <div class="grid" style:--_grid-cols={numCols} style:--_grid-rows={numRows}>
    {#each it as { cell, style, order }}
      <div
        class="cell cell-{cell} {style === CellStyle.GHOST ? 'ghost' : ''}"
        style="order: {order}"
      ></div>
    {/each}
  </div>
</div>

<style>
  @property --cell-size {
    syntax: "<length-percentage>";
    inherits: true;
    initial-value: 25px;
  }
  @property --grid-gap {
    syntax: "<length-percentage>";
    inherits: true;
    initial-value: 0px;
  }
  @property --grid-border {
    syntax: "*";
    inherits: true;
    initial-value: 2px solid #ccc;
  }
  @property --grid-cell-border {
    syntax: "*";
    inherits: true;
    initial-value: 1px solid #fff;
  }
  @property --grid-empty-cell-border {
    syntax: "*";
    inherits: true;
    initial-value: 1px solid #eee;
  }

  .grid-wrapper {
    display: flex;
    justify-content: center;
  }
  .grid {
    aspect-ratio: calc(var(--_grid-cols) / var(--_grid-rows));
    box-sizing: border-box;
    display: grid;
    grid-template-columns: repeat(var(--_grid-cols), var(--cell-size));
    gap: var(--grid-gap);
    border: var(--grid-border);
  }
  .cell {
    box-sizing: border-box;
    aspect-ratio: 1;
    border: var(--grid-cell-border);
  }
  .ghost {
    opacity: 0.5; /* TODO: prop */
  }
  /* Empty */
  .cell-0 {
    background-color: #fff;
    border: var(--grid-empty-cell-border);
  }
  /* Any */
  .cell-1 {
    background-color: darkslategray;
  }
  /* S */
  .cell-2 {
    background-color: darkolivegreen;
  }
  /* Z */
  .cell-3 {
    background-color: sienna;
  }
  /* L */
  .cell-4 {
    background-color: darkorange;
  }
  /* J */
  .cell-5 {
    background-color: darkblue;
  }
  /* I */
  .cell-6 {
    background-color: turquoise;
  }
  /* T */
  .cell-7 {
    background-color: purple;
  }
  /* O */
  .cell-8 {
    background-color: gold;
  }
  /* GARBAGE */
  .cell-9 {
    background-color: gray;
  }
</style>
