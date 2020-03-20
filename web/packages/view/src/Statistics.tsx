import React from "react";
import styled from "@emotion/styled";
import * as model from "@deep-trinity/model";

const Key = styled.div`
  padding: 0 0.5em;
`;

const Value = styled.div`
  padding: 0 0.5em;
  width: 5em;
`;

export type StatisticsProps = {
  game: model.Game,
  types: model.StatisticsEntryType[],
};

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

export const Statistics: React.FC<StatisticsProps> = props => {
  const rows = props.types.map(t => (
    <tr key={t}>
      <td><Key>{STRS[t]}</Key></td>
      <td><Value>{props.game.stats[t]}</Value></td>
    </tr>
  ));
  return (
    <div>
      <table>
        <tbody>
        {rows}
        </tbody>
      </table>
    </div>
  );
};

export default Statistics;
