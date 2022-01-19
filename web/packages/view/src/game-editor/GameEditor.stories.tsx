import React from "react";
import { ThemeProvider } from "@emotion/react";
import * as model from "@deep-trinity/model";
import { DEFAULT_THEME } from "../theme";
import { GameEditor } from "./GameEditor";

export default {
  component: GameEditor,
};

const game: model.Game = {
  width: 10,
  height: 40,
  visibleHeight: 20,
  cells: Array(400).fill(model.Cell.Empty),
  nextPieces: "SZLJI".split("").map(c => model.Piece[c as keyof typeof model.Piece]),
  stats: model.STATISTICS_ENTRY_TYPES.reduce((p, c) => {
    p[c.toString()] = 0;
    return p;
  }, {} as { [k: string]: number }) as model.Statistics,
};

export const Default = () => (
  <ThemeProvider theme={DEFAULT_THEME}>
    <GameEditor/>
  </ThemeProvider>
);

Default.parameters = {
  layout: "fullscreen",
};
