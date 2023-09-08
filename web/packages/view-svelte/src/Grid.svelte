<script lang="ts">
  import { type Cell } from "@deep-trinity/model";
  export let numCols: number;
  export let numRows: number;
  export let cells: ArrayLike<Cell>;

  function* generateGridIndices() {
    for (let y = numRows - 1; y >= 0; y--) {
      const baseIndex = y * numCols;
      for (let x = 0; x < numCols; x++) {
        yield baseIndex + x;
      }
    }
  }
</script>

<div class="grid-wrapper">
  <div class="grid" style:--grid-cols={numCols} style:--grid-rows={numRows}>
    {#each generateGridIndices() as i}
      <div class="cell cell-{cells[i]}"></div>
    {/each}
  </div>
</div>

<style>
  .grid-wrapper {
    height: 100%;
    display: flex;
    justify-content: center;
  }
  .grid {
    height: 100%;
    aspect-ratio: calc(var(--grid-cols) / var(--grid-rows));
    box-sizing: border-box;
    display: grid;
    grid-template-columns: repeat(var(--grid-cols), var(--grid-cell-size, 1fr));
    gap: var(--grid-gap, 0px);
    border: var(--grid-border, 2px solid #ccc);
  }
  .cell {
    box-sizing: border-box;
    aspect-ratio: 1;
    border: var(--grid-cell-border, 1px solid #fff);
  }
  /* Empty */
  .cell-0 {
    background-color: #fff;
    border: var(--grid-empty-cell-border, 1px solid #eee);
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
