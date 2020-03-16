import React from "react";
import ReactDOM from "react-dom";
import { ThemeProvider } from "emotion-theming";
import * as core from "@deep-trinity/core-wasm";
import * as ui from "@deep-trinity/ui";

const game = new core.Game();
game.appendNextPiece(new Uint8Array([
  core.Piece.L, core.Piece.J, core.Piece.I, core.Piece.O, core.Piece.T
]));
game.setupFallingPiece();

ReactDOM.render(
  <ThemeProvider theme={ui.DEFAULT_THEME}>
    <ui.Playfield game={game}/>
  </ThemeProvider>,
  document.querySelector("main"),
);
