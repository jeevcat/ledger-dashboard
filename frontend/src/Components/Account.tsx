import React, { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Button, Icon, Loader, Table, TableCell } from "semantic-ui-react";
import { Balance } from "../Models/Balance";
import { ImportAccount } from "../Models/ImportAccount";
import { getBalance } from "../Utils/BackendRequester";
import { asEuro } from "../Utils/TextUtils";

interface Props {
  account: ImportAccount;
}

export const AccountComponent: React.FC<Props> = ({ account }) => {
  const [balance, setBalance] = useState<Balance>();
  const [failure, setFailure] = useState<boolean>(false);
  useEffect(() => {
    getBalance(account)
      .then((balance) => {
        setBalance(balance);
        setFailure(false);
      })
      .catch((e) => {
        console.error(e);
        setFailure(true);
      });
  }, [account]);

  const cells = () => {
    if (failure) {
      return (
        <TableCell negative colSpan={3} textAlign="center">
          <Icon name="close" />
          Failed to fetch data
        </TableCell>
      );
    }
    if (balance) {
      const diff = balance.real - balance.recorded;
      const inSync = Math.abs(diff) < 0.1;

      return [
        <Table.Cell key="real">{asEuro(balance.real)}</Table.Cell>,
        <Table.Cell key="recorded">{asEuro(balance.recorded)}</Table.Cell>,
        <Table.Cell key="sync" negative={!inSync}>
          {inSync ? (
            <Icon name="check" color="green" />
          ) : (
            <span>
              <Icon name="exclamation" />
              {asEuro(diff)}
            </span>
          )}
        </Table.Cell>,
      ];
    } else {
      return (
        <Table.Cell colSpan={3} textAlign="center">
          <Loader inline active />
        </Table.Cell>
      );
    }
  };

  return (
    <Table.Row>
      <Table.Cell>{account.humanName}</Table.Cell>
      {cells()}
      <Table.Cell>
        <Link to={`/import/${account.path}`}>
          <Button icon="sign-in" />
        </Link>
      </Table.Cell>
    </Table.Row>
  );
};
