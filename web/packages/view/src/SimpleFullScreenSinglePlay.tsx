import React from "react";
import styled from "@emotion/styled";
import * as model from "@deep-trinity/model";
import Game from "./Game";
import Statistics from "./Statistics";

const Root = styled.div`
  display: grid;
  place-items: center;
  width: 100%;
  height: 100%;
`;

const RootInner = styled.div`
  display: grid;
  grid-template-columns: max-content max-content max-content;
  grid-gap: 2vmin;
`;

export type SimpleFullScreenSinglePlayProps = {
  game: model.Game,
  children?: React.ReactNode,
};

export function SimpleFullScreenSinglePlay(props: SimpleFullScreenSinglePlayProps) {
  return (
    <div>
      <Root>
        <RootInner>
          <Game game={props.game}/>
          <Statistics game={props.game} types={model.STATISTICS_ENTRY_TYPES}/>
          <div>{props.children}</div>
        </RootInner>
      </Root>
    </div>
  );
}

export default SimpleFullScreenSinglePlay;
