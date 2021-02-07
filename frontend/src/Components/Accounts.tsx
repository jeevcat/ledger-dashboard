import React from "react";
import { Grid, Header, Table } from "semantic-ui-react";
import { ImportAccounts } from "../Models/ImportAccount";
import { AccountComponent } from "./Account";

export const AccountsComponent: React.FC = () => {
  const accounts = ImportAccounts.map((x) => <AccountComponent account={x} key={x.path} />);
  return (
    <Grid textAlign="center" style={{ height: "100vh" }} verticalAlign="middle">
      <Grid.Column style={{ maxWidth: 800 }}>
        <Header as="h1">Accounts</Header>
        <Table>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell>Account</Table.HeaderCell>
              <Table.HeaderCell>Real Balance</Table.HeaderCell>
              <Table.HeaderCell>Ledger Balance</Table.HeaderCell>
              <Table.HeaderCell>Import</Table.HeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>{accounts}</Table.Body>
        </Table>
      </Grid.Column>
    </Grid>
  );
};
