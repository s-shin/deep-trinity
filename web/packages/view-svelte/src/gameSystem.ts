import { Game } from "@deep-trinity/web-core";

const Input = {
  UP: 0b0000_0001,
  DOWN: 0b0000_0010,
  RIGHT: 0b0000_0100,
  LEFT: 0b0000_1000,
  CW: 0b0001_0000,
  CCW: 0b0010_0000,
  HOLD: 0b0100_0000,
} as const;

type InputName = keyof typeof Input;
type Input = (typeof Input)[InputName];

class InputState {
  private state: number;
  constructor(state = 0) {
    this.state = state;
  }
  clone() {
    return new InputState(this.state);
  }
  reset() {
    this.state = 0;
  }
  on(t: Input) {
    this.state |= t;
  }
  off(t: Input) {
    this.state &= ~t;
  }
  isOn(t: Input) {
    return (this.state & t) !== 0;
  }
  isOff(t: Input) {
    return (this.state & t) === 0;
  }
}

const DEFAULT_INPUT_MAPPING = {
  ArrowUp: Input.UP,
  ArrowDown: Input.DOWN,
  ArrowRight: Input.RIGHT,
  ArrowLeft: Input.LEFT,
  KeyZ: Input.CCW,
  KeyX: Input.CW,
  ShiftLeft: Input.HOLD,
} as { [key: string]: Input };

export class InputWatcher {
  private isWatching = false;
  private inputState = new InputState();
  private onKeyDown: (ev: KeyboardEvent) => void;
  private onKeyUp: (ev: KeyboardEvent) => void;
  constructor() {
    this.onKeyDown = (ev) => {
      if (ev.repeat) return;
      const input = DEFAULT_INPUT_MAPPING[ev.code];
      if (input === void 0) return;
      this.inputState.on(input);
    };
    this.onKeyUp = (ev) => {
      const input = DEFAULT_INPUT_MAPPING[ev.code];
      if (input === void 0) return;
      this.inputState.off(input);
    };
  }
  watch() {
    if (this.isWatching) return;
    this.isWatching = true;
    this.inputState.reset();
    window.addEventListener("keydown", this.onKeyDown);
    window.addEventListener("keyup", this.onKeyUp);
  }
  unwatch() {
    if (!this.isWatching) return;
    this.isWatching = false;
    window.removeEventListener("keydown", this.onKeyDown);
    window.removeEventListener("keyup", this.onKeyUp);
  }
  isOn(input: Input) {
    return this.inputState.isOn(input);
  }
  dumpState() {
    return this.inputState.clone();
  }
}

const DEFAULT_GAME_RUNNER_OPTIONS = {
  frameTimeMs: 16,
  onGameUpdated(game: Game) {},
  gravity: 0.02 as number | ((frame: number, game: Game) => number),
  softDropGravity: 5,
  // https://harddrop.com/wiki/DAS
  dasDelayFrameCount: 3,
  // TODO: ARE, IRS, IHS
  // https://harddrop.com/wiki/ARE
  // areFrameCount: number;
  // https://harddrop.com/wiki/IRS#IRS
  // isIrsEnabled: boolean;
  // https://harddrop.com/wiki/IHS
  // isIhsEnabled: boolean;
};

type GameRunnerOptions = typeof DEFAULT_GAME_RUNNER_OPTIONS;

// cf. https://developer.mozilla.org/ja/docs/Games/Anatomy
export class GameRunner {
  private inputWatcher = new InputWatcher();
  private currentFrame = 0;
  private lastTime = 0;
  private frameRequestId?: number;
  private prevInputState = new InputState();
  private rightDasFrameCounter = 0;
  private leftDasFrameCounter = 0;
  private accumulatedGravity = 0;
  public game: Game;
  private opts: GameRunnerOptions;

  constructor(game: Game, opts: Partial<GameRunnerOptions>) {
    this.game = game;
    this.opts = { ...DEFAULT_GAME_RUNNER_OPTIONS, ...opts };
  }

