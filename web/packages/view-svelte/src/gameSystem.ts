// TODO: move to another package.
import { ActionHint, Game, Placement, RandomPieceGenerator } from "@deep-trinity/web-core";

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

//---

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

//---

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

//---

const DEFAULT_DAS_OPTIONS = {
  autoShiftDelayFrameCount: 10,
  autoRepeatIntervalFrameCount: 2,
};

type DasOptions = typeof DEFAULT_DAS_OPTIONS;

// cf. https://harddrop.com/wiki/DAS
class DasFrameCounter {
  private count = 0;
  private _shouldShift = false;
  private opts: DasOptions;
  constructor(opts: Partial<DasOptions>) {
    this.opts = { ...DEFAULT_DAS_OPTIONS, ...opts };
  }
  get shouldShift() {
    return this._shouldShift;
  }
  update(active: boolean) {
    if (active) {
      this.count++;
      this._shouldShift = this.check();
      return this._shouldShift;
    } else {
      this.count = 0;
      this._shouldShift = false;
      return false;
    }
  }
  private check() {
    if (this.count === 1) return true;
    let n = this.count - 2 - this.opts.autoShiftDelayFrameCount;
    if (n < 0) return false;
    if (n === 0) return true;
    if (this.opts.autoRepeatIntervalFrameCount === 0) {
      this.count--;
      return false;
    }
    if (n % this.opts.autoRepeatIntervalFrameCount === 0) {
      this.count -= this.opts.autoRepeatIntervalFrameCount;
      return true;
    }
    return false;
  }
}

//---

// TODO: delay on clearing
// TODO: show fps
// interface CustomCallback {
//   intervalMs: number;
//   callback: (runner: GameRunner) => void;
// }

interface InternalProcessResult {
  isLockedOrHeld?: boolean;
  isGameUpdated?: boolean;
  isActionDone?: boolean;
}

interface GameRunnerOptions {
  game?: Game;
  frameTimeMs: number;
  gravity: number | ((runner: GameRunner) => number);
  softDropGravity: number;
  das: Partial<DasOptions>;
  lockDelayFrameCount: number;
  moveResetLimit: number;
  // TODO: ARE, IRS, IHS
  // https://harddrop.com/wiki/ARE
  // areFrameCount: number;
  // https://harddrop.com/wiki/IRS#IRS
  // isIrsEnabled: boolean;
  // https://harddrop.com/wiki/IHS
  // isIhsEnabled: boolean;
  nextPieceGenerator(): Uint8Array;
  onFrameEntered(runner: GameRunner, dtMs: number): void;
  onGameUpdated(runner: GameRunner): void;
}

const DEFAULT_RANDOM_PIECE_GENERATOR = new RandomPieceGenerator(0n);

const DEFAULT_GAME_RUNNER_OPTIONS: GameRunnerOptions = {
  frameTimeMs: 13,
  gravity: 0.02,
  softDropGravity: 5,
  das: DEFAULT_DAS_OPTIONS as Partial<DasOptions>,
  lockDelayFrameCount: 30,
  moveResetLimit: 15,
  nextPieceGenerator: () => DEFAULT_RANDOM_PIECE_GENERATOR.generate(),
  onFrameEntered(runner: GameRunner, dtMs: number) {},
  onGameUpdated(runner: GameRunner) {},
};

// cf. https://developer.mozilla.org/ja/docs/Games/Anatomy
export class GameRunner {
  private opts: GameRunnerOptions;
  private _game: Game;
  private _currentFrame = 0;
  private lastFrameTime = 0;
  private _lsatFrameDeltaTime = 0;
  private frameRequestId = null as number | null;
  private inputWatcher = new InputWatcher();
  private prevInputState = new InputState();
  private lockDelayFrameCounter = 0;
  private moveResetCounter = 0;
  private fpMostBottomY: number;
  private accumulatedGravity = 0;
  private rightwardDasFrameCounter: DasFrameCounter;
  private leftwardDasFrameCounter: DasFrameCounter;

  constructor(opts: Partial<GameRunnerOptions>) {
    this.opts = { ...DEFAULT_GAME_RUNNER_OPTIONS, ...opts };
    let game = this.opts.game;
    if (!game) {
      game = new Game();
      game.supplyNextPieces(this.opts.nextPieceGenerator());
    }
    this._game = game;
    this.rightwardDasFrameCounter = new DasFrameCounter(this.opts.das);
    this.leftwardDasFrameCounter = new DasFrameCounter(this.opts.das);
    this.fpMostBottomY = this._game.height();
  }

  get currentFrame() {
    return this._currentFrame;
  }
  get lastFrameDeltaTime() {
    return this._lsatFrameDeltaTime;
  }
  get game() {
    return this._game;
  }
  get isRunning() {
    return this.frameRequestId !== void 0;
  }

  start() {
    if (this.frameRequestId !== null) return;
    this.inputWatcher.watch();
    this.update();
  }

  stop() {
    if (this.frameRequestId === null) return;
    this.inputWatcher.unwatch();
    cancelAnimationFrame(this.frameRequestId);
    this.frameRequestId = null;
  }

