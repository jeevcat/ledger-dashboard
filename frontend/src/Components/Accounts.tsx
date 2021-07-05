import React, { useCallback, useState } from "react";
import { Grid, Header, Table } from "semantic-ui-react";
import { Balances } from "../Models/Balance";
import { ImportAccounts } from "../Models/ImportAccount";
import { asEuro } from "../Utils/TextUtils";
import { AccountComponent } from "./Account";

export const AccountsComponent: React.FC = () => {
  const [total, setTotal] = useState<Record<string, number>>({});
  const onAccountUpdate = useCallback(
    (accountId: string, balance: Balances) =>
      setTotal((prevState) => ({
        ...prevState,
        [accountId]: balance.balances.reduce((total, b) => {
          total += b.realEuro ?? b.real;
          return total;
        }, 0),
      })),
    []
  );
  const accounts = ImportAccounts.map((x) => <AccountComponent account={x} key={x.id} onUpdate={onAccountUpdate} />);
  const realTotal = Object.keys(total).reduce((previous, key) => previous + total[key], 0);
  return (
    <Grid textAlign="center" verticalAlign="middle" style={{ height: "100vh", margin: 0 }}>
      <Grid.Column style={{ maxWidth: 800 }} textAlign="left">
        <Header as="h1">Accounts</Header>
        <Table celled structured>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell>Account</Table.HeaderCell>
              <Table.HeaderCell>Commodity</Table.HeaderCell>
              <Table.HeaderCell>Value</Table.HeaderCell>
              <Table.HeaderCell>Real balance</Table.HeaderCell>
              <Table.HeaderCell>
                <i>hledger</i> balance
              </Table.HeaderCell>
              <Table.HeaderCell>Synchronised</Table.HeaderCell>
              <Table.HeaderCell textAlign="center">Refresh</Table.HeaderCell>
              <Table.HeaderCell textAlign="center">Import</Table.HeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>{accounts}</Table.Body>
          <Table.Footer>
            <Table.Row>
              <Table.HeaderCell>Total</Table.HeaderCell>
              <Table.HeaderCell>EUR</Table.HeaderCell>
              <Table.HeaderCell>{asEuro(realTotal)}</Table.HeaderCell>
              <Table.HeaderCell colSpan={5} />
            </Table.Row>
          </Table.Footer>
        </Table>
      </Grid.Column>
    </Grid>
  );
};
