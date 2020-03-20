import React from "react";
import * as model from "@deep-trinity/model";
export declare type StatisticsProps = {
    game: model.Game;
    types: model.StatisticsEntryType[];
};
export declare const Statistics: React.FC<StatisticsProps>;
export default Statistics;