  get isRunning() {
    return this.frameRequestId !== void 0;
  }

  start() {
    if (this.frameRequestId !== void 0) return;
    this.inputWatcher.watch();
    this.update();
  }

  stop() {
    if (this.frameRequestId === void 0) return;
    this.inputWatcher.unwatch();
    cancelAnimationFrame(this.frameRequestId);
  }

  private getGravity() {
    return typeof this.opts.gravity == "number"
      ? this.opts.gravity
      : this.opts.gravity(this.currentFrame, this.game);
  }

  private update() {
    this.frameRequestId = requestAnimationFrame(() => this.update());
    let now = performance.now();
    let dt = now - this.lastTime;
    if (dt < this.opts.frameTimeMs) return;
    this.lastTime = now;

    let isGameUpdated = false;
    // hold
    {
      let isOn = this.inputWatcher.isOn(Input.HOLD);
      let isOnPrev = this.prevInputState.isOn(Input.HOLD);
      if (isOn && !isOnPrev) {
        try {
          this.game.hold();
          isGameUpdated = true;
        } catch {
          // do nothing
        }
      }
    }
    // up
    {
      let isOn = this.inputWatcher.isOn(Input.UP);
      let isOnPrev = this.prevInputState.isOn(Input.UP);
      if (isOn && !isOnPrev) {
        this.game.firmDrop();
        this.game.lock();
        isGameUpdated = true;
      }
    }
    // down
    {
      let isOn = this.inputWatcher.isOn(Input.DOWN);
      if (isOn) {
        this.game.drop(Math.min(1, this.opts.softDropGravity));
        isGameUpdated = true;
      }
    }
    // cw, ccw
    {
      let isCwOn = this.inputWatcher.isOn(Input.CW);
      let isCwOnPrev = this.prevInputState.isOn(Input.CW);
      let isCcwOn = this.inputWatcher.isOn(Input.CCW);
      let isCcwOnPrev = this.prevInputState.isOn(Input.CCW);
      if (isCwOn && !isCcwOn && !isCwOnPrev) {
        try {
          this.game.rotate(1);
          isGameUpdated = true;
        } catch {
          // do nothing
        }
      }
      if (isCcwOn && !isCwOn && !isCcwOnPrev) {
        try {
          this.game.rotate(-1);
          isGameUpdated = true;
        } catch {
          // do nothing
        }
      }
    }
    // right, left
    {
      let isRightOn = this.inputWatcher.isOn(Input.RIGHT);
      let isLeftOn = this.inputWatcher.isOn(Input.LEFT);
      if (isRightOn && !isLeftOn) {
        let c = this.rightDasFrameCounter;
        if (c == 0 || c > this.opts.dasDelayFrameCount) {
          try {
            this.game.shift(1, false);
            isGameUpdated = true;
          } catch {
            // do nothing
          }
        }
        this.rightDasFrameCounter++;
      } else {
        this.rightDasFrameCounter = 0;
      }
      if (isLeftOn && !isRightOn) {
        let c = this.leftDasFrameCounter;
        if (c == 0 || c > this.opts.dasDelayFrameCount) {
          try {
            this.game.shift(-1, false);
            isGameUpdated = true;
          } catch {
            // do nothing
          }
        }
        this.leftDasFrameCounter++;
      } else {
        this.leftDasFrameCounter = 0;
      }
    }
    // automatic drop
    {
      this.accumulatedGravity += this.getGravity();
      let n = Math.floor(this.accumulatedGravity);
      if (n > 0) {
        this.game.drop(n);
        isGameUpdated = true;
        this.accumulatedGravity = 0;
      }
    }

    if (isGameUpdated) {
      this.opts.onGameUpdated(this.game);
    }
    this.prevInputState = this.inputWatcher.dumpState();
    this.currentFrame++;
  }
}
