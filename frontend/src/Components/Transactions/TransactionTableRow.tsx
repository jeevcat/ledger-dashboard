import React from "react";
import { Button, Label, Popup, Table } from "semantic-ui-react";
import { ImportRow, RealTransaction, RealTransactionField } from "../../Models/ImportRow";
import { asCurrency, asDate, asEuro } from "../../Utils/TextUtils";
import RecordTransactionModal from "../RecordTransactionModal";
import TransactionSummary from "../TransactionSummary";
import GeneratedTransaction from "./GeneratedTransaction";

interface Props {
  importRow: ImportRow;
  realTransactionFields: RealTransactionField[];
  importAccount: string;
  accounts: string[];
  onTransactionWrite: () => void;
}

const TransactionTableRow: React.FC<Props> = ({
  importRow,
  realTransactionFields,
  accounts,
  importAccount,
  onTransactionWrite,
}) => {
  const formatField = (field: RealTransactionField) => {
    const val = importRow.real_transaction![field];
    const formatter = formatters[field];
    if (formatter !== undefined) {
      return formatter(val);
    }
    if (!val) {
      return <Label>None</Label>;
    }
    return val;
  };

  const real_transaction_cell = (field: RealTransactionField) => {
    switch (field) {
      case "amount": {
        const pos = importRow.real_transaction![field] > 0;
        return (
          <Table.Cell key={field} positive={pos} negative={!pos}>
            {formatField(field)}
          </Table.Cell>
        );
      }
      default:
        return <Table.Cell key={field}>{formatField(field)}</Table.Cell>;
    }
  };
  return (
    <Table.Row>
      <Table.Cell textAlign="center" verticalAlign="middle">
        <Popup
          flowing
          size="mini"
          content={
            importRow.real_transaction !== undefined && (
              <TransactionSummary realTransaction={importRow.real_transaction} />
            )
          }
          trigger={<Button icon="info" />}
        />
        {importRow.real_transaction && !importRow.recorded_transaction && (
          <RecordTransactionModal
            realTransaction={importRow.real_transaction}
            importAccount={importAccount}
            accounts={accounts}
            onWrite={onTransactionWrite}
          />
        )}
      </Table.Cell>
      {realTransactionFields.map(real_transaction_cell)}
      {importRow.rule && (
        <Table.Cell textAlign="center">
          <Label color="blue">{importRow.rule.ruleName}</Label>
        </Table.Cell>
      )}
      {importRow.recorded_transaction && (
        <Table.Cell>
          <GeneratedTransaction transaction={importRow.recorded_transaction!} importAccount={importAccount} />
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
