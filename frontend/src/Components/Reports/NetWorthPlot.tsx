import React, { useEffect, useState } from "react";
import { Container, Loader } from "semantic-ui-react";
import uPlot, { AlignedData } from "uplot";
import { getNetWorth } from "../../Utils/BackendRequester";
import { asEuro } from "../../Utils/TextUtils";
import { Plot } from "../Plot";
import { seriesBarsPlugin } from "./bar";

function opts(o: any, d: AlignedData): uPlot.Options {
  const ori = o.ori;
  const dir = o.dir;
  return {
    title: "Net Worth",
    width: 0,
    height: 400,
    padding: [null, 20, null, 20],
    series: [
      {
        label: "Month",
        value: (_, rawValue) => new Date(rawValue * 1000).toLocaleString("default", { month: "long", year: "numeric" }),
      },
      {
        label: "Assets",
        fill: "#0E6EB8DD",
        value: (_, rawValue) => asEuro(rawValue),
      },
      {
        label: "Debts",
        fill: "#DB2828DD",
        value: (_, rawValue) => asEuro(rawValue),
      },
      {
        label: "Net worth",
        stroke: "#000",
        width: 2,
        points: { size: 10 },
        value: (_, rawValue) => asEuro(rawValue),
      },
    ],
    axes: [
      { space: 100 },
      {
        values: (_, ticks) => ticks.map((rawValue) => asEuro(rawValue, false)),
      },
    ],
    plugins: [
      seriesBarsPlugin({
        labels: () =>
          d[0].map((x) => new Date(x! * 1000).toLocaleString("default", { month: "long", year: "numeric" })),
        bars: [1, 2],
        ori,
        dir,
      }),
    ],
  };
}

interface Props {
  startDate: Date;
}

export const NetWorthPlot: React.FC<Props> = ({ startDate }) => {
  const [netWorth, setNetWorth] = useState<AlignedData | null>(null);
  useEffect(() => {
    getNetWorth(startDate).then((report) => {
      setNetWorth(report);
    });
  }, [startDate]);

  return (
    <Container fluid>
      {netWorth ? (
        <div>
          <React.StrictMode>
            <Plot options={opts({ ori: 0, dir: 1 }, netWorth)} data={netWorth} />
          </React.StrictMode>
        </div>
      ) : (
        <Loader active />
      )}
    </Container>
  );
};
