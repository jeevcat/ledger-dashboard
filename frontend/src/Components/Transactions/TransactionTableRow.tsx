import React, { useContext } from "react";
import { Button, Label, Popup, Table } from "semantic-ui-react";
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
  onTransactionWrite: () => void;
}

const TransactionTableRow: React.FC<Props> = ({ importRow, realTransactionFields, onTransactionWrite }) => {
  const {
    importAccount: { id: path },
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
    if (importRow.real_transaction) {
      switch (field) {
        case "amount": {
          const pos = importRow.real_transaction[field] > 0;
          return (
            <Table.Cell key={field} positive={pos} negative={!pos}>
              {formatField(field)}
            </Table.Cell>
          );
        }
        default:
          break;
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
      {realTransactionFields.map(real_transaction_cell)}
      {"rule" in importRow && importRow.rule && (
        <Table.Cell textAlign="center">
          <Label color="blue">{importRow.rule.ruleName}</Label>
        </Table.Cell>
      )}
      {"real_cumulative" in importRow && <Table.Cell>{asEuro(importRow.real_cumulative)}</Table.Cell>}
      {"hledger_cumulative" in importRow && <Table.Cell>{asEuro(importRow.hledger_cumulative)}</Table.Cell>}
      {"errors" in importRow && importRow.errors && (
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
  Ammount: asCurrency,
};

export default TransactionTableRow;
