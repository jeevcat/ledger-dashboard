import React, { useEffect, useState } from "react";
import { Container, Grid, GridColumn, Header, Loader } from "semantic-ui-react";
import uPlot from "uplot";
import { IncomeStatementResponse } from "../../Models/IncomeStatementResponse";
import { getIncomeStatement } from "../../Utils/BackendRequester";
import { asEuro } from "../../Utils/TextUtils";
import { Plot } from "../Plot";
import GeneratedTransaction from "../Transactions/GeneratedTransaction";

function opts(is: IncomeStatementResponse, onCursor: (self: uPlot) => void): uPlot.Options {
  return {
    title: "Monthly Income Statement",
    width: 0,
    height: 400,
    padding: [null, null, null, 10],
    series: [
      {
        label: "Month",
        value: (_, rawValue) => new Date(rawValue * 1000).toLocaleString("default", { month: "long", year: "numeric" }),
      },
      {
        label: "Revenues",
        stroke: "#21BA45",
        fill: "#21BA4510",
        width: 3,
        value: (_, rawValue) => asEuro(rawValue),
      },
      {
        label: "Expenses",
        stroke: "#DB2828",
        fill: "#DB282810",
        width: 3,
        value: (_, rawValue) => asEuro(rawValue),
      },
    ],
    axes: [
      { space: 100 }, // Encourage months on x axis
      {
        values: (_, ticks) => ticks.map((rawValue) => asEuro(rawValue, false)),
      },
    ],
    plugins: [
      {
        hooks: {
          setCursor: onCursor,
        },
      },
    ],
  };
}

interface Props {
  startDate: Date;
}

export const IncomeStatementPlot: React.FC<Props> = ({ startDate }) => {
  const [incomeStatement, setIncomeStatement] = useState<IncomeStatementResponse | null>(null);
  const [cursorIndex, setCursorIndex] = useState<number | undefined>();
  useEffect(() => {
    getIncomeStatement(startDate).then((is) => {
      console.log(is);
      setIncomeStatement(is);
    });
  }, [startDate]);

  const onSelect = (self: uPlot) => {
    const idx = self.cursor.idx;
    if (idx) {
      setCursorIndex((prev) => idx);
    }
  };

  const maxRows = cursorIndex
    ? Math.max(
        incomeStatement?.topExpenses?.[cursorIndex].length ?? 0,
        incomeStatement?.topRevenues?.[cursorIndex].length ?? 0
      )
    : 0;

  return (
    <Container fluid>
      {incomeStatement ? (
        <div>
          <React.StrictMode>
            <Plot options={opts(incomeStatement, onSelect)} data={incomeStatement.data} />
          </React.StrictMode>
          <br />
          <Container>
            <Grid columns={2} divided>
              <Grid.Row>
                <GridColumn>
                  <Header textAlign="center">Top Revenues</Header>
                </GridColumn>
                <GridColumn>
                  <Header textAlign="center">Top Expenses</Header>
                </GridColumn>
              </Grid.Row>
              {sequence(maxRows).map((i) => {
                const e = cursorIndex ? incomeStatement?.topExpenses?.[cursorIndex][i] : undefined;
                const r = cursorIndex ? incomeStatement?.topRevenues?.[cursorIndex][i] : undefined;
                return (
                  <Grid.Row key={i}>
                    <GridColumn verticalAlign="middle">
                      {r && <GeneratedTransaction transaction={r} account="Income" negative={true} />}
                    </GridColumn>
                    <GridColumn verticalAlign="middle">
                      {e && <GeneratedTransaction transaction={e} account="Expenses" negative={true} />}
                    </GridColumn>
                  </Grid.Row>
                );
              })}
            </Grid>
          </Container>
        </div>
      ) : (
        <Loader active />
      )}
    </Container>
  );
};

const sequence = (max: number) =>
  Array(max)
    .fill(0)
    .map((_, i) => i);
