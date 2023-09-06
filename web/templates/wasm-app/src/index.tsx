import React, { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { ThemeProvider } from "@emotion/react";
import * as core from "@deep-trinity/web-core";
import * as coreHelper from "@deep-trinity/web-core-helper";
import * as view from "@deep-trinity/view";

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

const App: React.FC = () => {
  const [gameModel, setGameModel] = useState(coreHelper.getGameModel(game, true));

  useEffect(() => {
    (async () => {
      game.firmDrop();
      game.lock();
      setGameModel(coreHelper.getGameModel(game, true));
      await new Promise((resolve) => setTimeout(resolve, 1000));
      game.shift(1, true);
      game.firmDrop();
      game.lock();
      setGameModel(coreHelper.getGameModel(game, true));
      await new Promise((resolve) => setTimeout(resolve, 1000));
      game.hold();
      setGameModel(coreHelper.getGameModel(game, true));
      await new Promise((resolve) => setTimeout(resolve, 1000));
      game.shift(-1, true);
      game.firmDrop();
      game.lock();
      setGameModel(coreHelper.getGameModel(game, true));
    })().catch((e) => console.log(e));
  }, []);

  return (
    <ThemeProvider theme={view.DEFAULT_THEME}>
      <view.SimpleFullScreenSinglePlay game={gameModel} />
    </ThemeProvider>
  );
};

createRoot(document.querySelector("main")!).render(<App />);
