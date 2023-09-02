import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import styled from "@emotion/styled";
import * as model from "@deep-trinity/model";
import Game from "./Game";
import Statistics from "./Statistics";
const Root = styled.div `
  display: grid;
  place-items: center;
  width: 100%;
  height: 100%;
`;
const RootInner = styled.div `
  display: grid;
  grid-template-columns: max-content max-content max-content;
  grid-gap: 2vmin;
`;
export function SimpleFullScreenSinglePlay(props) {
    return (_jsx("div", { children: _jsx(Root, { children: _jsxs(RootInner, { children: [_jsx(Game, { game: props.game }), _jsx(Statistics, { game: props.game, types: model.STATISTICS_ENTRY_TYPES }), _jsx("div", { children: props.children })] }) }) }));
}
export default SimpleFullScreenSinglePlay;
//# sourceMappingURL=SimpleFullScreenSinglePlay.js.map