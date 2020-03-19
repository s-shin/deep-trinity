import React from "react";
import ReactDOM from "react-dom";
import { Global, css } from "@emotion/core";
import styled from "@emotion/styled";
import { ThemeProvider } from "emotion-theming";
import * as core from "@deep-trinity/web-core";
import * as view from "@deep-trinity/view";

const game = new core.Game();
game.supplyNextPieces(new Uint8Array([
  core.Piece.L, core.Piece.J, core.Piece.I, core.Piece.O, core.Piece.T, core.Piece.S, core.Piece.Z,
  core.Piece.L, core.Piece.J, core.Piece.I, core.Piece.O, core.Piece.T, core.Piece.S, core.Piece.Z,
]));
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

const STATS_ENTRY_TYPES = Object.values(core.StatisticsEntryType).map(t => t as core.StatisticsEntryType);

const Root = styled.div`
  display: grid;
  align-items: center;
  place-items: center;
  width: 100%;
  height: 100%;
`;

const RootInner = styled.div`
  display: grid;
  grid-template-columns: max-content max-content;
  grid-gap: 2vmin;
`;

ReactDOM.render(
  <ThemeProvider theme={view.DEFAULT_THEME}>
    <Global styles={css`
      body { margin: 0; padding: 0; }
    `}/>
    <Root>
      <RootInner>
        <view.Game game={game}/>
        <view.Statistics game={game} types={STATS_ENTRY_TYPES}/>
      </RootInner>
    </Root>
  </ThemeProvider>,
  document.querySelector("main"),
);
