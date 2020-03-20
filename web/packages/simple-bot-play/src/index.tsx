import React from "react";
import ReactDOM from "react-dom";
import { ThemeProvider } from "emotion-theming";
import * as core from "@deep-trinity/web-core";
import * as view from "@deep-trinity/view";

const game = new core.Game();

ReactDOM.render(
  <ThemeProvider theme={view.DEFAULT_THEME}>
    {/*<view.SimpleFullScreenSinglePlay game={game}/>*/}
  </ThemeProvider>,
  document.querySelector("main"),
);

const run = async () => {
  const pg = new core.RandomPieceGenerator(BigInt(0));
  const bot = new core.SimpleBot();
  console.log(pg.generate());
  console.log(pg.generate());

  game.supplyNextPieces(pg.generate());
  game.setupFallingPiece();
  if (true) {
    return;
  }
  for (let i = 0; i < 3; i++) {
    if (game.shouldSupplyNextPieces()) {
      game.supplyNextPieces(pg.generate());
    }
    const dst = bot.think(game);
    if (dst === undefined) {
      break;
    }
    const movePlayer = core.MovePlayer.from(game, dst!);
    console.log(movePlayer);
    while (movePlayer.step(game)) {
      await new Promise(resolve => setTimeout(resolve, 250));
    }
  }
};

run().catch(e => console.error(e));
