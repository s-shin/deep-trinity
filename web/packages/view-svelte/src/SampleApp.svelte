<script lang="ts">
  import * as core from "@deep-trinity/web-core";
  import * as coreHelper from "@deep-trinity/web-core-helper";
  import Grid from "./Grid.svelte";
  import NextPieces from "./NextPieces.svelte";
  import Panel from "./Panel.svelte";
  import Piece from "./Piece.svelte";
  import PiecePlaceholder from "./PiecePlaceholder.svelte";
  import Statistics from "./Statistics.svelte";
  import { GameRunner } from "./gameSystem";

  const game = new core.Game();
  game.supplyNextPieces(
    new Uint8Array([
      core.Piece.L,
      core.Piece.J,
      core.Piece.I,
      core.Piece.O,
      core.Piece.T,
      core.Piece.S,
      core.Piece.Z,
      core.Piece.L,
      core.Piece.J,
      core.Piece.I,
      core.Piece.O,
      core.Piece.T,
      core.Piece.S,
      core.Piece.Z,
    ]),
  );
  game.setupFallingPiece();
  $: g = coreHelper.getGameModel(game, true);
  let runner = new GameRunner(game, {
    onGameUpdated(game) {
      g = coreHelper.getGameModel(game, true);
    },
  });
  window.addEventListener("keydown", (ev) => {
    if (ev.repeat) return;
    if (ev.code === "KeyA") {
      console.log("start runner");
      runner.start();
    }
    if (ev.code === "KeyS") {
      console.log("stop runner");
      runner.stop();
    }
  });

  // function* run() {
  //   game.firmDrop();
  //   yield;
  //   game.lock();
  //   yield;

  //   game.shift(1, true);
  //   yield;
  //   game.firmDrop();
  //   yield;
  //   game.lock();
  //   yield;

  //   game.hold();
  //   yield;

  //   game.shift(-1, true);
  //   yield;
  //   game.firmDrop();
  //   yield;
  //   game.lock();
  //   yield;
  // }

  // window.addEventListener("keydown", (ev) => {
  //   // if (ev.repeat) return;
  //   switch (ev.code) {
  //     case "ArrowUp": {
  //       game.firmDrop();
  //       game.lock();
  //       break;
  //     }
  //     case "ArrowDown": {
  //       game.drop(1);
  //       break;
  //     }
  //     case "ArrowRight": {
  //       game.shift(1, false);
  //       break;
  //     }
  //     case "ArrowLeft": {
  //       game.shift(-1, false);
  //       break;
  //     }
  //     case "KeyZ": {
  //       game.rotate(-1);
  //       break;
  //     }
  //     case "KeyX": {
  //       game.rotate(1);
  //       break;
  //     }
  //     case "ShiftLeft": {
  //       game.hold();
  //       break;
  //     }
  //   }
  //   g = coreHelper.getGameModel(game, true);
  // });

  // (async () => {
  //   for (const _ of run()) {
  //     await new Promise((resolve) => setTimeout(resolve, 1000));
  //     g = coreHelper.getGameModel(game, true);
  //   }
  // })();
</script>

<div class="container">
  <div class="column left">
    <Panel title="HOLD">
      <div class="hold-piece">
        <PiecePlaceholder></PiecePlaceholder>
        <Piece piece={g.holdPiece}></Piece>
      </div>
    </Panel>
  </div>
  <div class="column center">
    <Grid cells={g.cells} numCols={g.width} numRows={g.visibleHeight}></Grid>
  </div>
  <div class="column right">
    <Panel title="NEXT">
      <PiecePlaceholder></PiecePlaceholder>
      <NextPieces pieces={g.nextPieces}></NextPieces>
    </Panel>
  </div>
  <div class="column">
    <Statistics stats={g.stats}></Statistics>
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
