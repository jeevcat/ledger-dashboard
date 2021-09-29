import React, { useCallback, useContext, useEffect, useState } from "react";
import { Table } from "semantic-ui-react";
import { getHledgerAmount } from "../../Models/HledgerTransaction";
import { getRealAmount } from "../../Models/ImportAccount";
import { ExistingTransactionResponse, RealTransaction, TransactionResponse } from "../../Models/ImportRow";
import { AccountsContext } from "../../Utils/AccountsContext";
import { maxTransactionsPerPage } from "../../Utils/Config";
import { toTitleCase } from "../../Utils/TextUtils";
import TransactionTableRow from "./TransactionTableRow";

type Columns = keyof RealTransaction;

type Response = TransactionResponse | ExistingTransactionResponse;
interface Props {
  transactions: Response[];
  pageNum: number;
  selectedFields: Columns[];
  debug: boolean;
  onTransactionWrite: () => void;
}

export const TransactionTable: React.FC<Props> = ({
  transactions,
  pageNum,
  selectedFields,
  onTransactionWrite,
  debug,
}) => {
  const {
    importAccount: { id: importAccountId, dateColumn, amountColumns },
  } = useContext(AccountsContext);

  const [sortedColumn, setSortedColumn] = useState<Columns>(dateColumn);
  const [sortDirection, setSortDirection] = useState<"ascending" | "descending" | undefined>(undefined);

  const handleSort = useCallback(
    (clickedColumn: Columns) =>
      setSortedColumn((prevSortedColumn) => {
        if (prevSortedColumn !== clickedColumn) {
          setSortDirection("ascending");
          return clickedColumn;
        } else {
          setSortDirection((s) => (s === "ascending" ? "descending" : "ascending"));
          return prevSortedColumn;
        }
      }),
    []
  );

  // Default to sorting by date
  useEffect(() => handleSort(dateColumn), [dateColumn, handleSort]);

  const sortCompare = (a: Response, b: Response) => {
    if (sortedColumn === "amt") {
      if (a.real_transaction && b.real_transaction) {
        const valA = getRealAmount(a.real_transaction, amountColumns);
        const valB = getRealAmount(b.real_transaction, amountColumns);
        if (valA && valB) {
          return sortCompareBase(valA, valB);
        }
      }
      if (a.hledger_transaction && b.hledger_transaction) {
        const valA = getHledgerAmount(a.hledger_transaction, importAccountId);
        const valB = getHledgerAmount(b.hledger_transaction, importAccountId);
        if (valA && valB) {
          return sortCompareBase(valA.value, valB.value);
        }
      }
    }
    if (sortedColumn === "rule" && "rule" in a && "rule" in b && a.rule?._id && b.rule?._id) {
      const valA = a.rule._id?.$oid;
      const valB = b.rule._id?.$oid;
      return sortCompareBase(valA, valB);
    }
    if (sortedColumn === "errors" && "errors" in a && "errors" in b && a.errors && b.errors) {
      const valA = a.errors;
      const valB = b.errors;
      return sortCompareBase(valA, valB);
    }
    if (sortedColumn === "generated" && a.hledger_transaction && b.hledger_transaction) {
      const valA = a.hledger_transaction.tdate;
      const valB = b.hledger_transaction.tdate;
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

  const header = (column: Columns, children: React.ReactNode) => (
    <Table.HeaderCell
      key={column}
      sorted={sortedColumn === column ? sortDirection : undefined}
      onClick={() => handleSort(column)}
    >
      {children}
    </Table.HeaderCell>
  );

  return (
    <Table compact celled sortable attached="bottom">
      <Table.Header>
        <Table.Row>
          <Table.HeaderCell />
          {header(dateColumn, "Date")}
          {header("amt", "Amount")}
          {selectedFields.map((field) => header(field, toTitleCase(field.toString())))}
          {"rule" in t && t.rule && header("rule", "Rule")}
          {debug && "real_cumulative" in t && <Table.HeaderCell>Cumulative</Table.HeaderCell>}
          {debug && "hledger_cumulative" in t && <Table.HeaderCell>hledger Cumulative</Table.HeaderCell>}
          {debug && "errors" in t && t.errors && header("errors", "Error")}
          {t.hledger_transaction && (
            <Table.HeaderCell
              collapsing
              sorted={sortedColumn === "generated" ? sortDirection : undefined}
              onClick={() => handleSort("generated")}
            >
              hledger Transaction
            </Table.HeaderCell>
          )}
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {transactions
          .sort(sortCompare)
          .slice((pageNum - 1) * maxTransactionsPerPage, pageNum * maxTransactionsPerPage)
          .map((r, i) => (
            <TransactionTableRow
              key={i}
              realTransactionFields={selectedFields}
              importRow={r}
              onTransactionWrite={onTransactionWrite}
              debug={debug}
            />
          ))}
      </Table.Body>
    </Table>
  );
};