  private getGravity() {
    return typeof this.opts.gravity == "number" ? this.opts.gravity : this.opts.gravity(this);
  }

  private resetOnLockedOrHeld() {
    this.lockDelayFrameCounter = 0;
    this.moveResetCounter = 0;
    this.fpMostBottomY = this.game.height();
    this.accumulatedGravity = 0;
    this.rightwardDasFrameCounter.update(false);
    this.leftwardDasFrameCounter.update(false);
  }

  private update() {
    this.frameRequestId = requestAnimationFrame(() => this.update());
    let now = performance.now();
    let dt = now - this.lastFrameTime;
    if (dt < this.opts.frameTimeMs) return;
    this.lastFrameTime = now;
    this._lsatFrameDeltaTime = dt;

    this.opts.onFrameEntered(this, dt);

    if (this._game.shouldSupplyNextPieces()) {
      this._game.supplyNextPieces(this.opts.nextPieceGenerator());
    }

    let fp = this._game.falling_piece_placement();
    if (!fp) {
      try {
        this._game.setupFallingPiece();
        this.opts.onGameUpdated(this);
      } catch {
        this.stop();
      }
      return;
    }

    let r = this.process();
    let isGameUpdated = !!r.isGameUpdated;
    if (r.isLockedOrHeld) {
      isGameUpdated = true;
      this.resetOnLockedOrHeld();
    } else if (r.isActionDone) {
      isGameUpdated = true;
      this.accumulatedGravity = 0;
      this.lockDelayFrameCounter = 0;
      if (fp.y < this.fpMostBottomY) {
        this.moveResetCounter = 0;
        this.fpMostBottomY = fp.y;
      } else if (fp.y === this.fpMostBottomY) {
        this.moveResetCounter++;
      }
    }
    if (isGameUpdated) {
      this.opts.onGameUpdated(this);
    }
    this.prevInputState = this.inputWatcher.dumpState();
    this._currentFrame++;
  }

  private process(): InternalProcessResult {
    let isGameUpdated = false;
    let hint = this._game.action_hint();

    if (hint.drop === 0) {
      // Extended Placement Lock Down
      if (
        hint.drop === 0 &&
        (this.lockDelayFrameCounter++ >= this.opts.lockDelayFrameCount ||
          this.moveResetCounter >= this.opts.moveResetLimit)
      ) {
        this.game.lock();
        return { isLockedOrHeld: true };
      }
    } else {
      // Gravity
      this.accumulatedGravity += this.getGravity();
      let n = Math.floor(this.accumulatedGravity);
      if (n > 0) {
        this._game.drop(Math.min(n, hint.drop));
        this.accumulatedGravity = 0;
        isGameUpdated = true;
        hint = this._game.action_hint();
      }
    }

    let isActionDone = false;
    // Hold
    if (hint.hold) {
      let isOn = this.inputWatcher.isOn(Input.HOLD);
      let isOnPrev = this.prevInputState.isOn(Input.HOLD);
      if (isOn && !isOnPrev) {
        this._game.hold();
        return { isLockedOrHeld: true };
      }
    }
    // Hard drop (up)
    {
      let isOn = this.inputWatcher.isOn(Input.UP);
      let isOnPrev = this.prevInputState.isOn(Input.UP);
      if (isOn && !isOnPrev) {
        if (hint.drop > 0) this._game.firmDrop();
        this._game.lock();
        return { isLockedOrHeld: true };
      }
    }
    // Soft drop (down)
    {
      let isOn = this.inputWatcher.isOn(Input.DOWN);
      if (isOn && hint.drop > 0) {
        this._game.drop(Math.min(1, this.opts.softDropGravity));
        isActionDone = true;
      }
    }
    // Rotation (cw, ccw)
    {
      let isCwOn = this.inputWatcher.isOn(Input.CW);
      let isCwOnPrev = this.prevInputState.isOn(Input.CW);
      let isCcwOn = this.inputWatcher.isOn(Input.CCW);
      let isCcwOnPrev = this.prevInputState.isOn(Input.CCW);
      if (isCwOn && !isCcwOn && !isCwOnPrev && hint.cw) {
        this._game.rotate(1);
        isActionDone = true;
      }
      if (isCcwOn && !isCwOn && !isCcwOnPrev && hint.ccw) {
        this._game.rotate(-1);
        isActionDone = true;
      }
    }
    // Shift (right, left)
    {
      let isRightOn = this.inputWatcher.isOn(Input.RIGHT);
      let isLeftOn = this.inputWatcher.isOn(Input.LEFT);
      if (isRightOn && !isLeftOn) {
        if (this.rightwardDasFrameCounter.update(true)) {
          if (hint.right > 0) {
            this._game.shift(1, false);
            isActionDone = true;
          }
        }
      } else {
        this.rightwardDasFrameCounter.update(false);
      }
      if (isLeftOn && !isRightOn) {
        if (this.leftwardDasFrameCounter.update(true)) {
          if (hint.left > 0) {
            this._game.shift(-1, false);
            isActionDone = true;
          }
        }
      } else {
        this.leftwardDasFrameCounter.update(false);
      }
    }

    return { isGameUpdated, isActionDone };
  }
}
