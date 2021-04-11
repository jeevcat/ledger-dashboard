import React, { useEffect, useState } from "react";
import { useHistory, useParams, useRouteMatch } from "react-router-dom";
import {
  Container,
  Dropdown,
  Icon,
  Input,
  Menu,
  Pagination,
  PaginationProps,
  Segment,
  Statistic,
} from "semantic-ui-react";
import { SemanticICONS } from "semantic-ui-react/dist/commonjs/generic";
import { ImportAccount } from "../../Models/ImportAccount";
import { RealTransactionField, TransactionResponse } from "../../Models/ImportRow";
import { maxTransactionsPerPage } from "../../Utils/Config";
import { toTitleCase } from "../../Utils/TextUtils";
import { TransactionTable } from "./TransactionTable";

export interface Tab {
  name: string;
  icon: SemanticICONS;
  transactionSource: (account: ImportAccount) => Promise<TransactionResponse[]>;
}

interface RouterProps {
  tabId: string;
  page?: string;
}

interface Props {
  transactions: TransactionResponse[];
  defaultColumns: RealTransactionField[];
  possibleColumns: RealTransactionField[];
  onRuleSaved: () => void;
  onTransactionWrite: () => void;
  filter: string;
  handleFilterChanged: (newFilter: string) => void;
}

export const TransactionTabs: React.FC<Props> = ({
  transactions,
  defaultColumns,
  possibleColumns,
  filter,
  handleFilterChanged,
  onTransactionWrite,
}) => {
  const history = useHistory();
  const { url } = useRouteMatch();
  const { tabId, page } = useParams<RouterProps>();

  const [pageNum, setPageNum] = useState<number>(1);
  const [selectedFields, setSelectedFields] = useState<RealTransactionField[]>(defaultColumns);

  useEffect(() => {
    setPageNum(parseInt(page ?? "1"));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tabId]);

  const onPageChange = (_: any, data: PaginationProps) => {
    history.push((page ? url.substring(0, url.lastIndexOf("/") + 1) : `${url}/`) + data.activePage);
    setPageNum(data.activePage as number);
  };

  return (
    <div>
      <Menu attached="top">
        <Menu.Menu position="left">
          <Menu.Item header>Columns: </Menu.Item>
          <Dropdown
            placeholder="Select some columns..."
            multiple
            search
            selection
            value={selectedFields}
            options={possibleColumns.map((field) => ({
              value: field,
              text: toTitleCase(field.toString()),
            }))}
            onChange={(_, data) => setSelectedFields(data.value as string[])}
          />
        </Menu.Menu>
        <Menu.Menu position="right">
          <Pagination
            activePage={pageNum}
            totalPages={Math.trunc(transactions.length / maxTransactionsPerPage + 1)}
            onPageChange={onPageChange}
          />
          <Input
            icon="search"
            iconPosition="left"
            placeholder="Search..."
            defaultValue={filter}
            onChange={(_e: any, d: any) => handleFilterChanged(d.value)}
          />
        </Menu.Menu>
      </Menu>
      {transactions.length > 0 ? (
        <TransactionTable
          pageNum={pageNum}
          selectedFields={selectedFields}
          transactions={transactions}
          onTransactionWrite={onTransactionWrite}
        />
      ) : (
        <Container textAlign="center">
          <Icon name="x" size="big" />
          <br />
          Nothing here...
        </Container>
      )}
      <Segment textAlign="center" basic>
        <Statistic>
          <Statistic.Value>{transactions.length}</Statistic.Value>
          <Statistic.Label>Transactions</Statistic.Label>
        </Statistic>
      </Segment>
    </div>
  );
};
