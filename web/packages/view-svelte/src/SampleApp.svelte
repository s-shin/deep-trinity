<script lang="ts">
  import Grid from "./Grid.svelte";
  import NextPieces from "./NextPieces.svelte";
  import { type Game } from "@deep-trinity/model";
  export let game: Game;
</script>

<div class="container">
  <div class="column left">TODO</div>
  <div class="column center">
    <Grid cells={game.cells} numCols={game.width} numRows={game.visibleHeight}
    ></Grid>
  </div>
  <div class="column right">
    <NextPieces pieces={game.nextPieces}></NextPieces>
  </div>
</div>

<style>
  :global(html) {
    width: 100%;
    height: 100%;
  }
  :global(body) {
    margin: 7.5vh 0;
  }
  .container {
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
  }
  .column {
    margin: 0 10px;
  }
</style>
