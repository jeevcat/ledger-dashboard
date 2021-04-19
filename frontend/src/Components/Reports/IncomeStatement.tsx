import React, { useEffect, useState } from "react";
import { Container, Loader } from "semantic-ui-react";
import uPlot, { AlignedData } from "uplot";
import { getIncomeStatement } from "../../Utils/BackendRequester";
import { asEuro } from "../../Utils/TextUtils";
import { Plot } from "../Plot";

const opts: uPlot.Options = {
  title: "Monthly Income Statement",
  width: 0,
  height: 600,
  padding: [null, null, null, 10],
  series: [
    {},
    {
      label: "Revenues",
      stroke: "green",
      width: 2,
      value: (_, rawValue) => asEuro(rawValue),
    },
    {
      label: "Expenses",
      stroke: "red",
      width: 2,
      value: (_, rawValue) => asEuro(rawValue),
    },
  ],
  axes: [
    {},
    {
      values: (_, ticks) => ticks.map((rawValue) => asEuro(rawValue, false)),
    },
  ],
};

export const IncomeStatement: React.FC = () => {
  const [data, setData] = useState<AlignedData | null>(null);
  useEffect(() => {
    getIncomeStatement().then((data) => {
      setData(data);
    });
  }, []);
  return (
    <Container fluid>
      {data ? (
        <React.StrictMode>
          <Plot options={opts} data={data} />
        </React.StrictMode>
      ) : (
        <Loader active />
      )}
    </Container>
  );
};
