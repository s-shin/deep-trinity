import React from "react";
import { Global, css } from "@emotion/core";
import styled from "@emotion/styled";
import * as model from "@deep-trinity/model";
import Game from "./Game";
import Statistics from "./Statistics";

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

export type SimpleFullScreenSinglePlayProps = {
  game: model.Game,
};

export const SimpleFullScreenSinglePlay: React.FC<SimpleFullScreenSinglePlayProps> = props => {
  return (
    <div>
      <Global styles={css`
        body { margin: 0; padding: 0; }
      `}/>
      <Root>
        <RootInner>
          <Game game={props.game}/>
          <Statistics game={props.game} types={model.STATISTICS_ENTRY_TYPES}/>
        </RootInner>
      </Root>
    </div>
  );
};

export default SimpleFullScreenSinglePlay;
