import React from "react";
import ReactDOM from "react-dom";
import { ThemeProvider } from "emotion-theming";
import * as core from "@deep-trinity/core-wasm";
import * as ui from "@deep-trinity/ui";

const game = new core.Game();
game.appendNextPiece(new Uint8Array([
  core.Piece.L, core.Piece.J, core.Piece.I, core.Piece.O, core.Piece.T, core.Piece.S, core.Piece.Z,
  core.Piece.L, core.Piece.J, core.Piece.I, core.Piece.O, core.Piece.T, core.Piece.S, core.Piece.Z,
]));
game.setupFallingPiece();
game.firmDrop();
game.lock();
game.shift(1, true);
game.firmDrop();
game.lock();

ReactDOM.render(
  <ThemeProvider theme={ui.DEFAULT_THEME}>
    <div>
      <ui.NextPieces pieces={[...game.getNextPieces()]}/>
    </div>
    <div>
      <ui.Playfield game={game}/>
    </div>
  </ThemeProvider>,
  document.querySelector("main"),
);
