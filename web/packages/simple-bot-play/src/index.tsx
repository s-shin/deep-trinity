import React, { useState } from "react";
import ReactDOM from "react-dom";
import { ThemeProvider } from "emotion-theming";
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
  private bot = new core.SimpleBot();
  private movePlayer?: core.MovePlayer;
  public isRunning = false;

  constructor() {
    // this.setup(0);
  }

  shouldSetup(): boolean {
    return !this.pg;
  }

  setup(seed: number): void {
    this.state = BotRunnerState.Think;
    this.game = new core.Game();
    this.pg = new core.RandomPieceGenerator(BigInt(seed));
    this.bot = new core.SimpleBot();

    this.game.supplyNextPieces(this.pg.generate());
    this.game.setupFallingPiece();
    this.game.hold(); // FIXME: remove
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

const randInt = (min: number, max: number) => min + Math.floor(Math.random() * max);

const ControlPanelRoot = styled.div``;

type SetupFormValue = {
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
            <input id="sleep_ms" name="sleep_ms" type="number" defaultValue={100} ref={runTriggerForm.register}/> ms
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
            <label htmlFor="seed">Seed: </label>
            <input id="seed" name="seed" type="number" defaultValue={0} ref={setupForm.register}/>
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
    botRunner.setup(data.seed);
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

ReactDOM.render(<App/>, document.querySelector("main"));
