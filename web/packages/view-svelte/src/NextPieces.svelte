<script lang="ts">
  import PieceComponent from "./Piece.svelte";
  import { type Piece } from "@deep-trinity/model";
  export let pieces: ArrayLike<Piece>;
</script>

<div class="next-pieces-wrapper">
  {#each pieces as p}
    <div class="next-pieces piece-{p}">
      <PieceComponent piece={p}></PieceComponent>
    </div>
  {/each}
</div>

<style>
  .next-pieces-wrapper {
    height: 100%;
    box-sizing: border-box;
  }
  .next-pieces {
    height: var(--piece-height, 10%);
    margin-bottom: 15%;
  }
  .next-pieces.piece-4 {
    height: calc(var(--piece-height, 10%) / 2);
    margin: 20% 0;
  }
</style>
