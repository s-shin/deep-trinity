<script lang="ts">
  import Piece from "./Piece.svelte";
</script>

<div class="container">
  <Piece></Piece>
</div>

<style>
  .container {
    height: var(--piece-placeholder-height, 0);
    overflow: hidden;
  }
</style>
