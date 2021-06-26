import React, { useCallback, useState } from "react";
import { Grid, Header, Icon, Table } from "semantic-ui-react";
import { Balance } from "../Models/Balance";
import { ImportAccounts } from "../Models/ImportAccount";
import { asEuro } from "../Utils/TextUtils";
import { AccountComponent } from "./Account";

export const AccountsComponent: React.FC = () => {
  const [totals, setTotals] = useState<Record<string, Balance>>({});
  const onAccountUpdate = useCallback(
    (accountId: string, balance: Balance) => setTotals((prevState) => ({ ...prevState, [accountId]: balance })),
    []
  );
  const accounts = ImportAccounts.map((x) => <AccountComponent account={x} key={x.id} onUpdate={onAccountUpdate} />);
  const realTotal = Object.keys(totals).reduce((previous, key) => previous + totals[key].real, 0);
  const hledgerTotal = Object.keys(totals).reduce((previous, key) => previous + totals[key].hledger, 0);
  const diff = realTotal - hledgerTotal;
  const inSync = Math.abs(diff) < 0.1;
  return (
    <Grid textAlign="center" verticalAlign="middle" style={{ height: "100vh", margin: 0 }}>
      <Grid.Column style={{ maxWidth: 800 }} textAlign="left">
        <Header as="h1">Accounts</Header>
        <Table celled>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell>Account</Table.HeaderCell>
              <Table.HeaderCell>Real Balance</Table.HeaderCell>
              <Table.HeaderCell>Ledger Balance</Table.HeaderCell>
              <Table.HeaderCell>In Sync</Table.HeaderCell>
              <Table.HeaderCell textAlign="center">Import</Table.HeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>{accounts}</Table.Body>
          <Table.Footer>
            <Table.Row>
              <Table.HeaderCell>Total</Table.HeaderCell>
              <Table.HeaderCell>{asEuro(realTotal)}</Table.HeaderCell>
              <Table.HeaderCell>{asEuro(hledgerTotal)}</Table.HeaderCell>
              <Table.HeaderCell textAlign="center" negative={!inSync} positive={inSync}>
                {inSync ? (
                  <Icon name="check" color="green" />
                ) : (
                  <span>
                    <Icon name="exclamation" />
                    {asEuro(diff)}
                  </span>
                )}
              </Table.HeaderCell>
              <Table.HeaderCell />
            </Table.Row>
          </Table.Footer>
        </Table>
      </Grid.Column>
    </Grid>
  );
};
