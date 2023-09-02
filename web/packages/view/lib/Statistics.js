import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import styled from "@emotion/styled";
const Key = styled.div `
  padding: 0 0.5em;
`;
const Value = styled.div `
  padding: 0 0.5em;
  width: 5em;
`;
const STRS = [
    "Single",
    "Double",
    "Triple",
    "Tetris",
    "TST",
    "TSD",
    "TSS",
    "TSMD",
    "TSMS",
    "Max Combos",
    "Max BTBs",
    "Perfect Clear",
    "Hold",
    "Lock",
];
export function Statistics(props) {
    const rows = props.types.map(t => (_jsxs("tr", { children: [_jsx("td", { children: _jsx(Key, { children: STRS[t] }) }), _jsx("td", { children: _jsx(Value, { children: props.game.stats[t] }) })] }, t)));
    return (_jsx("div", { children: _jsx("table", { children: _jsx("tbody", { children: rows }) }) }));
}
export default Statistics;
//# sourceMappingURL=Statistics.js.map