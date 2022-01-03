import React from "react";
import { AppBar, Box, IconButton, Toolbar, Typography } from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import * as model from "@deep-trinity/model";

export type GameEditorProps = {
  // TODO
};

export const GameEditor: React.FC<GameEditorProps> = props => {
  return (
    <Box display="grid" gridTemplateRows="auto 1fr" height="100vh">
      <AppBar position="static">
        <Toolbar variant="dense">
          <Typography variant="h6" color="inherit" component="div">
            Game Editor
          </Typography>
          <IconButton><AddIcon/></IconButton>
        </Toolbar>
      </AppBar>
      <Box display="grid" gridTemplateColumns="200px 1fr 200px">
        <Box bgcolor="grey.100">Foo</Box>
        <Box bgcolor="grey.400">Bar</Box>
        <Box bgcolor="grey.200">Hoge</Box>
      </Box>
    </Box>
  );
};