import * as core from "@deep-trinity/web-core";

const elApp = document.querySelector("main")!;

const elPre = document.createElement("pre");
elApp.appendChild(elPre);

function render(s: string): void {
  elPre.textContent = s;
}

async function run(): Promise<void> {
  core.setPanicHook();

  const game = new core.Game();
  game.supplyNextPieces(
    new Uint8Array([
      core.Piece.O,
      core.Piece.T,
      core.Piece.I,
      core.Piece.J,
      core.Piece.L,
      core.Piece.S,
      core.Piece.Z,
      core.Piece.O,
      core.Piece.T,
      core.Piece.I,
      core.Piece.J,
      core.Piece.L,
      core.Piece.S,
      core.Piece.Z,
    ]),
  );
  game.setupFallingPiece();

  const sleep = (t: number): Promise<void> => new Promise(resolve => setTimeout(resolve, t));

  const actions = [
    () => {
      // O
      game.shift(-1, true);
      game.firmDrop();
      game.lock();
    },
    () => {
      // T
      game.hold();
    },
    () => {
      // I
      game.firmDrop();
      game.lock();
    },
    () => {
      // J
      game.rotate(-1);
      game.shift(1, true);
      game.firmDrop();
      game.lock();
    },
    () => {
      // L
      game.rotate(1);
      game.shift(-1, true);
      game.firmDrop();
      game.lock();
    },
    () => {
      // S
      game.shift(1, false);
      game.firmDrop();
      game.lock();
    },
    () => {
      // Z
      game.shift(-2, false);
      game.rotate(1);
      game.firmDrop();
      game.lock();
    },
    () => {
      game.hold();
    },
    () => {
      // T
      game.shift(1, true);
      game.rotate(-1);
      game.firmDrop();
      game.rotate(-1);
      game.lock();
    },
  ];

  for (const action of actions) {
    action();
    render(game.toString());
    await sleep(500);
  }
}

run().catch(e => console.error(e));
