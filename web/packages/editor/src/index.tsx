import React, { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { ChakraProvider } from "@chakra-ui/react";
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

// game.firmDrop();
// game.firmDrop();
// game.lock();
// game.shift(1, true);
// game.firmDrop();
// game.lock();
// game.hold();
// game.shift(-1, true);
// game.firmDrop();
// game.lock();

const FRAME_TIME_MS: DOMHighResTimeStamp = 16.0;

function App() {
  const [gameModel, setGameModel] = useState(coreHelper.getGameModel(game, true));
  const isLoopEnd = useRef(false);
  const prevFrameTime = useRef(0);
  const activeKeys = useRef(new Set<string>());
  const [activeKeysStr, setActiveKeysStr] = useState("");

  useEffect(() => {
    function updateActiveKeyStr() {
      const keys = [];
      for (const key of activeKeys.current) {
        keys.push(key);
      }
      setActiveKeysStr(keys.join(" "));
    }
    function onKeyDown(e: KeyboardEvent) {
      activeKeys.current.add(e.code);
      updateActiveKeyStr();
    }

    function onKeyUp(e: KeyboardEvent) {
      activeKeys.current.delete(e.code);
      updateActiveKeyStr();
    }

    function onTick(t: DOMHighResTimeStamp) {
      if (!isLoopEnd.current) requestAnimationFrame(onTick);
      if (prevFrameTime.current - t > FRAME_TIME_MS) return;

      if (activeKeys.current.has("ArrowRight")) game.shift(1, false);
      if (activeKeys.current.has("ArrowLeft")) game.shift(-1, false);
      setGameModel(coreHelper.getGameModel(game, false));
    }

    requestAnimationFrame(onTick);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    return () => {
      isLoopEnd.current = true;
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    };
  });

  return (
    <ThemeProvider theme={view.DEFAULT_THEME}>
      <view.SimpleFullScreenSinglePlay game={gameModel} />
      <div>Keys: {activeKeysStr}</div>
    </ThemeProvider>
  );
}

createRoot(document.querySelector("main")!).render(
  <React.StrictMode>
    <ChakraProvider>
      <App />
    </ChakraProvider>
  </React.StrictMode>,
);
