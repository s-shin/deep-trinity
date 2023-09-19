<script lang="ts">
  import * as coreHelper from "@deep-trinity/web-core-helper";
  import Grid from "./Grid.svelte";
  import NextPieces from "./NextPieces.svelte";
  import Panel from "./Panel.svelte";
  import Piece from "./Piece.svelte";
  import PiecePlaceholder from "./PiecePlaceholder.svelte";
  import Statistics from "./Statistics.svelte";
  import { GameRunner } from "./gameSystem";

  function newGameRunner() {
    let frameSpan = { startedAt: 0, frameCount: 0 };
    return new GameRunner({
      onFrameEntered(runner, dt) {
        let now = performance.now();
        if (frameSpan.startedAt === 0) {
          frameSpan.startedAt = now;
          return;
        }
        frameSpan.frameCount++;
        if (now - frameSpan.startedAt > 1000) {
          console.log(frameSpan.frameCount);
          frameSpan.startedAt = now;
          frameSpan.frameCount = 0;
        }
      },
      onGameUpdated(runner) {
        game = coreHelper.getGameModel(runner.game, true);
      },
    });
  }

  let ctx = {
    runner: newGameRunner(),
  };

  window.addEventListener("keydown", (ev) => {
    if (ev.repeat) return;
    if (ev.code === "KeyA") {
      console.log("start runner");
      ctx.runner.start();
    }
    if (ev.code === "KeyS") {
      console.log("stop runner");
      ctx.runner.stop();
    }
    if (ev.code === "KeyR") {
      console.log("reset game");
      ctx.runner.stop();
      ctx.runner = newGameRunner();
    }
  });

  $: game = coreHelper.getGameModel(ctx.runner.game, true);
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
