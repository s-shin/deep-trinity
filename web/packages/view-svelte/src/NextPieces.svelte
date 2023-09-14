<script lang="ts">
  import PieceComponent from "./Piece.svelte";
  import { type Piece } from "@deep-trinity/model";
  export let pieces: ArrayLike<Piece>;
</script>

<div>
  {#each pieces as p}
    <div class="piece piece-{p}">
      <PieceComponent piece={p}></PieceComponent>
    </div>
  {/each}
</div>

<style>
</style>
