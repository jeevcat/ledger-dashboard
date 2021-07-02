import React, { useContext } from "react";
import { Button, Label, Popup, Table } from "semantic-ui-react";
import { getHledgerAmount } from "../../Models/HledgerTransaction";
import { getRealAmount } from "../../Models/ImportAccount";
import {
  ExistingTransactionResponse,
  RealTransaction,
  RealTransactionField,
  TransactionResponse,
} from "../../Models/ImportRow";
import { AccountsContext } from "../../Utils/AccountsContext";
import { asCurrency, asDate, asEuro } from "../../Utils/TextUtils";
import RecordTransactionModal from "../RecordTransactionModal";
import TransactionSummary from "../TransactionSummary";
import GeneratedTransaction from "./GeneratedTransaction";

type Response = TransactionResponse | ExistingTransactionResponse;

interface Props {
  importRow: Response;
  realTransactionFields: RealTransactionField[];
  debug: boolean;
  onTransactionWrite: () => void;
}

const TransactionTableRow: React.FC<Props> = ({ importRow, realTransactionFields, onTransactionWrite, debug }) => {
  const {
    importAccount: { id: path, dateColumn, amountColumns },
  } = useContext(AccountsContext);

  const formatField = (field: RealTransactionField) => {
    const val = importRow.real_transaction ? importRow.real_transaction[field] : null;
    if (!val) {
      return <Label>None</Label>;
    }
    const formatter = formatters[field];
    if (formatter !== undefined) {
      return formatter(val);
    }
    return val;
  };

  const real_transaction_cell = (field: RealTransactionField) => {
    if (field === "amt") {
      if (importRow.real_transaction) {
        const val = getRealAmount(importRow.real_transaction, amountColumns);
        if (val) {
          const pos = val > 0;
          return (
            <Table.Cell key={field} positive={pos} negative={!pos}>
              {asCurrency(val, importRow.real_transaction.currency ?? importRow.real_transaction.currencyCode ?? "EUR")}
            </Table.Cell>
          );
        }
      }
      if (importRow.hledger_transaction) {
        const val = getHledgerAmount(importRow.hledger_transaction, path);
        if (val) {
          return (
            <Table.Cell key={field} positive={val.positive} negative={!val.positive}>
              {val.formatted}
            </Table.Cell>
          );
        }
      }
    }
    if (field === dateColumn) {
      if (importRow.real_transaction) {
        const val = importRow.real_transaction[field];
        if (val) {
          if (typeof val === "string") {
            return <Table.Cell key={field}>{asDate(val)}</Table.Cell>;
          }
        }
      }
      if (importRow.hledger_transaction) {
        return <Table.Cell key={field}>{asDate(importRow.hledger_transaction.tdate)}</Table.Cell>;
      }
    }
    return <Table.Cell key={field}>{formatField(field)}</Table.Cell>;
  };

  return (
    <Table.Row>
      <Table.Cell textAlign="center" verticalAlign="middle">
        <Popup
          flowing
          size="mini"
          content={
            importRow.real_transaction !== null && <TransactionSummary realTransaction={importRow.real_transaction} />
          }
          trigger={<Button icon="info" />}
        />
        {importRow.real_transaction && !importRow.hledger_transaction && (
          <RecordTransactionModal realTransaction={importRow.real_transaction} onWrite={onTransactionWrite} />
        )}
      </Table.Cell>

      {real_transaction_cell(dateColumn)}
      {real_transaction_cell("amt")}
      {importRow.real_transaction ? (
        realTransactionFields.map(real_transaction_cell)
      ) : (
        <Table.Cell colSpan={realTransactionFields.length} textAlign="center">
          No matching real transaction
        </Table.Cell>
      )}
      {"rule" in importRow && importRow.rule && (
        <Table.Cell textAlign="center">
          <Label color="blue">{importRow.rule.ruleName}</Label>
        </Table.Cell>
      )}
      {debug && "real_cumulative" in importRow && <Table.Cell>{asEuro(importRow.real_cumulative)}</Table.Cell>}
      {debug && "hledger_cumulative" in importRow && <Table.Cell>{asEuro(importRow.hledger_cumulative)}</Table.Cell>}
      {debug && "errors" in importRow && importRow.errors && (
        <Table.Cell textAlign="center">
          {importRow.errors.length > 0 ? (
            importRow.errors.map((e) => <li key={e}>{e}</li>)
          ) : (
            <Label basic color="green">
              None
            </Label>
          )}
        </Table.Cell>
      )}
      {importRow.hledger_transaction && (
        <Table.Cell>
          <GeneratedTransaction transaction={importRow.hledger_transaction!} account={path} />
        </Table.Cell>
      )}
    </Table.Row>
  );
};

const formatters: {
  [key in keyof RealTransaction]?: (...args: any[]) => string;
} = {
  amount: asEuro,
  createdTS: asDate,
  visibleTS: asDate,
  userCertified: asDate,
  confirmed: asDate,
  Date: asDate,
  tradePrice: asEuro,
  ibCommission: asEuro,
};

export default TransactionTableRow;
