import React, { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Button, Icon, Loader, Popup, Table, TableCell } from "semantic-ui-react";
import { Balance, Balances } from "../Models/Balance";
import { ImportAccount } from "../Models/ImportAccount";
import { getBalance } from "../Utils/BackendRequester";
import { asCurrency, asEuro } from "../Utils/TextUtils";

interface Props {
  account: ImportAccount;
  onUpdate(accountId: string, balance: Balances): void;
}

export const AccountComponent: React.FC<Props> = ({ account, onUpdate }) => {
  const [balance, setBalance] = useState<Balances>();
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

  const cells = (balance?: Balance) => {
    if (failure) {
      return (
        <TableCell negative colSpan={5} textAlign="center">
          <Icon name="close" />
          Failed to fetch data
        </TableCell>
      );
    }
    if (balance) {
      const diff = balance.real - balance.hledger;
      const inSync = Math.abs(diff) < 0.1;

      return (
        <React.Fragment>
          <Table.Cell key="commodity">{balance.commodity}</Table.Cell>
          <Table.Cell key="value">
            {balance.realEuro ? asEuro(balance.realEuro) : asCurrency(balance.real, balance.commodity)}
          </Table.Cell>
          <Table.Cell key="real">{asCurrency(balance.real, balance.commodity)}</Table.Cell>
          <Table.Cell key="hledger">{asCurrency(balance.hledger, balance.commodity)}</Table.Cell>
          <Table.Cell key="sync" textAlign="center" negative={!inSync} positive={inSync}>
            {inSync ? (
              <Icon name="check" color="green" />
            ) : (
              <span>
                <Icon name="exclamation" />
                {asEuro(diff)}
              </span>
            )}
          </Table.Cell>
        </React.Fragment>
      );
    } else {
      return (
        <Table.Cell colSpan={3} textAlign="center">
          <Loader inline active />
        </Table.Cell>
      );
    }
  };

  const rows = balance?.balances.length ?? 1;

  return (
    <React.Fragment>
      <Table.Row>
        <Table.Cell rowSpan={rows}>{account.humanName}</Table.Cell>
        {cells(balance?.balances[0])}
        <Table.Cell rowSpan={rows} textAlign="center">
          <Popup size="mini" trigger={<Button icon="refresh" onClick={() => setBypassCache(true)} />}>
            Request updated data from external API
          </Popup>
        </Table.Cell>
        <Table.Cell rowSpan={rows} textAlign="center">
          <Link to={`/import/${account.id}`}>
            <Button icon="file import" />
          </Link>
        </Table.Cell>
      </Table.Row>
      {rows > 1 && balance?.balances.slice(1).map((b, i) => <Table.Row key={i}>{cells(b)}</Table.Row>)}
    </React.Fragment>
  );
};
