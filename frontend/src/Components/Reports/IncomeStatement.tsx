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
    plugins: [
      {
        hooks: {
          setCursor: onCursor,
        },
      },
    ],
  };
}

export const IncomeStatement: React.FC = () => {
  const [incomeStatement, setIncomeStatement] = useState<IncomeStatementResponse | null>(null);
  const [cursorIndex, setCursorIndex] = useState<number | undefined>();
  useEffect(() => {
    getIncomeStatement().then((is) => {
      console.log(is);
      setIncomeStatement(is);
    });
  }, []);

  const onSelect = (self: uPlot) => {
    const idx = self.cursor.idx;
    if (idx) {
      setCursorIndex((prev) => idx);
    }
  };

  return (
    <Container fluid>
      {incomeStatement ? (
        <div>
          <React.StrictMode>
            <Plot options={opts(incomeStatement, onSelect)} data={incomeStatement.data} />
          </React.StrictMode>
          <Container>
            <Grid columns={2}>
              <Grid.Row>
                <GridColumn>
                  <Header textAlign="center">Top Revenues</Header>
                </GridColumn>
                <GridColumn>
                  <Header textAlign="center">Top Expenses</Header>
                </GridColumn>
              </Grid.Row>
              {[0, 1, 2].map((i) => {
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
