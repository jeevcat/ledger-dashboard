import React, { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Button, Icon, Loader, Popup, Table, TableCell } from "semantic-ui-react";
import { Balance } from "../Models/Balance";
import { ImportAccount } from "../Models/ImportAccount";
import { getBalance } from "../Utils/BackendRequester";
import { asEuro } from "../Utils/TextUtils";

interface Props {
  account: ImportAccount;
  onUpdate(accountId: string, balance: Balance): void;
}

export const AccountComponent: React.FC<Props> = ({ account, onUpdate }) => {
  const [balance, setBalance] = useState<Balance>();
  const [failure, setFailure] = useState(false);
  const [bypassCache, setBypassCache] = useState(false);

  const updateBalance = useCallback(
    (bypassCache: boolean) => {
      setBalance(undefined);
      getBalance(account, bypassCache)
        .then((balance) => {
          setBalance(balance);
          setFailure(false);
          onUpdate(account.id, balance);
        })
        .catch((e) => {
          console.error(e);
          setFailure(true);
        })
        .finally(() => setBypassCache(false));
    },
    [account, onUpdate]
  );

  useEffect(() => {
    if (bypassCache) {
      updateBalance(true);
    }
  }, [updateBalance, bypassCache]);

  useEffect(() => {
    updateBalance(false);
  }, [updateBalance]);

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
      const diff = balance.real - balance.hledger;
      const inSync = Math.abs(diff) < 0.1;

      return [
        <Table.Cell key="real">{asEuro(balance.real)}</Table.Cell>,
        <Table.Cell key="hledger">{asEuro(balance.hledger)}</Table.Cell>,
        <Table.Cell key="sync" textAlign="center" negative={!inSync} positive={inSync}>
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
      <Table.Cell textAlign="center">
        <Popup size="mini" trigger={<Button icon="refresh" onClick={() => setBypassCache(true)} />}>
          Request updated data from external API
        </Popup>
      </Table.Cell>
      <Table.Cell textAlign="center">
        <Link to={`/import/${account.id}`}>
          <Button icon="sign-in" />
        </Link>
      </Table.Cell>
    </Table.Row>
  );
};
