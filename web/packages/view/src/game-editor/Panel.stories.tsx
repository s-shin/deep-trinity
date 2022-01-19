import React from "react";
import { ThemeProvider } from "@emotion/react";
import { DEFAULT_THEME } from "../theme";
import { Panel } from "./Panel";

export default {
  component: Panel,
};

export const Default = () => (
  <ThemeProvider theme={DEFAULT_THEME}>
    <Panel title="Foo">
      test
    </Panel>
  </ThemeProvider>
);
