import React from "react";
import { Grid, Header } from "semantic-ui-react";
import { SemanticWIDTHS } from "semantic-ui-react/dist/commonjs/generic";
import { RealTransaction } from "../Models/ImportRow";

interface Props {
  realTransaction: RealTransaction;
  columns?: SemanticWIDTHS;
}

const TransactionSummary: React.FC<Props> = React.memo(({ realTransaction, columns }) => (
  <Grid columns={columns ?? 8}>
    {Object.entries(realTransaction)
      .filter(([, value]) => value)
      .map(([key, value]) => (
        <Grid.Column key={key}>
          <div>
            <Header size="small">{key}</Header>
            {value}
          </div>
        </Grid.Column>
      ))}
  </Grid>
));
TransactionSummary.displayName = "TransactionSummary";

export default TransactionSummary;
