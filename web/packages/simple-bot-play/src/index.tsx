import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom";
import { ThemeProvider } from "emotion-theming";
import * as core from "@deep-trinity/web-core";
import * as coreHelper from "@deep-trinity/web-core-helper";
import * as view from "@deep-trinity/view";
import * as model from "@deep-trinity/model";

enum BotRunnerState {
  Think,
  Move,
}

class BotRunner {
  state: BotRunnerState;
  game: core.Game;
  pg: core.RandomPieceGenerator;
  bot: core.SimpleBot;
  movePlayer?: core.MovePlayer;

  constructor() {
    this.state = BotRunnerState.Think;
    this.game = new core.Game();
    this.pg = new core.RandomPieceGenerator(BigInt(3));
    this.bot = new core.SimpleBot();

    this.game.supplyNextPieces(this.pg.generate());
    this.game.setupFallingPiece();
  }
  getGameModel(): model.Game {
    return coreHelper.getGameModel(this.game, true);
  }
  isEnd(): boolean {
    return this.game.isGameOver();
  }
  update() {
    const { game, pg, bot } = this;
    switch (this.state) {
      case BotRunnerState.Think: {
        if (game.shouldSupplyNextPieces()) {
          game.supplyNextPieces(pg.generate());
        }
        const dst = bot.think(game);
        if (dst === undefined) {
          return;
        }
        this.movePlayer = core.MovePlayer.from(game, dst);
        this.state = BotRunnerState.Move;
        break;
      }
      case BotRunnerState.Move: {
        if (!this.movePlayer) {
          throw new Error("movePlayer should not be undefined");
        }
        if (!this.movePlayer.step(game)) {
          this.movePlayer = undefined;
          game.lock();
          this.state = BotRunnerState.Think;
        }
        break;
      }
    }
  }
}

const botRunner = new BotRunner();

const App: React.FC = () => {
  const [gameModel, setGameModel] = useState(botRunner.getGameModel());

  useEffect(() => {
    (async () => {
      for (let i = 0; i < 1000; i++) {
        await new Promise(resolve => setTimeout(resolve, 10));
        botRunner.update();
        setGameModel(botRunner.getGameModel());
        if (botRunner.isEnd()) {
          break;
        }
      }
    })().catch(e => console.log(e));
  }, []);

  return (
    <ThemeProvider theme={view.DEFAULT_THEME}>
      <view.SimpleFullScreenSinglePlay game={gameModel}/>
    </ThemeProvider>
  );
};

ReactDOM.render(<App/>, document.querySelector("main"));
