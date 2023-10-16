<script lang="ts">
  import { Game } from "@deep-trinity/model";
  import Grid from "./Grid.svelte";
  export let game: Game;
  // TODO: ghost
</script>

<Grid cells={game.cells} numCols={game.width} numRows={game.visibleHeight}
></Grid>
