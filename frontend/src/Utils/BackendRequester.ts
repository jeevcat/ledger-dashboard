import { Balance } from "../Models/Balance";
import { ImportAccount } from "../Models/ImportAccount";
import { TransactionResponse } from "../Models/ImportRow";
import { IncomeStatementResponse } from "../Models/IncomeStatementResponse";
import { Rule } from "../Models/Rule";
import { TransactionRequest } from "../Models/TransactionRequest";

// Using blank host relies on React Proxying when developing
// https://create-react-app.dev/docs/proxying-api-requests-in-development/
const host = !process.env.NODE_ENV || process.env.NODE_ENV === "development" ? "" : process.env.REACT_APP_BACKEND_URL;

export const getExistingTransactions = (account: ImportAccount): Promise<TransactionResponse[]> =>
  get(`transactions/existing/${account.id}`);

export const getGeneratedTransactions = (account: ImportAccount): Promise<TransactionResponse[]> =>
  get(`transactions/generated/${account.id}`);

export const getUnmatchedTransactions = (account: ImportAccount): Promise<TransactionResponse[]> =>
  get(`transactions/unmatched/${account.id}`);

export const writeGeneratedTransactions = (account: ImportAccount) => post(`transactions/write/${account.id}`);

export const generateSingleTransaction = (account: ImportAccount, request: TransactionRequest) =>
  post(`transactions/new/${account.id}`, request);

export const getRules = (account: ImportAccount): Promise<Rule[]> => get(`rules/${account.id}`);

export const setRule = (account: ImportAccount, rule: Rule): Promise<any> => post(`rules/${account.id}`, rule);

export const deleteRule = (rule: Rule): Promise<void> => del(`rule/${rule.id}`);

export const getAccounts = (): Promise<string[]> => get("accounts");

export const getBalance = (account: ImportAccount): Promise<Balance> => get(`balance/${account.id}`);

export const getIncomeStatement = (from?: Date, to?: Date): Promise<IncomeStatementResponse> => {
  const query: Record<string, string> = {};
  if (from) {
    query.from = from.toISOString().split("T")[0];
  }
  if (to) {
    query.to = to.toISOString().split("T")[0];
  }
  return get("reports/income_statement", query);
};

const makeUrl = (url: string, query?: Record<string, string>) =>
  query ? `${host}/${url}?` + new URLSearchParams(query) : `${host}/${url}`;

const get = <T>(url: string, query?: Record<string, string>): Promise<T> => {
  return fetch(makeUrl(url, query)).then((response) => {
    if (!response.ok) {
      throw new Error(response.statusText);
    }
    return response.json() as Promise<T>;
  });
};

const post = <T>(url: string, data?: T): Promise<any> => {
  return fetch(makeUrl(url), {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  }).then((response) => {
    return response.json().catch(() => {});
  });
};

const del = (url: string): Promise<void> => {
  return fetch(makeUrl(url), {
    method: "DELETE",
  }).then((response) => {
    if (!response.ok) {
      throw new Error(response.statusText);
    }
  });
};
