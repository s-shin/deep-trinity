<script lang="ts">
  import * as coreHelper from "@deep-trinity/web-core-helper";
  import { GameRunner } from "./gameSystem";
  import SinglePlay from "./SinglePlay.svelte";

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
          fps = frameSpan.frameCount.toString();
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
  $: fps = "";
</script>

<div class="fps">{fps}</div>
<SinglePlay {game}></SinglePlay>

<style>
  .fps {
    position: fixed;
    top: 0;
    left: 0;
    font-size: 0.6em;
    padding: 0.3em 0.5em;
  }
</style>
