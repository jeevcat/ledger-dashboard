import React, { useState } from "react";
import { Table } from "semantic-ui-react";
import { ImportRow, RealTransaction } from "../../Models/ImportRow";
import { maxTransactionsPerPage } from "../../Utils/Config";
import { toTitleCase } from "../../Utils/TextUtils";
import TransactionTableRow from "./TransactionTableRow";

type Columns = keyof RealTransaction;

interface Props {
  transactions: ImportRow[];
  pageNum: number;
  selectedFields: Columns[];
  importAccount: string;
  onTransactionWrite: () => void;
}

export const TransactionTable: React.FC<Props> = ({
  transactions,
  pageNum,
  selectedFields,
  importAccount,
  onTransactionWrite,
}) => {
  const [sortedColumn, setSortedColumn] = useState<Columns>("visibleTS");
  const [sortDirection, setSortDirection] = useState<"ascending" | "descending" | undefined>(undefined);

  const handleSort = (clickedColumn: Columns) => {
    if (sortedColumn !== clickedColumn) {
      setSortedColumn(clickedColumn);
      setSortDirection("ascending");
    } else {
      setSortDirection(sortDirection === "ascending" ? "descending" : "ascending");
    }
  };

  const sortCompare = (a: ImportRow, b: ImportRow) => {
    if (a.real_transaction === undefined || b.real_transaction === undefined) {
      return 0;
    }
    if (sortedColumn === "rule" && a.rule && b.rule) {
      const valA = a.rule.id;
      const valB = b.rule.id;
      return sortCompareBase(valA, valB);
    }
    const valA = a.real_transaction[sortedColumn];
    const valB = b.real_transaction[sortedColumn];
    return sortCompareBase(valA, valB);
  };

  const sortCompareBase = (a: any, b: any) => {
    if (a > b) {
      return sortDirection === "ascending" ? 1 : -1;
    }
    if (a < b) {
      return sortDirection === "ascending" ? -1 : 1;
    }
    return 0;
  };

  return (
    <Table compact celled sortable attached="bottom">
      <Table.Header>
        <Table.Row>
          <Table.HeaderCell />
          {selectedFields.map((field) => (
            <Table.HeaderCell
              key={field}
              sorted={sortedColumn === field ? sortDirection : undefined}
              onClick={() => handleSort(field)}
            >
              {toTitleCase(field.toString())}
            </Table.HeaderCell>
          ))}
          {transactions[0].rule && (
            <Table.HeaderCell
              textAlign="center"
              sorted={sortedColumn === "rule" ? sortDirection : undefined}
              onClick={() => handleSort("rule")}
            >
              Rule
            </Table.HeaderCell>
          )}
          {transactions[0].recorded_transaction && <Table.HeaderCell>Generated</Table.HeaderCell>}
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {transactions
          .sort(sortCompare)
          .slice((pageNum - 1) * maxTransactionsPerPage, pageNum * maxTransactionsPerPage)
          .map((r) => (
            <TransactionTableRow
              key={r.real_transaction?.id}
              realTransactionFields={selectedFields}
              importRow={r}
              importAccount={importAccount}
              onTransactionWrite={onTransactionWrite}
            />
          ))}
      </Table.Body>
    </Table>
  );
};
