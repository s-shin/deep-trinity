<script lang="ts">
  import * as model from "@deep-trinity/model";
  import Grid from "./Grid.svelte";
  import NextPieces from "./NextPieces.svelte";
  import Panel from "./Panel.svelte";
  import Piece from "./Piece.svelte";
  import PiecePlaceholder from "./PiecePlaceholder.svelte";
  import Statistics from "./Statistics.svelte";
  export let game: model.Game;
</script>

<div class="container">
  <div class="column left">
    <Panel title="HOLD">
      <div class="hold-piece">
        <PiecePlaceholder></PiecePlaceholder>
        <Piece piece={game.holdPiece}></Piece>
      </div>
    </Panel>
  </div>
  <div class="column center">
    <Grid cells={game.cells} numCols={game.width} numRows={game.visibleHeight}
    ></Grid>
  </div>
  <div class="column right">
    <Panel title="NEXT">
      <PiecePlaceholder></PiecePlaceholder>
      <NextPieces pieces={game.nextPieces}></NextPieces>
    </Panel>
  </div>
  <div class="column">
    <Statistics stats={game.stats}></Statistics>
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
