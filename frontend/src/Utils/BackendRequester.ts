import { getApiKey } from "../Components/Login/useApiKey";
import { Balances } from "../Models/Balance";
import { ImportAccount } from "../Models/ImportAccount";
import { TransactionResponse } from "../Models/ImportRow";
import { IncomeStatementResponse } from "../Models/IncomeStatementResponse";
import { Rule } from "../Models/Rule";
import { TransactionRequest } from "../Models/TransactionRequest";
import env from "@beam-australia/react-env";

// Using blank host relies on React Proxying when developing
// https://create-react-app.dev/docs/proxying-api-requests-in-development/
// @ts-ignore
const host = env("REACT_APP_BACKEND_URL") ?? "";
console.log("Using backend host " + host);
// @ts-ignore
console.log("Also" + window.__ENV.REACT_APP_BACKEND_URL);

export const getExistingTransactions = (account: ImportAccount, bypassCache: boolean): Promise<TransactionResponse[]> =>
  get(`transactions/existing/${account.id}`, { bypass_cache: bypassCache.toString() });

export const getGeneratedTransactions = (
  account: ImportAccount,
  bypassCache: boolean
): Promise<TransactionResponse[]> =>
  get(`transactions/generated/${account.id}`, { bypass_cache: bypassCache.toString() });

export const getUnmatchedTransactions = (
  account: ImportAccount,
  bypassCache: boolean
): Promise<TransactionResponse[]> =>
  get(`transactions/unmatched/${account.id}`, { bypass_cache: bypassCache.toString() });

export const writeGeneratedTransactions = (account: ImportAccount) => post(`transactions/write/${account.id}`);

export const generateSingleTransaction = (account: ImportAccount, request: TransactionRequest) =>
  post(`transactions/new/${account.id}`, request);

export const getRules = (account: ImportAccount): Promise<Rule[]> => get(`rules/${account.id}`);

export const setRule = (account: ImportAccount, rule: Rule): Promise<any> => post(`rules/${account.id}`, rule);

export const deleteRule = (rule: Rule): Promise<void> => del(`rule/${rule.id}`);

export const getAccounts = (): Promise<string[]> => get("accounts");

export const getBalance = (account: ImportAccount, bypassCache: boolean): Promise<Balances> =>
  get(`balance/${account.id}`, { bypass_cache: bypassCache.toString() });

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

export const getDirtyJournalFiles = (): Promise<string[]> => get("journal/dirty");

export const saveJournal = (body: { commitMsg: string; name: string; email: string }): Promise<void> =>
  post("journal/save", body);

const makeUrl = (url: string, query?: Record<string, string>) =>
  query ? `${host}/${url}?` + new URLSearchParams(query) : `${host}/${url}`;

const makeAuthHeader = () => ({ Authorization: "Basic " + btoa(":" + getApiKey()) });

export const ping = (apiKey: string): Promise<boolean> =>
  fetch(makeUrl("ping"), {
    headers: {
      Authorization: "Basic " + btoa(":" + apiKey),
    },
  }).then((response) => {
    return response.ok;
  });

const get = <T>(url: string, query?: Record<string, string>): Promise<T> => {
  return fetch(makeUrl(url, query), { headers: makeAuthHeader() }).then((response) => {
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
      ...makeAuthHeader(),
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
    headers: makeAuthHeader(),
  }).then((response) => {
    if (!response.ok) {
      throw new Error(response.statusText);
    }
  });
};
