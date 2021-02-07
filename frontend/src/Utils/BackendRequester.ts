import { ImportAccount } from "../Models/ImportAccount";
import { ImportRow } from "../Models/ImportRow";
import { Rule } from "../Models/Rule";
import { TransactionRequest } from "../Models/TransactionRequest";

// Using blank host relies on React Proxying when developing
// https://create-react-app.dev/docs/proxying-api-requests-in-development/
const host = !process.env.NODE_ENV || process.env.NODE_ENV === "development" ? "" : "http://tank:8080";

export const getExistingTransactions = (account: ImportAccount): Promise<ImportRow[]> =>
  get(`transactions/existing/${account.path}`);

export const getGeneratedTransactions = (account: ImportAccount): Promise<ImportRow[]> =>
  get(`transactions/generated/${account.path}`);

export const getUnmatchedTransactions = (account: ImportAccount): Promise<ImportRow[]> =>
  get(`transactions/unmatched/${account.path}`);

export const writeGeneratedTransactions = (account: ImportAccount) => post(`transactions/write/${account.path}`);

export const generateSingleTransaction = (request: TransactionRequest) => post("transactions/new", request);

export const getRules = (): Promise<Rule[]> => get("rules");

export const setRule = (rule: Rule): Promise<any> => post("rules", rule);

export const deleteRule = (rule: Rule): Promise<void> => del(`rule/${rule.id}`);

export const getAccounts = (): Promise<string[]> => get("accounts");

const makeUrl = (url: string) => `${host}/${url}`;

const get = <T>(url: string): Promise<T> => {
  console.log(url);
  return fetch(makeUrl(url)).then((response) => {
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