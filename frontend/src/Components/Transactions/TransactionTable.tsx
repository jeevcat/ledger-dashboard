import React, { useState } from "react";
import { Table } from "semantic-ui-react";
import { ExistingTransactionResponse, RealTransaction, TransactionResponse } from "../../Models/ImportRow";
import { getId } from "../../Models/RecordedTransaction";
import { maxTransactionsPerPage } from "../../Utils/Config";
import { toTitleCase } from "../../Utils/TextUtils";
import TransactionTableRow from "./TransactionTableRow";

type Columns = keyof RealTransaction;

type Response = TransactionResponse | ExistingTransactionResponse;
interface Props {
  transactions: Response[];
  pageNum: number;
  selectedFields: Columns[];
  onTransactionWrite: () => void;
}

export const TransactionTable: React.FC<Props> = ({ transactions, pageNum, selectedFields, onTransactionWrite }) => {
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

  const sortCompare = (a: Response, b: Response) => {
    if (sortedColumn === "rule" && "rule" in a && "rule" in b && a.rule && b.rule) {
      const valA = a.rule.id;
      const valB = b.rule.id;
      return sortCompareBase(valA, valB);
    }
    if (sortedColumn === "errors" && "errors" in a && "errors" in b && a.errors && b.errors) {
      const valA = a.errors;
      const valB = b.errors;
      return sortCompareBase(valA, valB);
    }
    if (!a.real_transaction || !b.real_transaction) {
      return 0;
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

  const t = transactions[0];

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
          {"rule" in t && t.rule && (
            <Table.HeaderCell
              textAlign="center"
              sorted={sortedColumn === "rule" ? sortDirection : undefined}
              onClick={() => handleSort("rule")}
            >
              Rule
            </Table.HeaderCell>
          )}
          {"real_cumulative" in t && <Table.HeaderCell>Cumulative</Table.HeaderCell>}
          {"recorded_cumulative" in t && <Table.HeaderCell>Recorded Cumulative</Table.HeaderCell>}
          {"errors" in t && (
            <Table.HeaderCell
              textAlign="center"
              sorted={sortedColumn === "errors" ? sortDirection : undefined}
              onClick={() => handleSort("errors")}
            >
              Errors
            </Table.HeaderCell>
          )}
          {t.recorded_transaction && <Table.HeaderCell>{"rule" in t ? "Generated" : "Recorded"}</Table.HeaderCell>}
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {transactions
          .sort(sortCompare)
          .slice((pageNum - 1) * maxTransactionsPerPage, pageNum * maxTransactionsPerPage)
          .map((r) => (
            <TransactionTableRow
              key={r.real_transaction?.id ?? getId(r.recorded_transaction!)}
              realTransactionFields={selectedFields}
              importRow={r}
              onTransactionWrite={onTransactionWrite}
            />
          ))}
      </Table.Body>
    </Table>
  );
};
