import React from "react";
import { Button, Table } from "semantic-ui-react";
import { Link } from "react-router-dom";
import { ImportAccount } from "../Models/ImportAccount";

interface Props {
  account: ImportAccount;
}

export const AccountComponent: React.FC<Props> = ({ account }) => {
  return (
    <Table.Row>
      <Table.Cell>{account.humanName}</Table.Cell>
      <Table.Cell>1000 EUR</Table.Cell>
      <Table.Cell>1200 EUR</Table.Cell>
      <Table.Cell>
        <Link to={`/import/${account.path}`}>
          <Button icon="sign-in" />
        </Link>
      </Table.Cell>
    </Table.Row>
  );
};
