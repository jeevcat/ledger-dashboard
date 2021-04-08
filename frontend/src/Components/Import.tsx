import React, { useEffect, useState } from "react";
import { matchPath, Redirect, Route, Switch, useHistory, useParams, useRouteMatch } from "react-router-dom";
import { Header, Image, Loader, Menu, MenuItemProps, Segment } from "semantic-ui-react";
import { ImportAccounts } from "../Models/ImportAccount";
import { TransactionResponse, RealTransactionField } from "../Models/ImportRow";
import { AccountsContextComponent } from "../Utils/AccountsContext";
import { getExistingTransactions, getGeneratedTransactions, getUnmatchedTransactions } from "../Utils/BackendRequester";
import { RecordTransactionsButton } from "./RecordTransactionsButtons";
import RulesModal from "./Rules/RulesModal";
import { Tab, TransactionTabs } from "./Transactions/TransactionTabs";

export enum TransactionTabType {
  Recorded = "recorded",
  Generated = "generated",
  Unmatched = "unmatched",
}

export const tabs: { [x in TransactionTabType]: Tab } = {
  recorded: {
    name: "Recorded Transactions",
    icon: "hdd",
    transactionSource: getExistingTransactions,
  },
  generated: {
    name: "Generated Transactions",
    icon: "print",
    transactionSource: getGeneratedTransactions,
  },
  unmatched: {
    name: "Unmatched Transactions",
    icon: "question circle",
    transactionSource: getUnmatchedTransactions,
  },
};

interface RouterProps {
  accountName: string;
}

export const Import: React.FC = () => {
  const history = useHistory();
  const { accountName } = useParams<RouterProps>();
  const { url, path } = useRouteMatch();

  const initialTabId = matchPath<{ tabId: TransactionTabType }>(history.location.pathname, {
    path: `${path}/:tabId`,
  })?.params.tabId;

  const [tabId, setTabId] = useState<TransactionTabType>(initialTabId ?? TransactionTabType.Recorded);
  const [areTransactionsLoading, setAreTransactionsLoading] = useState(false);
  const [transactions, setTransactions] = useState<TransactionResponse[]>([]);
  const [realTransactionFields, setRealTransactionFields] = useState<RealTransactionField[]>([]);
  const [filter, setFilter] = useState("");

  useEffect(() => {
    fetchTransactions();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tabId]);

  const handleTabClick = (_: any, data: MenuItemProps) => {
    history.push(url + "/" + data.id);
    setTabId(data.id);
  };

  const collectRealTransactionFields = (ts: TransactionResponse[]): RealTransactionField[] =>
    Array.from(new Set(ts.flatMap((t) => Object.keys(t.real_transaction ?? ""))) as Set<RealTransactionField>).sort();

  const fetchTransactions = () => {
    const newTab = tabs[tabId];
    if (newTab !== undefined && account !== undefined) {
      setAreTransactionsLoading(true);
      newTab
        .transactionSource(account)
        .then((data: TransactionResponse[]) => {
          setTransactions(data);
          setRealTransactionFields(collectRealTransactionFields(data));
        })
        .catch((e) => {
          console.error(`Couldn't fetch transactions: ${e}`);
        })
        .finally(() => {
          setAreTransactionsLoading(false);
        });
    }
  };

  const updateFilter = (newFilter: string) => setFilter(newFilter);

  const getFilteredTransactions = (): TransactionResponse[] =>
    transactions.filter((t) =>
      t.real_transaction
        ? Object.entries(t.real_transaction).some(([, value]) =>
            String(value).toLowerCase().includes(filter.toLowerCase())
          )
        : true
    );

  const accountPath = matchPath<{ accountName: string }>(url, {
    path: path,
  })?.params.accountName!;
  const account = ImportAccounts.find((k) => k.id === accountPath);
  if (account === undefined) {
    return (
      <Header textAlign="center" as="h1" style={{ marginTop: "1em" }}>
        Account with name <strong>{accountName}</strong> does not exist!
      </Header>
    );
  }

  return (
    <div>
      <AccountsContextComponent importAccount={account}>
        <Header textAlign="center" as="h1" icon style={{ marginTop: "1em" }}>
          <Image src={account.icon} circular />
          <Header.Content>{account.humanName}</Header.Content>
        </Header>
        <Menu attached="top" tabular>
          {Object.keys(tabs).map((id) => (
            <Menu.Item
              key={id}
              id={id}
              icon={tabs[id as TransactionTabType].icon}
              name={tabs[id as TransactionTabType].name}
              active={id === tabId}
              onClick={handleTabClick}
            />
          ))}
          <Menu.Item position="right">
            <RulesModal onRulesChanged={fetchTransactions} realTransactionFields={realTransactionFields} />
            <RecordTransactionsButton account={account} onGenerate={fetchTransactions} />
          </Menu.Item>
        </Menu>

        <Segment attached="bottom">
          <Switch>
            <Route path={`${path}/:tabId/:page?`}>
              {areTransactionsLoading ? (
                <Loader active />
              ) : (
                <TransactionTabs
                  transactions={getFilteredTransactions()}
                  defaultColumns={account.defaultColumns}
                  possibleColumns={realTransactionFields}
                  onRuleSaved={fetchTransactions}
                  onTransactionWrite={fetchTransactions}
                  filter={filter}
                  handleFilterChanged={updateFilter}
                />
              )}
            </Route>
            <Redirect to={`${path}/recorded`} />
          </Switch>
        </Segment>
      </AccountsContextComponent>
    </div>
  );
};
