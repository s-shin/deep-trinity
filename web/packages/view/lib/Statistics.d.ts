import React from "react";
import * as core from "@deep-trinity/web-core";
export declare type StatisticsProps = {
    game: core.Game;
    types: core.StatisticsEntryType[];
};
export declare const Statistics: React.FC<StatisticsProps>;
export default Statistics;
