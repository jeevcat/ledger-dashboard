import React from "react";
import uPlot, { AlignedData } from "uplot";
import { Plot } from "../Plot";

const opts: uPlot.Options = {
  title: "MyPlot",
  width: 800,
  height: 600,
  series: [
    {},
    {
      stroke: "red",
    },
  ],
};

const now = Date.now();

const data: AlignedData = [
  [now, now + 60, now + 120, now + 180],
  [1, 2, 3, 4],
];

export const IncomeStatement: React.FC = () => {
  return (
    <React.StrictMode>
      <Plot options={opts} data={data} />
    </React.StrictMode>
  );
};
