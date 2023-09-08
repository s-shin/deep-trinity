import * as core from "@deep-trinity/web-core";
import * as coreHelper from "@deep-trinity/web-core-helper";
import SampleApp from "./SampleApp.svelte";

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
game.firmDrop();
game.lock();
game.shift(1, true);
game.firmDrop();
game.lock();
game.hold();
game.shift(-1, true);
game.firmDrop();
game.lock();
const gameModel = coreHelper.getGameModel(game, true);

const app = new SampleApp({
  target: document.querySelector("main")!,
  props: {
    game: gameModel,
  },
});

export default app;
