import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import { ThemeProvider } from '@emotion/react';
import { useForm } from "react-hook-form";
import * as core from "@deep-trinity/web-core";
import * as coreHelper from "@deep-trinity/web-core-helper";
import * as view from "@deep-trinity/view";
import * as model from "@deep-trinity/model";
import styled from "@emotion/styled";

core.setPanicHook();

enum BotRunnerState {
  Think,
  Move,
}

class BotRunner {
  private state = BotRunnerState.Think;
  private game = new core.Game();
  private pg?: core.RandomPieceGenerator;
  private bot = new core.Bot();
  private movePlayer?: core.MovePlayer;
  public isRunning = false;

  constructor() {
    // this.setup(0);
  }

  shouldSetup(): boolean {
    return !this.pg;
  }

  setup(bot: number, seed: number): void {
    this.state = BotRunnerState.Think;
    this.game = new core.Game();
    this.pg = new core.RandomPieceGenerator(BigInt(seed));
    this.bot = new core.Bot(bot);

    this.game.supplyNextPieces(this.pg.generate());
    this.game.setupFallingPiece();
  }

  getGameModel(): model.Game {
    return coreHelper.getGameModel(this.game, true);
  }

  isEnd(): boolean {
    return this.game.isGameOver();
  }

  update(): void {
    const { game, pg, bot } = this;
    if (!pg) {
      throw new Error("should setup");
    }
    switch (this.state) {
      case BotRunnerState.Think: {
        if (game.shouldSupplyNextPieces()) {
          game.supplyNextPieces(pg.generate());
        }
        const action = bot.think(game);
        if (action.isHold()) {
          game.hold();
          break;
        }
        const dst = action.dst();
        if (dst === undefined) {
          break;
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

  async run(update: (isRunning: boolean) => void, sleep_ms = 100): Promise<void> {
    if (this.isRunning) {
      return;
    }
    this.isRunning = true;
    while (this.isRunning) {
      await new Promise(resolve => setTimeout(resolve, sleep_ms));
      this.update();
      update(this.isRunning);
      if (!this.isRunning || this.isEnd()) {
        break;
      }
    }
    this.isRunning = false;
    update(this.isRunning);
  }

  stop(): void {
    this.isRunning = false;
  }
}

const botRunner = new BotRunner();

//---

const randInt = (min: number, max: number): number => min + Math.floor(Math.random() * max);

const ControlPanelRoot = styled.div``;

type SetupFormValue = {
  bot: number,
  seed: number,
};

type RunTriggerFormValue = {
  sleep_ms: number,
}

type ControlPanelProps = {
  isRunning: boolean,
  onSetup: (data: SetupFormValue) => void,
  onToggleRun: (data: RunTriggerFormValue) => void,
};

const ControlPanel: React.FC<ControlPanelProps> = props => {
  const runTriggerForm = useForm<RunTriggerFormValue>();
  const setupForm = useForm<SetupFormValue>();
  return (
    <ControlPanelRoot>
      <div>
        <form onSubmit={runTriggerForm.handleSubmit(props.onToggleRun)}>
          <p>
            <label htmlFor="sleep_ms">Sleep: </label>
            <input {...runTriggerForm.register("sleep_ms", { value: 100 })}/> ms
          </p>
          <p>
            <button>{props.isRunning ? "Stop" : "Start"}</button>
          </p>
        </form>
      </div>
      <p>
      </p>
      <div>
        <form onSubmit={setupForm.handleSubmit(props.onSetup)}>
          <p>
            <label htmlFor="bot">Bot: </label>
            <select {...setupForm.register("bot", { value: 2 })}>
              <option value="1">Simple</option>
              <option value="2">Simple Tree</option>
              <option value="3">MCTS (PUCT)</option>
            </select>
          </p>
          <p>
            <label htmlFor="seed">Seed: </label>
            <input {...setupForm.register("seed", { value: 0 })}/>
            <button type="button" onClick={() => setupForm.setValue("seed", randInt(0, 1000000000))}>Generate</button>
          </p>
          <p>
            <button>Setup</button>
          </p>
        </form>
      </div>
    </ControlPanelRoot>
  );
};

//---

const App: React.FC = () => {
  const [gameModel, setGameModel] = useState(botRunner.getGameModel());
  const [isRunning, setIsRunning] = useState(false);

  const onSetup = (data: SetupFormValue): void => {
    botRunner.setup(data.bot, data.seed);
    setGameModel(botRunner.getGameModel());
  };

  const onToggleRun = (data: RunTriggerFormValue): void => {
    if (botRunner.shouldSetup()) {
      return;
    }
    if (isRunning) {
      botRunner.stop();
    } else {
      botRunner.run(isRunning => {
        setGameModel(botRunner.getGameModel());
        setIsRunning(isRunning);
      }, data.sleep_ms).catch(e => console.error(e));
    }
  };

  return (
    <ThemeProvider theme={view.DEFAULT_THEME}>
      <view.SimpleFullScreenSinglePlay game={gameModel}>
        <ControlPanel isRunning={isRunning} onSetup={onSetup} onToggleRun={onToggleRun}/>
      </view.SimpleFullScreenSinglePlay>
    </ThemeProvider>
  );
};

createRoot(document.querySelector("main")!).render(<App />);
